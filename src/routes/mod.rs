//! HTTP router wiring.

pub mod payments;
pub mod stats;
pub mod webhook;

use axum::{
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use tower_http::cors::{Any, CorsLayer};

use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    // Dev CORS: allow any origin so the Flutter Web dashboard (served on a
    // different port) can call the API. Tighten to the dashboard origin in prod.
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health))
        .route(
            "/v1/payments",
            get(payments::list_payments).post(payments::create_payment),
        )
        .route("/v1/payments/{id}", get(payments::get_payment))
        .route("/v1/stats", get(stats::get_stats))
        .route("/v1/webhook/{provider}", post(webhook::handle_webhook))
        .layer(cors)
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}
