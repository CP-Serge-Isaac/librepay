//! Provider abstraction — the reusable core.
//!
//! Every operator (mock today; MTN, Orange, Wave the day you sign) implements
//! the SAME `PaymentProvider` trait. The rest of the app never knows which one
//! it talks to. On contract day you add one file + fill env vars. Nothing else
//! changes.

use async_trait::async_trait;

use crate::models::PaymentStatus;

pub mod mock;

/// Errors an operator integration can return.
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("operator rejected the request: {0}")]
    Rejected(String),
    #[error("operator unreachable: {0}")]
    Network(String),
    #[error("misconfigured provider: {0}")]
    Config(String),
}

/// What we hand a provider to start a collection (charge the customer).
#[derive(Debug, Clone)]
pub struct InitiateRequest {
    pub transaction_id: String,
    pub amount: i64,
    pub currency: String,
    pub phone: String,
    pub reference: String,
}

/// What the provider returns after `initiate`.
#[derive(Debug, Clone)]
pub struct InitiateResponse {
    /// Operator-side reference (their id for this payment).
    pub provider_ref: String,
    /// Usually `Pending` — the customer still has to approve on their phone.
    pub status: PaymentStatus,
}

/// The contract every operator adapter must satisfy.
#[async_trait]
pub trait PaymentProvider: Send + Sync {
    /// Machine name: "mock", "mtn", "orange", "wave".
    fn name(&self) -> &str;

    /// Start a collection. Customer approves on their phone (async).
    async fn initiate(&self, req: &InitiateRequest) -> Result<InitiateResponse, ProviderError>;

    /// Poll current status (fallback when a webhook is missed).
    async fn check_status(&self, provider_ref: &str) -> Result<PaymentStatus, ProviderError>;

    /// Verify an inbound webhook is really from this operator.
    /// `secret` is the shared HMAC key; real operators may override with their
    /// own scheme.
    fn verify_webhook(&self, payload: &[u8], signature: &str, secret: &[u8]) -> bool {
        crate::signature::verify_hmac_sha256(payload, signature, secret)
    }
}
