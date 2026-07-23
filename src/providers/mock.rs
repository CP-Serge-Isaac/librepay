//! Mock operator — works TODAY, no contract, no network.
//!
//! Lets you build and test the whole flow (web + mobile clients, webhooks,
//! ledger) before any partnership. Deterministic so tests are stable:
//!   - `initiate`      -> Pending  (customer would approve on their phone)
//!   - `check_status`  -> Success  (simulates the operator confirming)
//!   - amount <= 0     -> Failed   (lets you exercise the failure path)
//!
//! Real adapters (MtnProvider, OrangeProvider, WaveProvider) will live next to
//! this file and implement the same trait against the operator's HTTP API.

use async_trait::async_trait;
use uuid::Uuid;

use super::{InitiateRequest, InitiateResponse, PaymentProvider, ProviderError};
use crate::models::PaymentStatus;

#[derive(Default)]
pub struct MockProvider;

#[async_trait]
impl PaymentProvider for MockProvider {
    fn name(&self) -> &str {
        "mock"
    }

    async fn initiate(&self, req: &InitiateRequest) -> Result<InitiateResponse, ProviderError> {
        if req.amount <= 0 {
            return Err(ProviderError::Rejected("amount must be positive".into()));
        }
        Ok(InitiateResponse {
            provider_ref: format!("mock_{}", Uuid::new_v4()),
            status: PaymentStatus::Pending,
        })
    }

    async fn check_status(&self, _provider_ref: &str) -> Result<PaymentStatus, ProviderError> {
        // Simulate the operator having confirmed the payment.
        Ok(PaymentStatus::Success)
    }
}
