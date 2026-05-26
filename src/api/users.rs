//! User management (standard users and dashboard users).
//!
//! This module provides functions for managing user resources, including
//! standard CRUD operations and dashboard‑specific views. All functions
//! operate through a shared [`HttpClient`] and return the raw
//! [`reqwest::Response`] for maximum flexibility.
//!
//! # Security
//!
//! * Mutating operations (`store`, `update`, `destroy`) automatically
//!   inject a CSRF token via [`HttpClient::request`].
//! * User identifiers (`nomor`) are validated to be **greater than 0**
//!   before any network request is made.
//!
//! # Example
//!
//! ```no_run
//! use librcekunit::http_client::HttpClient;
//! use librcekunit::Config;
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
use tracing::{instrument, warn};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Base path for user resources.
const PATH_USERS: &str = "/users";
/// Sub‑path for the creation form.
const PATH_CREATE: &str = "/create";
/// Sub‑path for the edit form.
const PATH_EDIT: &str = "/edit";
/// Path for the dashboard users index.
const PATH_DASHBOARD_USERS: &str = "/dashboard/users";

// ---------------------------------------------------------------------------
// Public functions
// ---------------------------------------------------------------------------

/// Lists all users (standard index).
///
/// Sends a `GET` request to `/users`.
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
    client.request(HttpMethod::GET, PATH_USERS, None).await
}

/// Lists users with query parameters.
///
/// Sends a `GET` request to `/users?key1=val1&...` using the given `params`.
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
        .request(HttpMethod::GET, PATH_USERS, Some(params))
        .await
}

/// Displays the user creation form.
///
/// Sends a `GET` request to `/users/create`. The server typically returns an
/// HTML form. This can be used to inspect required fields or to obtain a
/// fresh CSRF token.
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
            &format!("{}{}", PATH_USERS, PATH_CREATE),
            None,
        )
        .await
}

/// Stores a new user.
///
/// Sends a `POST` request to `/users` with the provided form data. A CSRF
/// token is **automatically injected** by [`HttpClient::request`] if not
/// already present.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `data` – Form data for the new user.
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
    client
        .request(HttpMethod::POST, PATH_USERS, Some(data))
        .await
}

/// Shows a specific user by its numeric identifier `nomor`.
///
/// Sends a `GET` request to `/users/{nomor}`.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `nomor` – The user ID. Must be **greater than 0**.
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
        warn!("Attempted to show user with invalid nomor 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    client
        .request(HttpMethod::GET, &format!("{}/{}", PATH_USERS, nomor), None)
        .await
}

/// Displays the edit form for a user.
///
/// Sends a `GET` request to `/users/{nomor}/edit`. The response typically
/// contains an HTML form pre‑filled with the current user data.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `nomor` – The user ID. Must be greater than 0.
///
/// # Errors
///
/// * [`Error::Api`] – Returned if `nomor == 0`.
/// * [`Error::Reqwest`] – Network error.
/// * [`Error::Api`] – Server returned an error.
#[instrument(skip(client))]
pub async fn edit(client: &HttpClient, nomor: u64) -> Result<reqwest::Response, Error> {
    if nomor == 0 {
        warn!("Attempted to edit user with invalid nomor 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    client
        .request(
            HttpMethod::GET,
            &format!("{}/{}{}", PATH_USERS, nomor, PATH_EDIT),
            None,
        )
        .await
}

/// Updates a user.
///
/// Sends a `POST` request to `/users/{nomor}` with form data and an injected
/// `_method=PUT` field to emulate HTTP PUT. CSRF token is added
/// automatically.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `nomor` – The user ID. Must be greater than 0.
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
        warn!("Attempted to update user with invalid nomor 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    data.entry("_method".to_string())
        .or_insert_with(|| "PUT".to_string());
    client
        .request(
            HttpMethod::POST,
            &format!("{}/{}", PATH_USERS, nomor),
            Some(data),
        )
        .await
}

/// Deletes a user.
///
/// Sends a `POST` request to `/users/{nomor}` with `_method=DELETE` in the
/// form data to emulate HTTP DELETE. CSRF token is added automatically.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `nomor` – The user ID to delete. Must be greater than 0.
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
        warn!("Attempted to destroy user with invalid nomor 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    let mut data = HashMap::with_capacity(1);
    data.insert("_method".to_string(), "DELETE".to_string());
    client
        .request(
            HttpMethod::POST,
            &format!("{}/{}", PATH_USERS, nomor),
            Some(data),
        )
        .await
}

/// Fetches the dashboard users index.
///
/// Sends a `GET` request to `/dashboard/users`. This endpoint typically
/// provides a summary or administrative view of users.
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
        .request(HttpMethod::GET, PATH_DASHBOARD_USERS, None)
        .await
}

/// Fetches the dashboard users index with query parameters.
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
        .request(HttpMethod::GET, PATH_DASHBOARD_USERS, Some(params))
        .await
}
