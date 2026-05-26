//! API submodules for specific resource endpoints.
//!
//! This module organizes all endpoint‑specific logic into focused submodules.
//! Each submodule provides free functions that accept a shared [`HttpClient`]
//! and return raw [`reqwest::Response`] objects, giving callers full control
//! over status inspection and body parsing.
//!
//! The same functionality is also exposed through the higher‑level
//! [`Client`](crate::Client) wrapper, which offers a more ergonomic interface.
//!
//! # Submodules
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`auth`] | Login and logout, including CSRF token handling and session cleanup. |
//! | [`crud`] | Full CRUD (Create, Read, Update, Delete) for `cekunit` resources. |
//! | [`dashboard`] | Main dashboard views, mass deletion, export, and unique‑value queries. |
//! | [`input_data`] | Display and submission of the input‑data form. |
//! | [`input_user`] | Full CRUD, export, and import for input users. |
//! | [`pic`] | Person‑In‑Charge management (CRUD + dashboard + input variants). |
//! | [`users`] | User management (CRUD + dashboard views). |
//!
//! # Usage
//!
//! **Via the high‑level [`Client`](crate::Client):**
//!
//! ```no_run
//! use librcekunit::{Client, Config};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::new("https://example.com");
//! let client = Client::new(config).await?;
//!
//! // Authenticate
//! client.login("user@example.com", "password").await?;
//!
//! // Access resources
//! let dashboard = client.dashboard_index().await?;
//! let all_cekunit = client.cekunit_index().await?;
//! # Ok(())
//! # }
//! ```
//!
//! **Directly with [`HttpClient`](crate::http_client::HttpClient):**
//!
//! ```no_run
//! use librcekunit::http_client::HttpClient;
//! use librcekunit::Config;
//! use librcekunit::api::{crud, dashboard};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::new("https://example.com");
//! let client = HttpClient::new(&config).await?;
//!
//! let resp = crud::index(&client).await?;
//! let unique = dashboard::get_unique_values(&client, "status").await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Security
//!
//! * All mutating requests (POST, PUT, DELETE via `_method`) automatically
//!   include a CSRF token, fetched from the server when needed.
//! * Sensitive data (passwords) are explicitly excluded from tracing spans.
//! * Resource identifiers are validated early to prevent invalid network
//!   requests.
//!
//! # See Also
//!
//! * [`Client`](crate::Client) – The high‑level API wrapper.
//! * [`HttpClient`](crate::http_client::HttpClient) – The low‑level HTTP
//!   client with CSRF and cookie management.

pub mod auth;
pub mod crud;
pub mod dashboard;
pub mod input_data;
pub mod input_user;
pub mod pic;
pub mod users;
