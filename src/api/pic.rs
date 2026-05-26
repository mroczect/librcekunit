//! PIC (Person In Charge) management.
//!
//! This module provides a comprehensive set of functions to manage Person In
//! Charge (PIC) resources. It covers standard CRUD operations, a dashboard
//! view, and additional input endpoints. All functions operate through a
//! shared [`HttpClient`] and return raw [`reqwest::Response`] for maximum
//! flexibility.
//!
//! # Security
//!
//! * Mutating operations (`store`, `update`, `destroy`, `input_store`)
//!   automatically inject a CSRF token via [`HttpClient::request`].
//! * Resource identifiers (`nomor`) are validated to be **greater than 0**
//!   to avoid accidental invalid requests.
//!
//! # Example
//!
//! ```no_run
//! use librcekunit::http_client::HttpClient;
//! use librcekunit::Config;
//! use librcekunit::api::pic;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::new("https://example.com");
//! let client = HttpClient::new(&config).await?;
//!
//! // List all PICs
//! let resp = pic::index(&client).await?;
//!
//! // Dashboard view
//! let dashboard = pic::dashboard_index(&client).await?;
//! # Ok(())
//! # }
//! ```

use crate::error::Error;
use crate::http_client::HttpClient;
use crate::types::HttpMethod;
use std::collections::HashMap;
use tracing::{instrument, warn};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Base path for PIC resources.
const PATH_PIC: &str = "/pic";
/// Sub‑path for the creation form.
const PATH_CREATE: &str = "/create";
/// Sub‑path for the edit form.
const PATH_EDIT: &str = "/edit";
/// Path for the dashboard PIC index.
const PATH_DASHBOARD_PIC: &str = "/dashboard/pic";
/// Path for the input PIC form.
const PATH_INPUT_PIC: &str = "/dashboard/input-PIC";

// ---------------------------------------------------------------------------
// Public functions
// ---------------------------------------------------------------------------

/// Retrieves all PIC entries (standard index).
///
/// Sends a `GET` request to `/pic`.
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
pub async fn index(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::GET, PATH_PIC, None).await
}

/// Retrieves PIC entries with query parameters.
///
/// Sends a `GET` request to `/pic?key1=val1&...` using the given `params`.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `params` – Query parameters as key‑value pairs.
///
/// # Errors
///
/// * [`Error::Reqwest`] – Network error, timeout, or invalid URL.
/// * [`Error::Api`] – The server returned a non‑success status.
#[instrument(skip(client))]
pub async fn index_with_params(
    client: &HttpClient,
    params: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, PATH_PIC, Some(params))
        .await
}

/// Displays the creation form for a new PIC.
///
/// Sends a `GET` request to `/pic/create`. The server typically returns an
/// HTML form. This can be used to inspect fields or to fetch a fresh CSRF
/// token.
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
    client
        .request(
            HttpMethod::GET,
            &format!("{}{}", PATH_PIC, PATH_CREATE),
            None,
        )
        .await
}

/// Stores a new PIC.
///
/// Sends a `POST` request to `/pic` with the provided form data. A CSRF
/// token is **automatically injected** by [`HttpClient::request`] if not
/// already present.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `data` – Form data for the new PIC.
///
/// # Errors
///
/// * [`Error::Reqwest`] – Network error, timeout, or invalid URL.
/// * [`Error::CsrfNotFound`] – CSRF token could not be obtained.
/// * [`Error::Api`] – The server returned an error (e.g., validation).
#[instrument(skip(client))]
pub async fn store(
    client: &HttpClient,
    data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::POST, PATH_PIC, Some(data)).await
}

/// Shows a specific PIC by its numeric identifier `nomor`.
///
/// Sends a `GET` request to `/pic/{nomor}`.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `nomor` – The PIC identifier. Must be **greater than 0**.
///
/// # Errors
///
/// * [`Error::Api`] – Returned immediately if `nomor == 0`.
/// * [`Error::Reqwest`] – Network error, timeout, or invalid URL.
/// * [`Error::Api`] – The server returned a non‑success status (e.g., 404 if
///   not found).
///
/// # Panics
///
/// This function is panic‑free.
#[instrument(skip(client))]
pub async fn show(client: &HttpClient, nomor: u64) -> Result<reqwest::Response, Error> {
    if nomor == 0 {
        warn!("Attempted to show PIC with invalid nomor 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    client
        .request(HttpMethod::GET, &format!("{}/{}", PATH_PIC, nomor), None)
        .await
}

/// Displays the edit form for a PIC.
///
/// Sends a `GET` request to `/pic/{nomor}/edit`. The response typically
/// contains an HTML form pre‑filled with the current PIC data.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `nomor` – The PIC identifier. Must be greater than 0.
///
/// # Errors
///
/// * [`Error::Api`] – Returned if `nomor == 0`.
/// * [`Error::Reqwest`] – Network error.
/// * [`Error::Api`] – Server returned an error.
#[instrument(skip(client))]
pub async fn edit(client: &HttpClient, nomor: u64) -> Result<reqwest::Response, Error> {
    if nomor == 0 {
        warn!("Attempted to edit PIC with invalid nomor 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    client
        .request(
            HttpMethod::GET,
            &format!("{}/{}{}", PATH_PIC, nomor, PATH_EDIT),
            None,
        )
        .await
}

/// Updates a PIC.
///
/// Sends a `POST` request to `/pic/{nomor}` with form data and an injected
/// `_method=PUT` field to emulate HTTP PUT. CSRF token is added
/// automatically.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `nomor` – The PIC identifier. Must be greater than 0.
/// * `data` – Updated form data. The map is consumed and a `_method` key is
///   inserted if not already present.
///
/// # Errors
///
/// * [`Error::Api`] – Returned if `nomor == 0`.
/// * [`Error::Reqwest`] – Network error.
/// * [`Error::CsrfNotFound`] – CSRF token could not be obtained.
/// * [`Error::Api`] – Server returned an error (e.g., validation).
#[instrument(skip(client))]
pub async fn update(
    client: &HttpClient,
    nomor: u64,
    mut data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    if nomor == 0 {
        warn!("Attempted to update PIC with invalid nomor 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    data.entry("_method".to_string())
        .or_insert_with(|| "PUT".to_string());
    client
        .request(
            HttpMethod::POST,
            &format!("{}/{}", PATH_PIC, nomor),
            Some(data),
        )
        .await
}

/// Deletes a PIC.
///
/// Sends a `POST` request to `/pic/{nomor}` with `_method=DELETE` in the
/// form data to emulate HTTP DELETE. CSRF token is added automatically.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `nomor` – The PIC identifier to delete. Must be greater than 0.
///
/// # Errors
///
/// * [`Error::Api`] – Returned if `nomor == 0`.
/// * [`Error::Reqwest`] – Network error.
/// * [`Error::CsrfNotFound`] – CSRF token could not be obtained.
/// * [`Error::Api`] – Server returned an error.
#[instrument(skip(client))]
pub async fn destroy(client: &HttpClient, nomor: u64) -> Result<reqwest::Response, Error> {
    if nomor == 0 {
        warn!("Attempted to destroy PIC with invalid nomor 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    let mut data = HashMap::with_capacity(1);
    data.insert("_method".to_string(), "DELETE".to_string());
    client
        .request(
            HttpMethod::POST,
            &format!("{}/{}", PATH_PIC, nomor),
            Some(data),
        )
        .await
}

/// Fetches the dashboard PIC index.
///
/// Sends a `GET` request to `/dashboard/pic`. This endpoint typically
/// provides a summary or administrative view of PICs.
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
pub async fn dashboard_index(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, PATH_DASHBOARD_PIC, None)
        .await
}

/// Fetches the dashboard PIC index with query parameters.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `params` – Query parameters.
///
/// # Errors
///
/// * [`Error::Reqwest`] – Network error.
/// * [`Error::Api`] – Server returned an error.
#[instrument(skip(client))]
pub async fn dashboard_index_with_params(
    client: &HttpClient,
    params: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, PATH_DASHBOARD_PIC, Some(params))
        .await
}

/// Displays the input PIC creation form.
///
/// Sends a `GET` request to `/dashboard/input-PIC`. The response is
/// typically an HTML form used to quickly create a new PIC from the
/// dashboard.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
///
/// # Errors
///
/// * [`Error::Reqwest`] – Network error.
/// * [`Error::Api`] – Server returned an error.
#[instrument(skip(client))]
pub async fn input_create(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::GET, PATH_INPUT_PIC, None).await
}

/// Stores a new input PIC.
///
/// Sends a `POST` request to `/dashboard/input-PIC` with the provided form
/// data. CSRF token is automatically injected.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `data` – Form data for the new PIC.
///
/// # Errors
///
/// * [`Error::Reqwest`] – Network error.
/// * [`Error::CsrfNotFound`] – CSRF token could not be obtained.
/// * [`Error::Api`] – Server returned an error.
#[instrument(skip(client))]
pub async fn input_store(
    client: &HttpClient,
    data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::POST, PATH_INPUT_PIC, Some(data))
        .await
}
