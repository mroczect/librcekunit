//! PIC (Person In Charge) management.
//!
//! This module provides functions to manage PIC resources, including
//! standard CRUD operations plus dashboard and input variants.
//!
//! # Example
//!
//! ```no_run
//! use librcekunit::{HttpClient, Config};
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
use tracing::instrument;

/// Retrieves all PIC entries (standard index).
///
/// Sends a GET request to `/pic`.
///
/// # Arguments
///
/// * `client` - Shared HTTP client.
#[instrument(skip(client))]
pub async fn index(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::GET, "/pic", None).await
}

/// Retrieves PIC entries with query parameters.
///
/// # Arguments
///
/// * `client` - Shared HTTP client.
/// * `params` - Query parameters.
#[instrument(skip(client))]
pub async fn index_with_params(
    client: &HttpClient,
    params: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::GET, "/pic", Some(params)).await
}

/// Displays the creation form for a new PIC.
///
/// Sends a GET request to `/pic/create`.
#[instrument(skip(client))]
pub async fn create(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::GET, "/pic/create", None).await
}

/// Stores a new PIC.
///
/// Sends a POST request to `/pic` with form data.
///
/// # Arguments
///
/// * `client` - Shared HTTP client.
/// * `data` - Form data.
#[instrument(skip(client))]
pub async fn store(
    client: &HttpClient,
    data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::POST, "/pic", Some(data)).await
}

/// Shows a specific PIC by its number `nomor`.
///
/// Sends a GET request to `/pic/{nomor}`.
///
/// # Arguments
///
/// * `client` - Shared HTTP client.
/// * `nomor` - PIC identifier.
#[instrument(skip(client))]
pub async fn show(client: &HttpClient, nomor: u64) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, &format!("/pic/{}", nomor), None)
        .await
}

/// Displays the edit form for a PIC.
///
/// Sends a GET request to `/pic/{nomor}/edit`.
#[instrument(skip(client))]
pub async fn edit(client: &HttpClient, nomor: u64) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, &format!("/pic/{}/edit", nomor), None)
        .await
}

/// Updates a PIC.
///
/// Sends a POST request to `/pic/{nomor}` with `_method=PUT` injected.
///
/// # Arguments
///
/// * `client` - Shared HTTP client.
/// * `nomor` - PIC identifier.
/// * `data` - Updated form data.
#[instrument(skip(client))]
pub async fn update(
    client: &HttpClient,
    nomor: u64,
    mut data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    data.entry("_method".to_string())
        .or_insert_with(|| "PUT".to_string());
    client
        .request(HttpMethod::POST, &format!("/pic/{}", nomor), Some(data))
        .await
}

/// Deletes a PIC.
///
/// Sends a POST request to `/pic/{nomor}` with `_method=DELETE`.
#[instrument(skip(client))]
pub async fn destroy(client: &HttpClient, nomor: u64) -> Result<reqwest::Response, Error> {
    let mut data = HashMap::new();
    data.insert("_method".to_string(), "DELETE".to_string());
    client
        .request(HttpMethod::POST, &format!("/pic/{}", nomor), Some(data))
        .await
}

/// Fetches the dashboard PIC index.
///
/// Sends a GET request to `/dashboard/pic`.
#[instrument(skip(client))]
pub async fn dashboard_index(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, "/dashboard/pic", None)
        .await
}

/// Fetches the dashboard PIC index with query parameters.
#[instrument(skip(client))]
pub async fn dashboard_index_with_params(
    client: &HttpClient,
    params: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, "/dashboard/pic", Some(params))
        .await
}

/// Displays the input PIC creation form.
///
/// Sends a GET request to `/dashboard/input-PIC`.
#[instrument(skip(client))]
pub async fn input_create(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, "/dashboard/input-PIC", None)
        .await
}

/// Stores a new input PIC.
///
/// Sends a POST request to `/dashboard/input-PIC` with form data.
#[instrument(skip(client))]
pub async fn input_store(
    client: &HttpClient,
    data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::POST, "/dashboard/input-PIC", Some(data))
        .await
}
