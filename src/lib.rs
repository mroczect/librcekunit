//! client.login("user@example.com", "password").await?;
pub mod api;
pub mod client;
pub mod config;
pub mod cookies;
pub mod error;
pub mod http_client;
pub mod types;

pub use client::Client;
pub use config::{Config, CookieStore};
pub use error::Error;
pub use types::HttpMethod;

pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
}
