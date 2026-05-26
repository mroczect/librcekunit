//! User management (standard users and dashboard users).
//!
//! This module provides functions to manage user resources, including
//! standard CRUD operations and dashboard‑specific views.
//!
//! # Example
//!
//! ```no_run
//! use librcekunit::{HttpClient, Config};
//! use librcekunit::api::users;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::new("https://example.com");
//! let client = HttpClient::new(&config).await?;
//!
//! let users = users::index(&client).await?;
//! let dashboard_users = users::dashboard_index(&client).await?;
//! # Ok(())
//! # }
//! ```

use crate::error::Error;
use crate::http_client::HttpClient;
use crate::types::HttpMethod;
use std::collections::HashMap;
use tracing::instrument;

/// Lists all users (standard index).
///
/// Sends a GET request to `/users`.
#[instrument(skip(client))]
pub async fn index(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::GET, "/users", None).await
}

/// Lists users with query parameters.
#[instrument(skip(client))]
pub async fn index_with_params(
    client: &HttpClient,
    params: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, "/users", Some(params))
        .await
}

/// Displays the user creation form.
///
/// Sends a GET request to `/users/create`.
#[instrument(skip(client))]
pub async fn create(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::GET, "/users/create", None).await
}

/// Stores a new user.
///
/// Sends a POST request to `/users` with form data.
#[instrument(skip(client))]
pub async fn store(
    client: &HttpClient,
    data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::POST, "/users", Some(data)).await
}

/// Shows a specific user by `nomor`.
#[instrument(skip(client))]
pub async fn show(client: &HttpClient, nomor: u64) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, &format!("/users/{}", nomor), None)
        .await
}

/// Displays the edit form for a user.
#[instrument(skip(client))]
pub async fn edit(client: &HttpClient, nomor: u64) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, &format!("/users/{}/edit", nomor), None)
        .await
}

/// Updates a user.
///
/// Sends a POST request to `/users/{nomor}` with `_method=PUT`.
#[instrument(skip(client))]
pub async fn update(
    client: &HttpClient,
    nomor: u64,
    mut data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    data.entry("_method".to_string())
        .or_insert_with(|| "PUT".to_string());
    client
        .request(HttpMethod::POST, &format!("/users/{}", nomor), Some(data))
        .await
}

/// Deletes a user.
///
/// Sends a POST request to `/users/{nomor}` with `_method=DELETE`.
#[instrument(skip(client))]
pub async fn destroy(client: &HttpClient, nomor: u64) -> Result<reqwest::Response, Error> {
    let mut data = HashMap::new();
    data.insert("_method".to_string(), "DELETE".to_string());
    client
        .request(HttpMethod::POST, &format!("/users/{}", nomor), Some(data))
        .await
}

/// Fetches the dashboard users index.
///
/// Sends a GET request to `/dashboard/users`.
#[instrument(skip(client))]
pub async fn dashboard_index(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, "/dashboard/users", None)
        .await
}

/// Fetches the dashboard users index with query parameters.
#[instrument(skip(client))]
pub async fn dashboard_index_with_params(
    client: &HttpClient,
    params: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, "/dashboard/users", Some(params))
        .await
}
