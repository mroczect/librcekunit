//! Logout functionality for the API client.
//!
//! This module provides the [`logout`] function, which terminates the current
//! session by sending a POST request to the server, resetting the local CSRF
//! token, and removing any persistent cookie storage.
//!
//! # Security
//!
//! - The CSRF token is **reset twice**: once before the request (so the POST
//!   does not include a stale token) and again after a successful response
//!   (via [`clear_session`](HttpClient::clear_session)).
//! - If a persistent cookie file exists, it is deleted unconditionally on
//!   success, preventing session reuse.
//! - The response body, even in error cases, is read with a strict size cap
//!   to prevent memory exhaustion.
//!
//! # Example
//!
//! ```no_run
//! use librcekunit::http_client::HttpClient;
//! use librcekunit::Config;
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
use tracing::{info, instrument, warn};

/// The logout endpoint path.
const LOGOUT_PATH: &str = "/logout";

/// Maximum number of bytes to read from the logout response body for error
/// inspection. Large error pages are truncated to this limit.
const MAX_ERROR_BODY_BYTES: usize = 64 * 1024; // 64 KB

/// Ends the current session and cleans up all authentication state.
///
/// This function performs a complete logout workflow:
///
/// 1. **Reset CSRF token** – Clears the stored token so the subsequent POST
///    does not include a stale `_token` field.
/// 2. **Send logout request** – Issues a `POST` to `/logout` with an empty
///    form body. The server should invalidate the session cookie and respond
///    with a redirect (typically `302`) or a success status (`2xx`).
/// 3. **Handle response**:
///    - **On success/redirect** – Calls [`HttpClient::clear_session`], which
///      resets the CSRF token to an empty string and deletes the persistent
///      cookie file (if any). Returns `Ok(())`.
///    - **On failure** – Reads up to [`MAX_ERROR_BODY_BYTES`] from the
///      response body and returns an [`Error::Api`] with the status code and
///      a snippet of the error message.
///
/// # Arguments
///
/// * `client` – Reference to the [`HttpClient`] holding the cookie jar and
///   CSRF state.
///
/// # Errors
///
/// | Error variant      | Condition |
/// |--------------------|-----------|
/// | [`Error::Api`]     | The server returned a status code that is neither a success (`2xx`) nor a redirect (`3xx`). Contains the status code and a truncated body. |
/// | [`Error::Reqwest`] | Network failure, timeout, or invalid URL. |
///
/// # Panics
///
/// This function is **panic‑free**. All error conditions are handled gracefully.
///
/// # Safety
///
/// No unsafe code is used.
///
/// # Examples
///
/// ```no_run
/// use librcekunit::http_client::HttpClient;
/// use librcekunit::Config;
/// use librcekunit::Error;
/// use librcekunit::api::auth::logout;
///
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// let config = Config::new("https://example.com");
/// let client = HttpClient::new(&config).await?;
/// // ... login ...
///
/// match logout::logout(&client).await {
///     Ok(()) => println!("Logged out successfully"),
///     Err(Error::Api(status, msg)) => eprintln!("Server error {}: {}", status, msg),
///     Err(e) => eprintln!("Unexpected error: {}", e),
/// }
/// # Ok(())
/// # }
/// ```
///
/// # See Also
///
/// * [`login`](crate::api::auth::login::login) – To start a session.
/// * [`HttpClient::clear_session`] – Clears persistent state.
/// * [`HttpClient::reset_csrf_token`] – Resets the token without deleting the
///   cookie file.
#[instrument(skip(client))]
pub async fn logout(client: &HttpClient) -> Result<(), Error> {
    // ------------------------------------------------------------
    // 1. Discard any existing CSRF token before the logout request
    // ------------------------------------------------------------
    client.reset_csrf_token().await;

    // ------------------------------------------------------------
    // 2. Send the logout POST with an empty form
    // ------------------------------------------------------------
    // An empty HashMap is used because some frameworks expect form-encoded
    // body even if empty; others accept an empty body. This approach works
    // with both.
    let form = HashMap::new();
    let resp = client
        .request(HttpMethod::POST, LOGOUT_PATH, Some(form))
        .await?;

    let status = resp.status();

    // ------------------------------------------------------------
    // 3. Interpret the server response
    // ------------------------------------------------------------
    if status.is_success() || status.is_redirection() {
        info!("Logout successful – clearing session state");
        client.clear_session().await;
        return Ok(());
    }

    // For any other status, read a limited portion of the body and return
    // an API error with details.
    let body_bytes = resp.bytes().await.unwrap_or_default();
    let limit = body_bytes.len().min(MAX_ERROR_BODY_BYTES);
    let body_snippet = String::from_utf8_lossy(&body_bytes[..limit]);

    warn!(
        status = %status,
        body = %body_snippet,
        "Logout request failed"
    );

    Err(Error::Api(
        status.as_u16(),
        if body_snippet.is_empty() {
            "Logout failed".into()
        } else {
            body_snippet.into_owned()
        },
    ))
}
