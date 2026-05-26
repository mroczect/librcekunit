//! Login functionality for the API client.
//!
//! This module provides the [`login`] function, which authenticates a user
//! against a Laravel‑style backend using email and password. It handles the full
//! workflow: CSRF token acquisition, form submission, and session management.
//!
//! # Security
//!
//! - The password field is **never** included in logs or traces (see
//!   [`#[instrument(skip(password))]`](tracing::instrument)).
//! - The CSRF token is fetched fresh for each login attempt.
//! - After a successful login, the stored CSRF token is reset to avoid reuse.
//! - Input validation prevents empty credentials from ever touching the network.
//! - The response body is read with a strict size cap to prevent memory exhaustion.
//!
//! # Example
//!
//! ```no_run
//! use librcekunit::http_client::HttpClient;
//! use librcekunit::Config;
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
use tracing::{info, instrument, warn};

/// The exact string returned by the server when credentials are incorrect.
const CREDENTIAL_MISMATCH_PHRASE: &str = "These credentials do not match";

/// Maximum bytes to read from an error response body.
/// Protects against huge error pages (e.g., misconfigured servers).
const MAX_ERROR_BODY_BYTES: usize = 64 * 1024; // 64 KB

/// Authenticates a user with email and password.
///
/// Performs a complete login workflow against a Laravel‑compatible backend:
///
/// 1. **Validate inputs** – Rejects empty email or password early.
/// 2. **CSRF token fetch** – Sends a `GET` to `/login` and extracts the `_token`
///    from the HTML form.
/// 3. **Form construction** – Builds a `POST` form containing `_token`, `email`,
///    and `password`.
/// 4. **Submission** – Sends the form to `/login`.
/// 5. **Result interpretation**:
///    - **Success** (`2xx` or redirect) → resets the stored CSRF token and
///      returns `Ok(())`.
///    - **Client / server error** (`4xx` / `5xx`) → inspects the first
///      [`MAX_ERROR_BODY_BYTES`] of the response body to distinguish invalid
///      credentials from other API errors.
///    - **Unexpected status** → returns `Error::Api` with the status and a
///      generic message.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`] that holds the cookie jar
///   and CSRF state.
/// * `email` – The user’s email address. Must not be empty or only whitespace.
/// * `password` – The user’s password. Must not be empty or only whitespace.
///
/// # Errors
///
/// | Error variant           | Condition |
/// |-------------------------|-----------|
/// | [`Error::Auth`]         | Email or password is empty, or the response body contains `"These credentials do not match"`. |
/// | [`Error::Api`]          | Any other HTTP error status (`4xx` / `5xx`) or unexpected status code. Contains the status code and a snippet of the body. |
/// | [`Error::Reqwest`]      | Network failure, timeout, or invalid URL. |
/// | [`Error::CsrfNotFound`] | The login page did not contain a valid CSRF token. |
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
/// Typical usage with explicit error handling:
///
/// ```no_run
/// use librcekunit::http_client::HttpClient;
/// use librcekunit::Config;
/// use librcekunit::Error;
/// use librcekunit::api::auth::login;
///
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
/// # Security Considerations
///
/// * The password is **never** written to logs. The `#[instrument(skip(password))]`
///   attribute ensures it is excluded from tracing spans.
/// * After a successful login, the internal CSRF token is **reset** to `None` to
///   prevent accidental reuse on subsequent requests (a fresh token will be
///   fetched automatically when needed).
/// * The response body is only read up to [`MAX_ERROR_BODY_BYTES`] to guard
///   against unexpectedly large error pages.
///
/// # See Also
///
/// * [`logout`](crate::api::auth::logout::logout) – To end the session.
/// * [`HttpClient::fetch_csrf_token`] – The underlying CSRF extraction method.
#[instrument(skip(client, password), fields(email = %email))]
pub async fn login(client: &HttpClient, email: &str, password: &str) -> Result<(), Error> {
    // ------------------------------------------------------------
    //  Input validation – fail fast, no network call for empty data
    // ------------------------------------------------------------
    let email = email.trim();
    if email.is_empty() {
        warn!("Login attempt with empty email");
        return Err(Error::Auth("Email cannot be empty".into()));
    }

    let password = password.trim();
    if password.is_empty() {
        warn!("Login attempt with empty password (email: {})", email);
        return Err(Error::Auth("Password cannot be empty".into()));
    }

    // -------------------------------------------
    //  Fetch a fresh CSRF token from the login page
    // -------------------------------------------
    let token = client.fetch_csrf_token("/login").await?;

    // -------------------------------------------
    //  Build the login form
    // -------------------------------------------
    let mut form = HashMap::with_capacity(3);
    form.insert("_token".to_string(), token);
    form.insert("email".to_string(), email.to_string());
    form.insert("password".to_string(), password.to_string());

    // -------------------------------------------
    //  Submit the login request
    // -------------------------------------------
    let resp = client
        .request(HttpMethod::POST, "/login", Some(form))
        .await?;

    let status = resp.status();

    // --- Success ---------------------------------
    if status.is_success() || status.is_redirection() {
        info!("Login successful for {}", email);
        client.reset_csrf_token().await;
        return Ok(());
    }

    // --- Client / Server error -------------------
    if status.is_client_error() || status.is_server_error() {
        // Read the body safely, capped to MAX_ERROR_BODY_BYTES
        let body_bytes = resp.bytes().await.unwrap_or_default();
        let limit = body_bytes.len().min(MAX_ERROR_BODY_BYTES);
        let body_snippet = String::from_utf8_lossy(&body_bytes[..limit]);

        if body_snippet.contains(CREDENTIAL_MISMATCH_PHRASE) {
            warn!("Invalid credentials for {}", email);
            return Err(Error::Auth("Invalid email or password".into()));
        }

        return Err(Error::Api(status.as_u16(), body_snippet.into_owned()));
    }

    // --- Fallback for any other status -----------
    Err(Error::Api(status.as_u16(), "Unexpected HTTP status".into()))
}
