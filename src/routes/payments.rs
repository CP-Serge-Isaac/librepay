//! Payment endpoints: create a charge, then poll its status.

use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::{PaymentRequest, PaymentStatus, Transaction};
use crate::providers::InitiateRequest;
use crate::state::AppState;

/// `POST /v1/payments` — start a collection (charge the customer).
///
/// Send header `Idempotency-Key: <uuid>` so a retried request never charges
/// twice. Same key -> same transaction returned, no second charge.
pub async fn create_payment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<PaymentRequest>,
) -> Result<Json<Transaction>, ApiError> {
    if req.amount <= 0 {
        return Err(ApiError::BadRequest("amount must be > 0".into()));
    }
    if req.phone.trim().is_empty() {
        return Err(ApiError::BadRequest("phone is required".into()));
    }

    let idempotency_key = headers
        .get("Idempotency-Key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Idempotency: if we've seen this key, return the existing transaction.
    if let Some(key) = &idempotency_key {
        if let Some(existing) = state
            .ledger
            .find_by_idempotency_key(key)
            .await
            .map_err(ApiError::Internal)?
        {
            return Ok(Json(existing));
        }
    }

    let provider = state
        .provider(&req.provider)
        .ok_or_else(|| ApiError::UnknownProvider(req.provider.clone()))?;

    let id = Uuid::new_v4();
    let init = provider
        .initiate(&InitiateRequest {
            transaction_id: id.to_string(),
            amount: req.amount,
            currency: req.currency.clone(),
            phone: req.phone.clone(),
            reference: req.reference.clone(),
        })
        .await?;

    let now = Utc::now();
    let txn = Transaction {
        id,
        reference: req.reference,
        amount: req.amount,
        currency: req.currency,
        provider: req.provider,
        phone: req.phone,
        status: init.status,
        provider_ref: Some(init.provider_ref),
        callback_url: req.callback_url,
        idempotency_key,
        created_at: now,
        updated_at: now,
    };

    state
        .ledger
        .insert(txn.clone())
        .await
        .map_err(ApiError::Internal)?;

    Ok(Json(txn))
}

/// `GET /v1/payments/{id}` — current status.
///
/// If still `Pending`, we poll the operator (fallback for a missed webhook)
/// and persist any change before returning.
pub async fn get_payment(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Transaction>, ApiError> {
    let txn = state
        .ledger
        .get(id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;

    if txn.status != PaymentStatus::Pending {
        return Ok(Json(txn));
    }

    let provider = state
        .provider(&txn.provider)
        .ok_or_else(|| ApiError::UnknownProvider(txn.provider.clone()))?;

    let provider_ref = txn.provider_ref.clone().unwrap_or_default();
    let latest = provider.check_status(&provider_ref).await?;

    if latest != txn.status {
        let updated = state
            .ledger
            .update_status(id, latest, None)
            .await
            .map_err(ApiError::Internal)?
            .ok_or(ApiError::NotFound)?;
        return Ok(Json(updated));
    }

    Ok(Json(txn))
}
