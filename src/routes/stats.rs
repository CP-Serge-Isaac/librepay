//! Aggregate metrics for the dashboard.

use std::collections::BTreeMap;

use axum::{extract::State, Json};
use serde::Serialize;

use crate::error::ApiError;
use crate::models::PaymentStatus;
use crate::state::AppState;

/// Per-operator breakdown row.
#[derive(Debug, Serialize)]
pub struct ProviderStat {
    pub provider: String,
    pub count: usize,
    /// Sum of amounts for `success` transactions only.
    pub success_volume: i64,
}

/// Response for `GET /v1/stats`.
#[derive(Debug, Serialize)]
pub struct Stats {
    pub total: usize,
    pub pending: usize,
    pub success: usize,
    pub failed: usize,
    /// success / (success + failed), 0.0 if none settled yet.
    pub success_rate: f64,
    /// Total amount of successful transactions (integer base unit, e.g. XOF).
    pub total_volume: i64,
    pub by_provider: Vec<ProviderStat>,
}

/// `GET /v1/stats` — dashboard KPIs computed from the ledger.
pub async fn get_stats(State(state): State<AppState>) -> Result<Json<Stats>, ApiError> {
    // Pull everything (cap high); a Postgres ledger would aggregate in SQL later.
    let txns = state
        .ledger
        .list(None, None, usize::MAX)
        .await
        .map_err(ApiError::Internal)?;

    let mut s = Stats {
        total: txns.len(),
        pending: 0,
        success: 0,
        failed: 0,
        success_rate: 0.0,
        total_volume: 0,
        by_provider: Vec::new(),
    };

    let mut per: BTreeMap<String, (usize, i64)> = BTreeMap::new();
    for t in &txns {
        let entry = per.entry(t.provider.clone()).or_insert((0, 0));
        entry.0 += 1;
        match t.status {
            PaymentStatus::Pending => s.pending += 1,
            PaymentStatus::Success => {
                s.success += 1;
                s.total_volume += t.amount;
                entry.1 += t.amount;
            }
            PaymentStatus::Failed => s.failed += 1,
        }
    }

    let settled = s.success + s.failed;
    if settled > 0 {
        s.success_rate = s.success as f64 / settled as f64;
    }

    s.by_provider = per
        .into_iter()
        .map(|(provider, (count, success_volume))| ProviderStat {
            provider,
            count,
            success_volume,
        })
        .collect();

    Ok(Json(s))
}
