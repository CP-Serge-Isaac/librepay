//! Webhook endpoint — how operators tell us a payment's final status.
//!
//! Real operators POST here asynchronously after the customer approves on their
//! phone. We MUST verify the signature before trusting the body.

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::error::ApiError;
use crate::models::PaymentStatus;
use crate::state::AppState;

/// Body operators send us. Real operators differ — each adapter will normalize
/// its own shape into this. `provider_ref` links back to the transaction.
#[derive(Debug, Deserialize)]
pub struct WebhookPayload {
    pub provider_ref: String,
    pub status: PaymentStatus,
}

/// `POST /v1/webhook/{provider}` — operator callback.
///
/// Header `X-Signature` must be the hex HMAC-SHA256 of the raw body using the
/// shared `WEBHOOK_SECRET`. Invalid signature -> 401, body ignored.
pub async fn handle_webhook(
    State(state): State<AppState>,
    Path(provider_name): Path<String>,
    headers: HeaderMap,
    body: Bytes, // raw bytes: signature must be checked over exactly what was sent
) -> Result<Json<serde_json::Value>, ApiError> {
    let provider = state
        .provider(&provider_name)
        .ok_or_else(|| ApiError::UnknownProvider(provider_name.clone()))?;

    let signature = headers
        .get("X-Signature")
        .and_then(|v| v.to_str().ok())
        .ok_or(ApiError::InvalidSignature)?;

    if !provider.verify_webhook(&body, signature, state.config.webhook_secret.as_bytes()) {
        return Err(ApiError::InvalidSignature);
    }

    let payload: WebhookPayload = serde_json::from_slice(&body)
        .map_err(|e| ApiError::BadRequest(format!("invalid webhook body: {e}")))?;

    let txn = state
        .ledger
        .get_by_provider_ref(&payload.provider_ref)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;

    let updated = state
        .ledger
        .update_status(txn.id, payload.status, None)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;

    // TODO(phase 3): forward a signed notification to `updated.callback_url`
    // so the merchant is pushed the result instead of polling.
    tracing::info!(
        txn_id = %updated.id,
        status = ?updated.status,
        "webhook applied"
    );

    Ok(Json(json!({ "ok": true, "transaction_id": updated.id })))
}
