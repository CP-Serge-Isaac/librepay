//! Ledger abstraction — where transactions live.
//!
//! `InMemoryLedger` today (runs with zero setup). Swap in a `PostgresLedger`
//! (Supabase) later by implementing the same trait — routes don't change.

use async_trait::async_trait;
use uuid::Uuid;

use crate::models::{PaymentStatus, Transaction};

pub mod memory;

#[async_trait]
pub trait Ledger: Send + Sync {
    /// Persist a new transaction.
    async fn insert(&self, txn: Transaction) -> Result<(), String>;

    /// Fetch by our transaction id.
    async fn get(&self, id: Uuid) -> Result<Option<Transaction>, String>;

    /// Fetch by operator-side reference (used when a webhook arrives).
    async fn get_by_provider_ref(&self, provider_ref: &str)
        -> Result<Option<Transaction>, String>;

    /// Idempotency: has this key already produced a transaction?
    async fn find_by_idempotency_key(&self, key: &str)
        -> Result<Option<Transaction>, String>;

    /// Update status (+ provider_ref) and bump `updated_at`.
    async fn update_status(
        &self,
        id: Uuid,
        status: PaymentStatus,
        provider_ref: Option<String>,
    ) -> Result<Option<Transaction>, String>;

    /// List transactions, newest first, optionally filtered. `limit` caps the
    /// number returned. Powers the dashboard's transactions table + metrics.
    async fn list(
        &self,
        status: Option<PaymentStatus>,
        provider: Option<String>,
        limit: usize,
    ) -> Result<Vec<Transaction>, String>;
}
