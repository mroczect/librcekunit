//! Logout functionality for the API client.
//!
//! This module provides the `logout` function which ends the current session,
//! clears the CSRF token, and removes persistent cookie storage.
//!
//! # Example
//!
//! ```no_run
//! use librcekunit::{HttpClient, Config};
//! use librcekunit::api::auth::logout;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::new("https://example.com");
//! let client = HttpClient::new(&config).await?;
//! // ... login first ...
//! logout::logout(&client).await?;
//! # Ok(())
//! # }
//! ```

use crate::error::Error;
use crate::http_client::HttpClient;
use crate::types::HttpMethod;
use std::collections::HashMap;
use tracing::instrument;

/// Logs out the current user.
///
/// This function performs the following steps:
/// 1. Resets the local CSRF token (clears it from memory).
/// 2. Sends a POST request to `/logout` with an empty form body.
/// 3. If the response status is a redirect or success (2xx), it calls
///    `client.clear_session()` which removes the persistent cookie file
///    and clears the CSRF token again.
/// 4. If the response status is anything else, returns an API error.
///
/// # Arguments
///
/// * `client` - Shared HTTP client with session state.
///
/// # Errors
///
/// Returns `Error::Api(status, "Logout failed")` if the response status
/// is neither a success nor a redirect.
///
/// Network errors are propagated via `Error::Reqwest`.
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
/// # use librcekunit::{HttpClient, Config, api::auth::logout};
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// let config = Config::new("https://example.com");
/// let client = HttpClient::new(&config).await?;
/// // Assume we are logged in
/// if let Err(e) = logout::logout(&client).await {
///     eprintln!("Logout error: {}", e);
/// }
/// # Ok(())
/// # }
/// ```
///
/// # See Also
///
/// * [`login`](crate::api::auth::login::login) – To start a session.
#[instrument(skip(client))]
pub async fn logout(client: &HttpClient) -> Result<(), Error> {
    client.reset_csrf_token().await;

    let form = HashMap::new();
    let resp = client
        .request(HttpMethod::POST, "/logout", Some(form))
        .await?;

    if resp.status().is_redirection() || resp.status().is_success() {
        client.clear_session().await;
        Ok(())
    } else {
        Err(Error::Api(resp.status().as_u16(), "Logout failed".into()))
    }
}
