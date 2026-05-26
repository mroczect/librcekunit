//! Input data management (create and store).
//!
//! This module provides functions to display the input data creation form
//! (`/dashboard/input-data`) and to submit new input data entries. It is
//! designed to work with a Laravel‑style backend that serves an HTML form
//! and accepts POST submissions with CSRF protection.
//!
//! # Security
//!
//! * The [`store`] function automatically injects a CSRF token via
//!   [`HttpClient::request`] before sending the form data.
//! * All functions delegate to [`HttpClient`], which handles cookie
//!   persistence and redirect following.
//!
//! # Example
//!
//! ```no_run
//! use librcekunit::http_client::HttpClient;
//! use librcekunit::Config;
//! use librcekunit::api::input_data;
//! use std::collections::HashMap;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::new("https://example.com");
//! let client = HttpClient::new(&config).await?;
//!
//! // Show creation form
//! let form = input_data::create(&client).await?;
//!
//! // Submit new data
//! let mut data = HashMap::new();
//! data.insert("value".to_string(), "42".to_string());
//! let response = input_data::store(&client, data).await?;
//! # Ok(())
//! # }
//! ```

use crate::error::Error;
use crate::http_client::HttpClient;
use crate::types::HttpMethod;
use std::collections::HashMap;
use tracing::instrument;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Base path for the input data form and submission.
const PATH_INPUT_DATA: &str = "/dashboard/input-data";

// ---------------------------------------------------------------------------
// Public functions
// ---------------------------------------------------------------------------

/// Displays the input data creation form.
///
/// Sends a `GET` request to `/dashboard/input-data`. The server typically
/// responds with an HTML page containing a form for entering new data. This
/// is useful to inspect the form structure or to obtain a fresh CSRF token.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
///
/// # Errors
///
/// * [`Error::Reqwest`] – Network error, timeout, or invalid URL.
/// * [`Error::Api`] – The server returned a non‑success status.
#[instrument(skip(client))]
pub async fn create(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::GET, PATH_INPUT_DATA, None).await
}

/// Stores new input data.
///
/// Sends a `POST` request to `/dashboard/input-data` with the provided form
/// data. A CSRF token is **automatically injected** by
/// [`HttpClient::request`] if not already present in `data`.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `data` – Form data to submit. Keys and values should match the expected
///   server fields.
///
/// # Errors
///
/// * [`Error::Reqwest`] – Network error, timeout, or invalid URL.
/// * [`Error::Api`] – The server returned an error (e.g., validation errors,
///   422 Unprocessable Entity).
/// * [`Error::CsrfNotFound`] – A CSRF token was needed but could not be
///   obtained from the server.
///
/// # Examples
///
/// ```no_run
/// # use librcekunit::http_client::HttpClient;
/// # use librcekunit::api::input_data;
/// # use std::collections::HashMap;
/// # async fn run(client: &HttpClient) -> Result<(), Box<dyn std::error::Error>> {
/// let mut data = HashMap::new();
/// data.insert("name".to_string(), "Sample".to_string());
/// data.insert("value".to_string(), "100".to_string());
/// let response = input_data::store(client, data).await?;
/// # Ok(())
/// # }
/// ```
#[instrument(skip(client))]
pub async fn store(
    client: &HttpClient,
    data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::POST, PATH_INPUT_DATA, Some(data))
        .await
}
