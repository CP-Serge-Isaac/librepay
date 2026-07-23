//! HTTP router wiring.

pub mod payments;
pub mod webhook;

use axum::{
    routing::{get, post},
    Json, Router,
};
use serde_json::json;

use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/payments", post(payments::create_payment))
        .route("/v1/payments/{id}", get(payments::get_payment))
        .route("/v1/webhook/{provider}", post(webhook::handle_webhook))
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}
