//! Unified error type. Every error maps to a clean JSON response so clients
//! (web + mobile, any language) get a predictable shape.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("unknown provider: {0}")]
    UnknownProvider(String),

    #[error("transaction not found")]
    NotFound,

    #[error("invalid request: {0}")]
    BadRequest(String),

    #[error("invalid webhook signature")]
    InvalidSignature,

    #[error("provider error: {0}")]
    Provider(#[from] crate::providers::ProviderError),

    #[error("internal error: {0}")]
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            ApiError::UnknownProvider(_) => (StatusCode::BAD_REQUEST, "unknown_provider"),
            ApiError::NotFound => (StatusCode::NOT_FOUND, "not_found"),
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
            ApiError::InvalidSignature => (StatusCode::UNAUTHORIZED, "invalid_signature"),
            ApiError::Provider(_) => (StatusCode::BAD_GATEWAY, "provider_error"),
            ApiError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal"),
        };
        let body = Json(json!({
            "error": { "code": code, "message": self.to_string() }
        }));
        (status, body).into_response()
    }
}
