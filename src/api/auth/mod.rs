//! Authentication submodule: login and logout.
//!
//! This module provides the fundamental authentication primitives for the
//! crate. It contains two submodules:
//!
//! - [`login`] – Functionality for authenticating a user and starting a session.
//! - [`logout`] – Functionality for terminating a session and cleaning up state.
//!
//! Both functions operate on a shared [`HttpClient`](crate::http_client::HttpClient)
//! and are re‑exported here for convenience. They can also be accessed through
//! the higher‑level [`Client`](crate::Client) wrapper, which delegates to these
//! functions internally.
//!
//! # Usage
//!
//! **Via the high‑level `Client`:**
//!
//! ```no_run
//! use librcekunit::{Client, Config};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::new("https://example.com");
//! let client = Client::new(config).await?;
//!
//! // Login
//! client.login("user@example.com", "pass").await?;
//!
//! // ... perform authenticated requests ...
//!
//! // Logout
//! client.logout().await?;
//! # Ok(())
//! # }
//! ```
//!
//! **Directly with `HttpClient`:**
//!
//! ```no_run
//! use librcekunit::http_client::HttpClient;
//! use librcekunit::Config;
//! use librcekunit::api::auth::{login, logout};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::new("https://example.com");
//! let client = HttpClient::new(&config).await?;
//!
//! login::login(&client, "user@example.com", "pass").await?;
//! // ... use the client ...
//! logout::logout(&client).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Security Considerations
//!
//! * [`login`] handles CSRF token acquisition automatically and ensures the
//!   password is never logged.
//! * [`logout`] resets the CSRF token and deletes persistent cookie files
//!   (if any) to completely destroy the session.
//!
//! # See Also
//!
//! * [`Client`](crate::Client) – The high‑level client that wraps these
//!   functions in convenient methods.
//! * [`HttpClient`](crate::http_client::HttpClient) – The low‑level HTTP
//!   client that holds the session state.

pub mod login;
pub mod logout;

pub use login::*;
pub use logout::*;
