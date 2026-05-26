//! Dashboard and bulk operations.
//!
//! This module provides functions for the main dashboard view and additional
//! operations such as mass deletion, export, and fetching unique values.
//!
//! # Example
//!
//! ```no_run
//! use librcekunit::{HttpClient, Config};
//! use librcekunit::api::dashboard;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::new("https://example.com");
//! let client = HttpClient::new(&config).await?;
//!
//! let dashboard = dashboard::dashboard_index(&client).await?;
//! let unique = dashboard::get_unique_values(&client, "category").await?;
//! # Ok(())
//! # }
//! ```

use crate::error::Error;
use crate::http_client::HttpClient;
use crate::types::HttpMethod;
use std::collections::HashMap;
use tracing::instrument;

/// Fetches the main dashboard page.
///
/// Sends a GET request to `/dashboard`.
///
/// # Arguments
///
/// * `client` - Shared HTTP client.
///
/// # Errors
///
/// Propagates network or HTTP errors.
#[instrument(skip(client))]
pub async fn dashboard_index(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::GET, "/dashboard", None).await
}

/// Fetches the dashboard with query parameters (e.g., filters).
///
/// # Arguments
///
/// * `client` - Shared HTTP client.
/// * `params` - Query parameters.
#[instrument(skip(client))]
pub async fn dashboard_index_with_params(
    client: &HttpClient,
    params: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, "/dashboard", Some(params))
        .await
}

/// Deletes all entries (mass delete).
///
/// Sends a POST request to `/dashboard/delete-all` with `_method=DELETE` injected.
///
/// # Arguments
///
/// * `client` - Shared HTTP client.
///
/// # Errors
///
/// Propagates network or HTTP errors.
#[instrument(skip(client))]
pub async fn delete_all(client: &HttpClient) -> Result<reqwest::Response, Error> {
    let mut data = HashMap::new();
    data.insert("_method".to_string(), "DELETE".to_string());
    client
        .request(HttpMethod::POST, "/dashboard/delete-all", Some(data))
        .await
}

/// Deletes entries by category column and value.
///
/// Sends a POST request to `/dashboard/cekunit/delete-by-category` with
/// `column` and `value` in the form data, plus `_method=DELETE`.
///
/// # Arguments
///
/// * `client` - Shared HTTP client.
/// * `column` - Name of the column to filter by.
/// * `value` - Value to match.
///
/// # Errors
///
/// Propagates network or HTTP errors.
#[instrument(skip(client))]
pub async fn delete_by_category(
    client: &HttpClient,
    column: &str,
    value: &str,
) -> Result<reqwest::Response, Error> {
    let mut data = HashMap::new();
    data.insert("column".to_string(), column.to_string());
    data.insert("value".to_string(), value.to_string());
    client
        .request(
            HttpMethod::POST,
            "/dashboard/cekunit/delete-by-category",
            Some(data),
        )
        .await
}

/// Exports data in the specified format with sorting options.
///
/// Sends a GET request to `/dashboard/cekunit/export` with query parameters
/// `format`, `sort`, and `direction`.
///
/// # Arguments
///
/// * `client` - Shared HTTP client.
/// * `format` - Export format (e.g., `"csv"`, `"json"`, `"xlsx"`).
/// * `sort` - Column name to sort by.
/// * `direction` - Sort direction (`"asc"` or `"desc"`).
///
/// # Errors
///
/// Propagates network or HTTP errors.
///
/// # Examples
///
/// ```no_run
/// # use librcekunit::{HttpClient, Config, api::dashboard};
/// # async fn run(client: &HttpClient) -> Result<(), Box<dyn std::error::Error>> {
/// let response = dashboard::export(client, "csv", "id", "asc").await?;
/// # Ok(())
/// # }
/// ```
#[instrument(skip(client))]
pub async fn export(
    client: &HttpClient,
    format: &str,
    sort: &str,
    direction: &str,
) -> Result<reqwest::Response, Error> {
    let mut params = HashMap::new();
    params.insert("format".to_string(), format.to_string());
    params.insert("sort".to_string(), sort.to_string());
    params.insert("direction".to_string(), direction.to_string());
    client
        .request(HttpMethod::GET, "/dashboard/cekunit/export", Some(params))
        .await
}

/// Retrieves unique values for a given column.
///
/// Sends a GET request to `/dashboard/cekunit/get-unique-values` with the
/// `column` query parameter.
///
/// # Arguments
///
/// * `client` - Shared HTTP client.
/// * `column` - Column name to get distinct values from.
///
/// # Errors
///
/// Propagates network or HTTP errors.
///
/// # Examples
///
/// ```no_run
/// # use librcekunit::{HttpClient, Config, api::dashboard};
/// # async fn run(client: &HttpClient) -> Result<(), Box<dyn std::error::Error>> {
/// let response = dashboard::get_unique_values(client, "status").await?;
/// # Ok(())
/// # }
/// ```
#[instrument(skip(client))]
pub async fn get_unique_values(
    client: &HttpClient,
    column: &str,
) -> Result<reqwest::Response, Error> {
    let mut params = HashMap::new();
    params.insert("column".to_string(), column.to_string());
    client
        .request(
            HttpMethod::GET,
            "/dashboard/cekunit/get-unique-values",
            Some(params),
        )
        .await
}
