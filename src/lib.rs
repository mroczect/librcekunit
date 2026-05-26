//! # librcekunit
//!
//! A pure Rust library for interacting with the **cekunit** admin panel backend.
//!
//! `librcekunit` provides an HTTP client with automatic CSRF token handling,
//! cookie persistence, and a full set of API bindings for authentication,
//! CRUD operations, dashboard views, and user management. It is designed to
//! be **robust**, **secure**, and **easy to use** – whether you prefer a
//! high‑level [`Client`] or direct control via [`HttpClient`].
//!
//! ## Features
//!
//! * **Full API coverage** – Authentication (login/logout), CRUD for
//!   resources, dashboard, bulk operations, export/import, and more.
//! * **Automatic CSRF handling** – Token extraction from HTML forms and
//!   injection into mutating requests.
//! * **Cookie persistence** – In‑memory or file‑based cookie storage
//!   with automatic save/load.
//! * **Tracing support** – Every request is instrumented for
//!   observability.
//! * **Builder‑style configuration** – `Config` struct with sensible
//!   defaults and chainable setters.
//! * **Strong error handling** – Comprehensive `Error` enum covering all
//!   failure modes.
//!
//! ## Quick start
//!
//! ```no_run
//! use librcekunit::{Client, Config};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a client from environment or manually
//! let config = Config::new("https://admin.example.com");
//! let client = Client::new(config).await?;
//!
//! // Login
//! client.login("user@example.com", "password").await?;
//!
//! // Fetch dashboard
//! let dashboard = client.dashboard_index().await?;
//!
//! // Logout
//! client.logout().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Crate structure
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`api`] | Submodules for all endpoint functions (auth, CRUD, dashboard, etc.) |
//! | [`client`] | High‑level `Client` wrapper around `HttpClient` with convenience methods |
//! | [`config`] | `Config` builder and `CookieStore` enum |
//! | [`cookies`] | Utilities for saving/loading cookies to/from JSON files |
//! | [`error`] | Comprehensive `Error` enum |
//! | [`http_client`] | Low‑level HTTP client with CSRF and cookie handling |
//! | [`types`] | Shared types: `HttpMethod`, `join_url`, `ApiResponse<T>` |
//!
//! ## Re‑exported items
//!
//! For convenience, the following items are re‑exported at the crate root:
//!
//! * [`Client`] – The high‑level API client.
//! * [`Config`] – Configuration builder.
//! * [`CookieStore`] – Cookie persistence strategy.
//! * [`Error`] – All possible errors.
//! * [`HttpMethod`] – HTTP method enum.
//!
//! ## Security
//!
//! * **Password safety** – The `login` function skips the password field
//!   from tracing spans.
//! * **CSRF protection** – Tokens are fetched fresh for each login and
//!   automatically inserted into every mutating request.
//! * **Cookie file permissions** – Files are created with default
//!   permissions; ensure the path is in a secure location when using
//!   `CookieStore::Persistent`.
//!
//! ## Logging
//!
//! Call [`init_tracing`] at the start of your application to enable
//! structured logging via `tracing_subscriber`. The default filter is
//! `info`, but you can override it with the `RUST_LOG` environment
//! variable.
//!
//! ## Further reading
//!
//! * [`Client`] – The recommended entry point for most use cases.
//! * [`HttpClient`] – Low‑level access, direct request building.
//! * [`Config`] – All configuration options with defaults.

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

/// Initializes the `tracing_subscriber` for structured logging.
///
/// This function sets up a subscriber that writes logs to `stderr` with
/// human‑readable formatting. The log level is controlled by the
/// `RUST_LOG` environment variable, defaulting to `info` if not set.
///
/// # Examples
///
/// ```
/// librcekunit::init_tracing();
/// ```
///
/// # Panics
///
/// Panics if called twice (the global subscriber can only be set once).
/// Consider using `tracing::subscriber::set_global_default` or
/// `try_init` for more control.
pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
}
