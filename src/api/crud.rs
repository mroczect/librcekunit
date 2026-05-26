//! CRUD operations for `cekunit` resources.
//!
//! This module provides the core Create, Read, Update, and Delete (CRUD)
//! functions for the `cekunit` resource. It integrates tightly with
//! [`HttpClient`] to handle HTTP request construction, CSRF token injection,
//! and method override via the `_method` field (for `PUT` and `DELETE`).
//!
//! # Usage
//!
//! All functions accept a shared reference to [`HttpClient`] and return the
//! raw [`reqwest::Response`] on success. This gives callers maximum
//! flexibility to inspect status codes, headers, and bodies.
//!
//! # Example
//!
//! ```no_run
//! use librcekunit::http_client::HttpClient;
//! use librcekunit::Config;
//! use librcekunit::api::crud;
//! use std::collections::HashMap;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::new("https://example.com");
//! let client = HttpClient::new(&config).await?;
//!
//! // List all cekunit
//! let resp = crud::index(&client).await?;
//!
//! // Create a new cekunit
//! let mut data = HashMap::new();
//! data.insert("name".to_string(), "Test".to_string());
//! let resp = crud::store(&client, data).await?;
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

/// Base path for cekunit resource endpoints.
const PATH_CEKUNIT: &str = "/cekunit";
/// Sub‑path for the creation form.
const PATH_CREATE: &str = "/create";
/// Sub‑path for the edit form.
const PATH_EDIT: &str = "/edit";

// ---------------------------------------------------------------------------
// Public functions
// ---------------------------------------------------------------------------

/// Retrieves all `cekunit` entries (index).
///
/// Sends a `GET` request to `/cekunit` without any parameters. The response
/// typically contains an HTML table or a JSON list depending on the server
/// configuration.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
///
/// # Errors
///
/// * [`Error::Reqwest`] – Network error, timeout, or invalid URL.
/// * [`Error::Api`] – The server returned a non‑success status (relayed by
///   [`HttpClient::request`]).
///
/// # Examples
///
/// ```no_run
/// # use librcekunit::http_client::HttpClient;
/// # use librcekunit::api::crud;
/// # async fn run(client: &HttpClient) -> Result<(), Box<dyn std::error::Error>> {
/// let response = crud::index(client).await?;
/// # Ok(())
/// # }
/// ```
#[instrument(skip(client))]
pub async fn index(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::GET, PATH_CEKUNIT, None).await
}

/// Retrieves `cekunit` entries with query parameters.
///
/// Sends a `GET` request to `/cekunit?key1=val1&key2=val2...` using the
/// provided `params` map. This is useful for server‑side filtering,
/// sorting, and pagination.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `params` – Query parameters as key‑value pairs. Keys and values are
///   used as‑is; the caller is responsible for any URL‑encoding if required
///   (though `reqwest` handles basic encoding).
///
/// # Errors
///
/// * [`Error::Reqwest`] – Network error, timeout, or invalid URL.
/// * [`Error::Api`] – The server returned a non‑success status.
///
/// # Examples
///
/// ```no_run
/// # use librcekunit::http_client::HttpClient;
/// # use librcekunit::api::crud;
/// # use std::collections::HashMap;
/// # async fn run(client: &HttpClient) -> Result<(), Box<dyn std::error::Error>> {
/// let mut params = HashMap::new();
/// params.insert("page".to_string(), "2".to_string());
/// let response = crud::index_with_params(client, params).await?;
/// # Ok(())
/// # }
/// ```
#[instrument(skip(client))]
pub async fn index_with_params(
    client: &HttpClient,
    params: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, PATH_CEKUNIT, Some(params))
        .await
}

/// Displays the form to create a new `cekunit`.
///
/// Sends a `GET` request to `/cekunit/create`. The response is typically an
/// HTML form that a browser would render. This function is useful for
/// scraping the form fields or checking the presence of a fresh CSRF token.
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
            &format!("{}{}", PATH_CEKUNIT, PATH_CREATE),
            None,
        )
        .await
}

/// Stores a new `cekunit` resource.
///
/// Sends a `POST` request to `/cekunit` with the given form data. A CSRF
/// token (`_token`) is **automatically injected** by [`HttpClient::request`]
/// if not already present in `data`.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `data` – Form data to submit. Must contain at least the fields required
///   by the server for successful creation.
///
/// # Errors
///
/// * [`Error::Reqwest`] – Network error, timeout, or invalid URL.
/// * [`Error::Api`] – The server returned an error status (e.g., validation
///   errors, 422 Unprocessable Entity).
/// * [`Error::CsrfNotFound`] – A CSRF token was needed but could not be
///   extracted from the server.
///
/// # Examples
///
/// ```no_run
/// # use librcekunit::http_client::HttpClient;
/// # use librcekunit::api::crud;
/// # use std::collections::HashMap;
/// # async fn run(client: &HttpClient) -> Result<(), Box<dyn std::error::Error>> {
/// let mut data = HashMap::new();
/// data.insert("title".to_string(), "New Item".to_string());
/// let response = crud::store(client, data).await?;
/// # Ok(())
/// # }
/// ```
#[instrument(skip(client))]
pub async fn store(
    client: &HttpClient,
    data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::POST, PATH_CEKUNIT, Some(data))
        .await
}

/// Shows a specific `cekunit` by its numeric ID `no`.
///
/// Sends a `GET` request to `/cekunit/{no}`. The server usually returns a
/// detail view or JSON representation of the record.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `no` – The record number (ID). Must be **greater than 0**; an ID of 0
///   will be rejected with an [`Error::Api`] before any network request.
///
/// # Errors
///
/// * [`Error::Api`] – Returned immediately if `no == 0`.
/// * [`Error::Reqwest`] – Network error, timeout, or invalid URL.
/// * [`Error::Api`] – The server returned a non‑success status (e.g., 404
///   if the record does not exist).
///
/// # Panics
///
/// This function is panic‑free.
#[instrument(skip(client))]
pub async fn show(client: &HttpClient, no: u64) -> Result<reqwest::Response, Error> {
    if no == 0 {
        warn!("Attempted to show cekunit with invalid ID 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    client
        .request(HttpMethod::GET, &format!("{}/{}", PATH_CEKUNIT, no), None)
        .await
}

/// Displays the edit form for a `cekunit`.
///
/// Sends a `GET` request to `/cekunit/{no}/edit`. The response typically
/// contains an HTML form pre‑filled with the current values.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `no` – The record number (ID). Must be greater than 0.
///
/// # Errors
///
/// * [`Error::Api`] – Returned if `no == 0`.
/// * [`Error::Reqwest`] – Network error.
/// * [`Error::Api`] – Server returned an error.
#[instrument(skip(client))]
pub async fn edit(client: &HttpClient, no: u64) -> Result<reqwest::Response, Error> {
    if no == 0 {
        warn!("Attempted to edit cekunit with invalid ID 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    client
        .request(
            HttpMethod::GET,
            &format!("{}/{}{}", PATH_CEKUNIT, no, PATH_EDIT),
            None,
        )
        .await
}

/// Updates a `cekunit` resource (full update).
///
/// Sends a `POST` request to `/cekunit/{no}` with the form data and an
/// injected `_method=PUT` field to emulate an HTTP `PUT` request. The CSRF
/// token is added automatically.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `no` – The record number (ID). Must be greater than 0.
/// * `data` – Updated form data. The map is consumed and a `_method` key is
///   inserted if not already present.
///
/// # Errors
///
/// * [`Error::Api`] – Returned if `no == 0`.
/// * [`Error::Reqwest`] – Network error.
/// * [`Error::CsrfNotFound`] – CSRF token could not be obtained.
/// * [`Error::Api`] – Server returned an error (e.g., validation).
///
/// # Examples
///
/// ```no_run
/// # use librcekunit::http_client::HttpClient;
/// # use librcekunit::api::crud;
/// # use std::collections::HashMap;
/// # async fn run(client: &HttpClient) -> Result<(), Box<dyn std::error::Error>> {
/// let mut data = HashMap::new();
/// data.insert("title".to_string(), "Updated Title".to_string());
/// let response = crud::update(client, 42, data).await?;
/// # Ok(())
/// # }
/// ```
#[instrument(skip(client))]
pub async fn update(
    client: &HttpClient,
    no: u64,
    mut data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    if no == 0 {
        warn!("Attempted to update cekunit with invalid ID 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    data.entry("_method".to_string())
        .or_insert_with(|| "PUT".to_string());
    client
        .request(
            HttpMethod::POST,
            &format!("{}/{}", PATH_CEKUNIT, no),
            Some(data),
        )
        .await
}

/// Deletes a `cekunit` resource.
///
/// Sends a `POST` request to `/cekunit/{no}` with `_method=DELETE` in the
/// form data to emulate an HTTP `DELETE` request. CSRF token is automatically
/// added.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `no` – The record number (ID). Must be greater than 0.
///
/// # Errors
///
/// * [`Error::Api`] – Returned if `no == 0`.
/// * [`Error::Reqwest`] – Network error.
/// * [`Error::CsrfNotFound`] – CSRF token could not be obtained.
/// * [`Error::Api`] – Server returned an error.
#[instrument(skip(client))]
pub async fn destroy(client: &HttpClient, no: u64) -> Result<reqwest::Response, Error> {
    if no == 0 {
        warn!("Attempted to destroy cekunit with invalid ID 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    let mut data = HashMap::with_capacity(1);
    data.insert("_method".to_string(), "DELETE".to_string());
    client
        .request(
            HttpMethod::POST,
            &format!("{}/{}", PATH_CEKUNIT, no),
            Some(data),
        )
        .await
}
