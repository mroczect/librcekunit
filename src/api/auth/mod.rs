//! Authentication submodule: login and logout.
//!
//! This module re‑exports the `login` and `logout` functions for convenient
//! access from the parent `api` module.
//!
//! # Organization
//!
//! - `login` – authenticates a user and starts a session.
//! - `logout` – terminates the session and cleans up cookies.
//!
//! # Example
//!
//! ```no_run
//! use librcekunit::{Client, Config};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::new("https://example.com");
//! let client = Client::new(config).await?;
//! client.login("user@example.com", "pass").await?;
//! client.logout().await?;
//! # Ok(())
//! # }
//! ```

pub mod login;
pub mod logout;

pub use login::*;
pub use logout::*;
