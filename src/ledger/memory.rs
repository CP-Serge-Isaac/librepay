//! In-memory ledger. Zero setup: `cargo run` and it works.
//! Data is lost on restart — fine for dev/demo/tests. Replace with Postgres
//! for anything real (transactions must be durable).

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use super::Ledger;
use crate::models::{PaymentStatus, Transaction};

#[derive(Default)]
pub struct InMemoryLedger {
    inner: Mutex<HashMap<Uuid, Transaction>>,
}

#[async_trait]
impl Ledger for InMemoryLedger {
    async fn insert(&self, txn: Transaction) -> Result<(), String> {
        let mut map = self.inner.lock().map_err(|e| e.to_string())?;
        map.insert(txn.id, txn);
        Ok(())
    }

    async fn get(&self, id: Uuid) -> Result<Option<Transaction>, String> {
        let map = self.inner.lock().map_err(|e| e.to_string())?;
        Ok(map.get(&id).cloned())
    }

    async fn get_by_provider_ref(
        &self,
        provider_ref: &str,
    ) -> Result<Option<Transaction>, String> {
        let map = self.inner.lock().map_err(|e| e.to_string())?;
        Ok(map
            .values()
            .find(|t| t.provider_ref.as_deref() == Some(provider_ref))
            .cloned())
    }

    async fn find_by_idempotency_key(
        &self,
        key: &str,
    ) -> Result<Option<Transaction>, String> {
        let map = self.inner.lock().map_err(|e| e.to_string())?;
        Ok(map
            .values()
            .find(|t| t.idempotency_key.as_deref() == Some(key))
            .cloned())
    }

    async fn update_status(
        &self,
        id: Uuid,
        status: PaymentStatus,
        provider_ref: Option<String>,
    ) -> Result<Option<Transaction>, String> {
        let mut map = self.inner.lock().map_err(|e| e.to_string())?;
        match map.get_mut(&id) {
            Some(txn) => {
                txn.status = status;
                if provider_ref.is_some() {
                    txn.provider_ref = provider_ref;
                }
                txn.updated_at = Utc::now();
                Ok(Some(txn.clone()))
            }
            None => Ok(None),
        }
    }

    async fn list(
        &self,
        status: Option<PaymentStatus>,
        provider: Option<String>,
        limit: usize,
    ) -> Result<Vec<Transaction>, String> {
        let map = self.inner.lock().map_err(|e| e.to_string())?;
        let mut txns: Vec<Transaction> = map
            .values()
            .filter(|t| status.map_or(true, |s| t.status == s))
            .filter(|t| provider.as_deref().map_or(true, |p| t.provider == p))
            .cloned()
            .collect();
        // Newest first.
        txns.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        txns.truncate(limit);
        Ok(txns)
    }
}
