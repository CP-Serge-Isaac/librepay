//! Domain types shared across the API.
//!
//! Money note: XOF (franc CFA) has NO minor unit. Amounts are integer `i64`.
//! Never use floats for money — integer minor units only.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Lifecycle of a payment. Mobile money is asynchronous: a payment starts
/// `Pending`, then the operator confirms `Success` or `Failed` (webhook/polling).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaymentStatus {
    Pending,
    Success,
    Failed,
}

/// Body a merchant sends to `POST /v1/payments`.
#[derive(Debug, Clone, Deserialize)]
pub struct PaymentRequest {
    /// Integer amount in the currency's base unit (XOF has no decimals).
    pub amount: i64,
    #[serde(default = "default_currency")]
    pub currency: String,
    /// Which operator to route to: "mock", "mtn", "orange", "wave"...
    pub provider: String,
    /// Customer mobile money number, E.164 e.g. "+22670000000".
    pub phone: String,
    /// Merchant's own order reference (their side).
    pub reference: String,
    /// Where we notify the merchant when status changes (optional in mock).
    #[serde(default)]
    pub callback_url: Option<String>,
}

fn default_currency() -> String {
    "XOF".to_string()
}

/// A transaction as stored in the ledger and returned to the merchant.
#[derive(Debug, Clone, Serialize)]
pub struct Transaction {
    /// Our transaction id (the one the merchant polls).
    pub id: Uuid,
    pub reference: String,
    pub amount: i64,
    pub currency: String,
    pub provider: String,
    pub phone: String,
    pub status: PaymentStatus,
    /// Reference returned by the operator (their side), if any.
    pub provider_ref: Option<String>,
    #[serde(skip_serializing)]
    pub callback_url: Option<String>,
    #[serde(skip_serializing)]
    pub idempotency_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
