//! Dashboard and bulk operations.
//!
//! This module provides functions for interacting with the main dashboard view
//! and performing bulk actions such as mass deletion, data export, and
//! fetching distinct column values. All functions accept a shared reference to
//! [`HttpClient`] and return the raw [`reqwest::Response`] for maximum
//! flexibility.
//!
//! # Security
//!
//! - All mutating requests (delete, export) automatically include a CSRF
//!   token via [`HttpClient::request`].
//! - Input parameters are validated early – empty strings for required
//!   fields are rejected before any network request.
//!
//! # Example
//!
//! ```no_run
//! use librcekunit::http_client::HttpClient;
//! use librcekunit::Config;
//! use librcekunit::api::dashboard;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::new("https://example.com");
//! let client = HttpClient::new(&config).await?;
//!
//! // Fetch the main dashboard
//! let dash = dashboard::dashboard_index(&client).await?;
//!
//! // Get unique values of a column
//! let unique = dashboard::get_unique_values(&client, "category").await?;
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

/// Base path for the dashboard.
const PATH_DASHBOARD: &str = "/dashboard";
/// Path for mass‑deletion endpoint.
const PATH_DELETE_ALL: &str = "/dashboard/delete-all";
/// Path for delete‑by‑category endpoint.
const PATH_DELETE_BY_CATEGORY: &str = "/dashboard/cekunit/delete-by-category";
/// Path for export endpoint.
const PATH_EXPORT: &str = "/dashboard/cekunit/export";
/// Path for unique‑values endpoint.
const PATH_UNIQUE_VALUES: &str = "/dashboard/cekunit/get-unique-values";

// ---------------------------------------------------------------------------
// Public functions
// ---------------------------------------------------------------------------

/// Fetches the main dashboard page.
///
/// Sends a `GET` request to `/dashboard` without query parameters.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
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
/// # use librcekunit::api::dashboard;
/// # async fn run(client: &HttpClient) -> Result<(), Box<dyn std::error::Error>> {
/// let response = dashboard::dashboard_index(client).await?;
/// # Ok(())
/// # }
/// ```
#[instrument(skip(client))]
pub async fn dashboard_index(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::GET, PATH_DASHBOARD, None).await
}

/// Fetches the dashboard with query parameters.
///
/// Sends a `GET` request to `/dashboard?key1=val1&...` using the provided
/// `params` map. This is useful for server‑side filtering, pagination, or
/// search.
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
#[instrument(skip(client))]
pub async fn dashboard_index_with_params(
    client: &HttpClient,
    params: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, PATH_DASHBOARD, Some(params))
        .await
}

/// Deletes **all** entries (mass delete).
///
/// Sends a `POST` request to `/dashboard/delete-all` with `_method=DELETE`
/// injected into the form data. **This operation is irreversible** and
/// should be used with caution.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
///
/// # Errors
///
/// * [`Error::Reqwest`] – Network error, timeout, or invalid URL.
/// * [`Error::CsrfNotFound`] – CSRF token could not be obtained.
/// * [`Error::Api`] – The server returned an error (e.g., permission denied).
#[instrument(skip(client))]
pub async fn delete_all(client: &HttpClient) -> Result<reqwest::Response, Error> {
    let mut data = HashMap::with_capacity(1);
    data.insert("_method".to_string(), "DELETE".to_string());
    client
        .request(HttpMethod::POST, PATH_DELETE_ALL, Some(data))
        .await
}

/// Deletes entries by matching a column and a value.
///
/// Sends a `POST` request to `/dashboard/cekunit/delete-by-category` with
/// `column`, `value`, and the injected `_method=DELETE`. The server is
/// expected to delete all records where the given column matches the given
/// value.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `column` – Name of the column to filter by. Must not be empty or
///   whitespace‑only.
/// * `value` – Value to match. Must not be empty or whitespace‑only.
///
/// # Errors
///
/// * [`Error::Api`] – Returned immediately if `column` or `value` is empty.
/// * [`Error::Reqwest`] – Network error, timeout, or invalid URL.
/// * [`Error::CsrfNotFound`] – CSRF token could not be obtained.
/// * [`Error::Api`] – The server returned an error.
///
/// # Panics
///
/// This function is panic‑free.
#[instrument(skip(client), fields(column = %column, value = %value))]
pub async fn delete_by_category(
    client: &HttpClient,
    column: &str,
    value: &str,
) -> Result<reqwest::Response, Error> {
    let column = column.trim();
    let value = value.trim();

    if column.is_empty() {
        warn!("delete_by_category called with empty column");
        return Err(Error::Api(400, "Column name cannot be empty".into()));
    }
    if value.is_empty() {
        warn!("delete_by_category called with empty value");
        return Err(Error::Api(400, "Match value cannot be empty".into()));
    }

    let mut data = HashMap::with_capacity(3);
    data.insert("column".to_string(), column.to_string());
    data.insert("value".to_string(), value.to_string());
    // `_method` is injected automatically by HttpClient::request for non‑GET
    // requests, but some servers require it to be explicitly present.
    data.entry("_method".to_string())
        .or_insert_with(|| "DELETE".to_string());

    client
        .request(HttpMethod::POST, PATH_DELETE_BY_CATEGORY, Some(data))
        .await
}

/// Exports data in the specified format with sorting options.
///
/// Sends a `GET` request to `/dashboard/cekunit/export` with query
/// parameters `format`, `sort`, and `direction`.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `format` – Export format (e.g., `"csv"`, `"json"`, `"xlsx"`). Must not
///   be empty.
/// * `sort` – Column name to sort by. Must not be empty.
/// * `direction` – Sort direction (`"asc"` or `"desc"`). Must not be empty.
///
/// # Errors
///
/// * [`Error::Api`] – Returned immediately if any of the string arguments is
///   empty.
/// * [`Error::Reqwest`] – Network error, timeout, or invalid URL.
/// * [`Error::Api`] – The server returned an error.
///
/// # Examples
///
/// ```no_run
/// # use librcekunit::http_client::HttpClient;
/// # use librcekunit::api::dashboard;
/// # async fn run(client: &HttpClient) -> Result<(), Box<dyn std::error::Error>> {
/// let response = dashboard::export(client, "csv", "id", "asc").await?;
/// # Ok(())
/// # }
/// ```
#[instrument(skip(client), fields(format = %format, sort = %sort, direction = %direction))]
pub async fn export(
    client: &HttpClient,
    format: &str,
    sort: &str,
    direction: &str,
) -> Result<reqwest::Response, Error> {
    let format = format.trim();
    let sort = sort.trim();
    let direction = direction.trim();

    // Fail early if any required parameter is empty.
    if format.is_empty() {
        warn!("export called with empty format");
        return Err(Error::Api(400, "Export format cannot be empty".into()));
    }
    if sort.is_empty() {
        warn!("export called with empty sort column");
        return Err(Error::Api(400, "Sort column cannot be empty".into()));
    }
    if direction.is_empty() {
        warn!("export called with empty sort direction");
        return Err(Error::Api(400, "Sort direction cannot be empty".into()));
    }

    let mut params = HashMap::with_capacity(3);
    params.insert("format".to_string(), format.to_string());
    params.insert("sort".to_string(), sort.to_string());
    params.insert("direction".to_string(), direction.to_string());

    client
        .request(HttpMethod::GET, PATH_EXPORT, Some(params))
        .await
}

/// Retrieves unique values for a given column.
///
/// Sends a `GET` request to `/dashboard/cekunit/get-unique-values` with the
/// `column` query parameter. This can be used to populate filter dropdowns
/// or to inspect data distribution.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `column` – Column name to get distinct values from. Must not be empty.
///
/// # Errors
///
/// * [`Error::Api`] – Returned immediately if `column` is empty.
/// * [`Error::Reqwest`] – Network error, timeout, or invalid URL.
/// * [`Error::Api`] – The server returned an error.
///
/// # Examples
///
/// ```no_run
/// # use librcekunit::http_client::HttpClient;
/// # use librcekunit::api::dashboard;
/// # async fn run(client: &HttpClient) -> Result<(), Box<dyn std::error::Error>> {
/// let response = dashboard::get_unique_values(client, "status").await?;
/// # Ok(())
/// # }
/// ```
#[instrument(skip(client), fields(column = %column))]
pub async fn get_unique_values(
    client: &HttpClient,
    column: &str,
) -> Result<reqwest::Response, Error> {
    let column = column.trim();
    if column.is_empty() {
        warn!("get_unique_values called with empty column");
        return Err(Error::Api(400, "Column name cannot be empty".into()));
    }

    let mut params = HashMap::with_capacity(1);
    params.insert("column".to_string(), column.to_string());

    client
        .request(HttpMethod::GET, PATH_UNIQUE_VALUES, Some(params))
        .await
}
