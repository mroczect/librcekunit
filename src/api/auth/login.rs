//! Login functionality for the API client.
//!
//! This module provides the `login` function which authenticates a user
//! using email and password. It handles CSRF token fetching, form submission,
//! and session management.
//!
//! # Example
//!
//! ```no_run
//! use librcekunit::{HttpClient, Config};
//! use librcekunit::api::auth::login;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::new("https://example.com");
//! let client = HttpClient::new(&config).await?;
//! login::login(&client, "user@example.com", "password").await?;
//! # Ok(())
//! # }
//! ```

use crate::error::Error;
use crate::http_client::HttpClient;
use crate::types::HttpMethod;
use std::collections::HashMap;
use tracing::{info, instrument};

/// Authenticates a user with email and password.
///
/// This function performs a complete login workflow:
/// 1. Fetches a CSRF token from the `/login` page.
/// 2. Builds a form with `_token`, `email`, and `password`.
/// 3. Sends a POST request to `/login`.
/// 4. On success (HTTP 2xx or redirect), resets the CSRF token and returns `Ok(())`.
/// 5. On client/server error, inspects the response body to distinguish
///    between invalid credentials and other API errors.
///
/// # Arguments
///
/// * `client` - Shared HTTP client with cookie jar and CSRF token storage.
/// * `email` - User's email address. This is logged for tracing (field `email`).
/// * `password` - User's password. It is **skipped** from tracing for security.
///
/// # Errors
///
/// Returns `Error::Auth` if the response body contains the string
/// `"These credentials do not match"` (invalid email/password).
///
/// Returns `Error::Api(status, body)` for any other HTTP error (4xx/5xx)
/// or unexpected status codes.
///
/// Returns `Error::Reqwest` if the network request fails.
///
/// Returns `Error::CsrfNotFound` if the CSRF token cannot be extracted from
/// the login page.
///
/// # Panics
///
/// This function does not panic.
///
/// # Safety
///
/// No unsafe code is used.
///
/// # Examples
///
/// ```no_run
/// # use librcekunit::{HttpClient, Config, api::auth::login};
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// let config = Config::new("https://example.com");
/// let client = HttpClient::new(&config).await?;
///
/// match login::login(&client, "admin@example.com", "wrong_password").await {
///     Err(Error::Auth(msg)) => eprintln!("Login failed: {}", msg),
///     Ok(()) => println!("Logged in successfully"),
///     Err(e) => eprintln!("Unexpected error: {}", e),
/// }
/// # Ok(())
/// # }
/// ```
///
/// # See Also
///
/// * [`logout`](crate::api::auth::logout::logout) – To end the session.
#[instrument(skip(client, password), fields(email = %email))]
pub async fn login(client: &HttpClient, email: &str, password: &str) -> Result<(), Error> {
    let token = client.fetch_csrf_token("/login").await?;

    let mut form = HashMap::new();
    form.insert("_token".to_string(), token);
    form.insert("email".to_string(), email.to_string());
    form.insert("password".to_string(), password.to_string());

    let resp = client
        .request(HttpMethod::POST, "/login", Some(form))
        .await?;

    let status = resp.status();

    if status.is_success() || status.is_redirection() {
        info!("Login successful");
        client.reset_csrf_token().await;
        Ok(())
    } else if status.is_client_error() || status.is_server_error() {
        let body = resp.text().await.unwrap_or_default();
        if body.contains("These credentials do not match") {
            return Err(Error::Auth("Invalid email or password".into()));
        }
        Err(Error::Api(status.as_u16(), body))
    } else {
        Err(Error::Api(status.as_u16(), "Unexpected response".into()))
    }
}
