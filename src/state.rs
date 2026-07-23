//! Shared application state, injected into every handler.

use std::collections::HashMap;
use std::sync::Arc;

use crate::config::Config;
use crate::ledger::Ledger;
use crate::providers::PaymentProvider;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub ledger: Arc<dyn Ledger>,
    /// Registered operators, keyed by machine name ("mock", "mtn"...).
    pub providers: Arc<HashMap<String, Arc<dyn PaymentProvider>>>,
}

impl AppState {
    pub fn provider(&self, name: &str) -> Option<Arc<dyn PaymentProvider>> {
        self.providers.get(name).cloned()
    }
}
