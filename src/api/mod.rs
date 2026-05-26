//! API submodules for specific resource endpoints.
//!
//! This module re‑exports all endpoint modules: authentication, CRUD,
//! dashboard, input data, input user, PIC, and users.
//!
//! # Organization
//!
//! - `auth` – Login and logout.
//! - `crud` – Basic CRUD for `cekunit`.
//! - `dashboard` – Dashboard views and bulk actions.
//! - `input_data` – Input data creation and storage.
//! - `input_user` – Full CRUD + export/import for input users.
//! - `pic` – Person In Charge management.
//! - `users` – User management.
//!
//! # Example
//!
//! ```no_run
//! use librcekunit::{Client, Config};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::new("https://example.com");
//! let client = Client::new(config).await?;
//!
//! // Use methods directly on Client
//! client.login("user@example.com", "password").await?;
//! let dashboard = client.dashboard_index().await?;
//! # Ok(())
//! # }
//! ```

pub mod auth;
pub mod crud;
pub mod dashboard;
pub mod input_data;
pub mod input_user;
pub mod pic;
pub mod users;
