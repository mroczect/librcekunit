//! CRUD operations for `cekunit` resources.
//!
//! This module provides functions to list, create, read, update, and delete
//! `cekunit` entries. All functions delegate to `HttpClient` and handle
//! HTTP method routing (including `PUT`/`DELETE` via `_method` injection).
//!
//! # Example
//!
//! ```no_run
//! use librcekunit::{HttpClient, Config};
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
use tracing::instrument;

/// Retrieves all `cekunit` entries (index).
///
/// Sends a GET request to `/cekunit`. Returns the raw HTTP response.
///
/// # Arguments
///
/// * `client` - Shared HTTP client.
///
/// # Errors
///
/// Propagates any network or HTTP error from `HttpClient::request`.
///
/// # Examples
///
/// ```no_run
/// # use librcekunit::{HttpClient, Config, api::crud};
/// # async fn run(client: &HttpClient) -> Result<(), Box<dyn std::error::Error>> {
/// let response = crud::index(client).await?;
/// # Ok(())
/// # }
/// ```
#[instrument(skip(client))]
pub async fn index(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::GET, "/cekunit", None).await
}

/// Retrieves `cekunit` entries with query parameters.
///
/// Sends a GET request to `/cekunit` with the given query parameters.
/// Useful for filtering, sorting, or pagination.
///
/// # Arguments
///
/// * `client` - Shared HTTP client.
/// * `params` - Query parameters as key‑value pairs.
///
/// # Errors
///
/// Propagates network or HTTP errors.
///
/// # Examples
///
/// ```no_run
/// # use librcekunit::{HttpClient, Config, api::crud};
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
        .request(HttpMethod::GET, "/cekunit", Some(params))
        .await
}

/// Displays the form to create a new `cekunit`.
///
/// Sends a GET request to `/cekunit/create`. The response typically contains
/// an HTML form.
///
/// # Arguments
///
/// * `client` - Shared HTTP client.
///
/// # Errors
///
/// Propagates network or HTTP errors.
#[instrument(skip(client))]
pub async fn create(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, "/cekunit/create", None)
        .await
}

/// Stores a new `cekunit` (create).
///
/// Sends a POST request to `/cekunit` with form data. CSRF token is
/// automatically injected by `HttpClient`.
///
/// # Arguments
///
/// * `client` - Shared HTTP client.
/// * `data` - Form data for the new resource.
///
/// # Errors
///
/// Propagates errors including CSRF token fetch failures.
///
/// # Examples
///
/// ```no_run
/// # use librcekunit::{HttpClient, Config, api::crud};
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
        .request(HttpMethod::POST, "/cekunit", Some(data))
        .await
}

/// Shows a specific `cekunit` by its ID `no`.
///
/// Sends a GET request to `/cekunit/{no}`.
///
/// # Arguments
///
/// * `client` - Shared HTTP client.
/// * `no` - The record number (ID) of the `cekunit`.
///
/// # Errors
///
/// Propagates network or HTTP errors. May return a 404 if the record does not exist.
#[instrument(skip(client))]
pub async fn show(client: &HttpClient, no: u64) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, &format!("/cekunit/{}", no), None)
        .await
}

/// Displays the edit form for a `cekunit`.
///
/// Sends a GET request to `/cekunit/{no}/edit`.
///
/// # Arguments
///
/// * `client` - Shared HTTP client.
/// * `no` - Record ID.
///
/// # Errors
///
/// Propagates network or HTTP errors.
#[instrument(skip(client))]
pub async fn edit(client: &HttpClient, no: u64) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, &format!("/cekunit/{}/edit", no), None)
        .await
}

/// Updates a `cekunit` (full update).
///
/// Sends a POST request to `/cekunit/{no}` with form data and injects a
/// `_method=PUT` field to emulate HTTP PUT. CSRF token is added automatically.
///
/// # Arguments
///
/// * `client` - Shared HTTP client.
/// * `no` - Record ID.
/// * `data` - Updated form data.
///
/// # Errors
///
/// Propagates network or HTTP errors.
///
/// # Examples
///
/// ```no_run
/// # use librcekunit::{HttpClient, Config, api::crud};
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
    data.entry("_method".to_string())
        .or_insert_with(|| "PUT".to_string());
    client
        .request(HttpMethod::POST, &format!("/cekunit/{}", no), Some(data))
        .await
}

/// Deletes a `cekunit`.
///
/// Sends a POST request to `/cekunit/{no}` with `_method=DELETE` in the form data
/// to emulate HTTP DELETE. CSRF token is automatically included.
///
/// # Arguments
///
/// * `client` - Shared HTTP client.
/// * `no` - Record ID to delete.
///
/// # Errors
///
/// Propagates network or HTTP errors.
#[instrument(skip(client))]
pub async fn destroy(client: &HttpClient, no: u64) -> Result<reqwest::Response, Error> {
    let mut data = HashMap::new();
    data.insert("_method".to_string(), "DELETE".to_string());
    client
        .request(HttpMethod::POST, &format!("/cekunit/{}", no), Some(data))
        .await
}
