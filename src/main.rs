//! LibrePay — open-source mobile money aggregator API.
//!
//! One HTTP/JSON API in front of many operators. Any client — web (React, Vue,
//! Laravel...) or mobile (Flutter, Kotlin, Swift, React Native) — talks to it
//! the same way. Today it runs on a mock operator with zero setup; the day you
//! sign with MTN/Orange/Wave you add an adapter + env vars and go live. No
//! rewrite.

mod config;
mod error;
mod ledger;
mod models;
mod providers;
mod routes;
mod signature;
mod state;

use std::collections::HashMap;
use std::sync::Arc;

use config::Config;
use ledger::memory::InMemoryLedger;
use providers::{mock::MockProvider, PaymentProvider};
use state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = Config::from_env();

    // Register operators. Add MtnProvider/OrangeProvider/WaveProvider here as
    // you integrate them — the rest of the app is untouched.
    let mut providers: HashMap<String, Arc<dyn PaymentProvider>> = HashMap::new();
    providers.insert("mock".to_string(), Arc::new(MockProvider));

    let state = AppState {
        config: Arc::new(config.clone()),
        ledger: Arc::new(InMemoryLedger::default()),
        providers: Arc::new(providers),
    };

    let app = routes::build_router(state);

    let listener = tokio::net::TcpListener::bind(&config.bind_addr)
        .await
        .expect("failed to bind address");

    tracing::info!(
        addr = %config.bind_addr,
        provider = %config.default_provider,
        "librepay listening"
    );

    axum::serve(listener, app)
        .await
        .expect("server crashed");
}
