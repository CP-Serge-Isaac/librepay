//! Runtime config from environment variables.
//!
//! This is the whole trick behind "build now, go live on contract day":
//! the code is fixed, the environment decides which operator is live and with
//! which secrets. `.env` is git-ignored — the public repo ships zero secrets.

use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    /// `host:port` to bind. Default `0.0.0.0:8080`.
    pub bind_addr: String,
    /// Shared secret used to sign/verify webhooks (mock/dev default provided).
    pub webhook_secret: String,
    /// Active provider machine name: "mock" today; "mtn"/"orange"/"wave" later.
    pub default_provider: String,
}

impl Config {
    pub fn from_env() -> Self {
        Config {
            bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            webhook_secret: env::var("WEBHOOK_SECRET")
                .unwrap_or_else(|_| "dev-insecure-change-me".to_string()),
            default_provider: env::var("DEFAULT_PROVIDER")
                .unwrap_or_else(|_| "mock".to_string()),
        }
    }
}
