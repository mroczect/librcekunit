//! Input user management (full CRUD + export/import).
//!
//! This module provides functions to manage "input user" resources,
//! including listing, creating, reading, updating, deleting, exporting,
//! and importing data.
//!
//! # Example
//!
//! ```no_run
//! use librcekunit::{HttpClient, Config};
//! use librcekunit::api::input_user;
//! use std::collections::HashMap;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::new("https://example.com");
//! let client = HttpClient::new(&config).await?;
//!
//! // List all input users
//! let resp = input_user::index(&client).await?;
//!
//! // Create a new input user
//! let mut data = HashMap::new();
//! data.insert("name".to_string(), "John".to_string());
//! let resp = input_user::store(&client, data).await?;
//! # Ok(())
//! # }
//! ```

use crate::error::Error;
use crate::http_client::HttpClient;
use crate::types::HttpMethod;
use std::collections::HashMap;
use tracing::instrument;

/// Retrieves all input user entries (index).
///
/// Sends a GET request to `/dashboard/input-user`. Returns the raw HTTP response.
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
/// # use librcekunit::{HttpClient, Config, api::input_user};
/// # async fn run(client: &HttpClient) -> Result<(), Box<dyn std::error::Error>> {
/// let response = input_user::index(client).await?;
/// # Ok(())
/// # }
/// ```
#[instrument(skip(client))]
pub async fn index(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, "/dashboard/input-user", None)
        .await
}

/// Retrieves input user entries with query parameters.
///
/// Sends a GET request to `/dashboard/input-user` with the given query parameters.
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
#[instrument(skip(client))]
pub async fn index_with_params(
    client: &HttpClient,
    params: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, "/dashboard/input-user", Some(params))
        .await
}

/// Displays the form to create a new input user.
///
/// Sends a GET request to `/dashboard/input_user/create`.
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
        .request(HttpMethod::GET, "/dashboard/input_user/create", None)
        .await
}

/// Stores a new input user (create).
///
/// Sends a POST request to `/dashboard/input_user` with form data.
/// CSRF token is automatically injected.
///
/// # Arguments
///
/// * `client` - Shared HTTP client.
/// * `data` - Form data for the new resource.
///
/// # Errors
///
/// Propagates errors including CSRF token fetch failures.
#[instrument(skip(client))]
pub async fn store(
    client: &HttpClient,
    data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::POST, "/dashboard/input_user", Some(data))
        .await
}

/// Shows a specific input user by ID.
///
/// Sends a GET request to `/dashboard/input_user/{id}`.
///
/// # Arguments
///
/// * `client` - Shared HTTP client.
/// * `id` - The user ID.
///
/// # Errors
///
/// Propagates network or HTTP errors. May return 404 if not found.
#[instrument(skip(client))]
pub async fn show(client: &HttpClient, id: u64) -> Result<reqwest::Response, Error> {
    client
        .request(
            HttpMethod::GET,
            &format!("/dashboard/input_user/{}", id),
            None,
        )
        .await
}

/// Displays the edit form for an input user.
///
/// Sends a GET request to `/dashboard/input_user/{id}/edit`.
///
/// # Arguments
///
/// * `client` - Shared HTTP client.
/// * `id` - User ID.
///
/// # Errors
///
/// Propagates network or HTTP errors.
#[instrument(skip(client))]
pub async fn edit(client: &HttpClient, id: u64) -> Result<reqwest::Response, Error> {
    client
        .request(
            HttpMethod::GET,
            &format!("/dashboard/input_user/{}/edit", id),
            None,
        )
        .await
}

/// Updates an input user.
///
/// Sends a POST request to `/dashboard/input_user/{id}` with `_method=PUT` injected.
/// CSRF token is added automatically.
///
/// # Arguments
///
/// * `client` - Shared HTTP client.
/// * `id` - User ID.
/// * `data` - Updated form data.
///
/// # Errors
///
/// Propagates network or HTTP errors.
#[instrument(skip(client))]
pub async fn update(
    client: &HttpClient,
    id: u64,
    mut data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    data.entry("_method".to_string())
        .or_insert_with(|| "PUT".to_string());
    client
        .request(
            HttpMethod::POST,
            &format!("/dashboard/input_user/{}", id),
            Some(data),
        )
        .await
}

/// Deletes an input user.
///
/// Sends a POST request to `/dashboard/input_user/{id}` with `_method=DELETE`.
///
/// # Arguments
///
/// * `client` - Shared HTTP client.
/// * `id` - User ID to delete.
///
/// # Errors
///
/// Propagates network or HTTP errors.
#[instrument(skip(client))]
pub async fn destroy(client: &HttpClient, id: u64) -> Result<reqwest::Response, Error> {
    let mut data = HashMap::new();
    data.insert("_method".to_string(), "DELETE".to_string());
    client
        .request(
            HttpMethod::POST,
            &format!("/dashboard/input_user/{}", id),
            Some(data),
        )
        .await
}

/// Exports input user data with query parameters.
///
/// Sends a GET request to `/dashboard/input_user/export`.
/// The `params` can include format, sorting, filters, etc.
///
/// # Arguments
///
/// * `client` - Shared HTTP client.
/// * `params` - Export options as query parameters.
///
/// # Errors
///
/// Propagates network or HTTP errors.
#[instrument(skip(client))]
pub async fn export(
    client: &HttpClient,
    params: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(
            HttpMethod::GET,
            "/dashboard/input_user/export",
            Some(params),
        )
        .await
}

/// Imports input user data.
///
/// Sends a POST request to `/dashboard/input_user/insert` with form data.
///
/// # Arguments
///
/// * `client` - Shared HTTP client.
/// * `data` - Import data as form fields.
///
/// # Errors
///
/// Propagates network or HTTP errors.
#[instrument(skip(client))]
pub async fn import(
    client: &HttpClient,
    data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::POST, "/dashboard/input_user/insert", Some(data))
        .await
}
