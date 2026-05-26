//! Input user management (full CRUD + export/import).
//!
//! This module provides a complete set of functions to manage "input user"
//! resources: list, create, read, update, delete, export, and import. It
//! integrates tightly with [`HttpClient`] for request building, CSRF token
//! injection, and cookie persistence.
//!
//! # Usage
//!
//! All functions return the raw [`reqwest::Response`], giving callers full
//! control over status code inspection and body parsing.
//!
//! # Security
//!
//! * Mutating endpoints (`store`, `update`, `destroy`, `import`) automatically
//!   include a CSRF token.
//! * Resource IDs are validated to be greater than zero to prevent accidental
//!   requests with invalid identifiers.
//!
//! # Example
//!
//! ```no_run
//! use librcekunit::http_client::HttpClient;
//! use librcekunit::Config;
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
use tracing::{instrument, warn};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Base path for listing input users (note the dash vs. underscore difference
/// from other endpoints – this is intentional and matches the server routes).
const PATH_INDEX: &str = "/dashboard/input-user";
/// Base path for CRUD operations on individual input users.
const PATH_RESOURCE: &str = "/dashboard/input_user";
/// Sub‑path for the creation form.
const PATH_CREATE: &str = "/create";
/// Sub‑path for the edit form.
const PATH_EDIT: &str = "/edit";
/// Path for exporting input users.
const PATH_EXPORT: &str = "/dashboard/input_user/export";
/// Path for importing input users.
const PATH_IMPORT: &str = "/dashboard/input_user/insert";

// ---------------------------------------------------------------------------
// Public functions
// ---------------------------------------------------------------------------

/// Retrieves all input user entries (index).
///
/// Sends a `GET` request to `/dashboard/input-user` without parameters.
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
/// # use librcekunit::api::input_user;
/// # async fn run(client: &HttpClient) -> Result<(), Box<dyn std::error::Error>> {
/// let response = input_user::index(client).await?;
/// # Ok(())
/// # }
/// ```
#[instrument(skip(client))]
pub async fn index(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::GET, PATH_INDEX, None).await
}

/// Retrieves input user entries with query parameters.
///
/// Sends a `GET` request to `/dashboard/input-user?key1=val1&...` using the
/// provided `params` map. Useful for filtering, sorting, or pagination.
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
        .request(HttpMethod::GET, PATH_INDEX, Some(params))
        .await
}

/// Displays the form to create a new input user.
///
/// Sends a `GET` request to `/dashboard/input_user/create`. The response
/// typically contains an HTML form. This can be used to inspect available
/// fields or to obtain a fresh CSRF token.
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
            &format!("{}{}", PATH_RESOURCE, PATH_CREATE),
            None,
        )
        .await
}

/// Stores a new input user.
///
/// Sends a `POST` request to `/dashboard/input_user` with the provided form
/// data. CSRF token is **automatically injected** if not already present.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `data` – Form data for the new resource. Must contain the fields
///   expected by the server.
///
/// # Errors
///
/// * [`Error::Reqwest`] – Network error, timeout, or invalid URL.
/// * [`Error::Api`] – The server returned an error (e.g., validation).
/// * [`Error::CsrfNotFound`] – CSRF token could not be obtained.
#[instrument(skip(client))]
pub async fn store(
    client: &HttpClient,
    data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::POST, PATH_RESOURCE, Some(data))
        .await
}

/// Shows a specific input user by its numeric ID.
///
/// Sends a `GET` request to `/dashboard/input_user/{id}`.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `id` – The user ID. Must be greater than 0.
///
/// # Errors
///
/// * [`Error::Api`] – Returned immediately if `id == 0`.
/// * [`Error::Reqwest`] – Network error, timeout, or invalid URL.
/// * [`Error::Api`] – The server returned a non‑success status (e.g., 404 if
///   not found).
///
/// # Panics
///
/// This function is panic‑free.
#[instrument(skip(client))]
pub async fn show(client: &HttpClient, id: u64) -> Result<reqwest::Response, Error> {
    if id == 0 {
        warn!("Attempted to show input user with invalid ID 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    client
        .request(HttpMethod::GET, &format!("{}/{}", PATH_RESOURCE, id), None)
        .await
}

/// Displays the edit form for an input user.
///
/// Sends a `GET` request to `/dashboard/input_user/{id}/edit`. The response
/// typically contains an HTML form pre‑filled with current values.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `id` – The user ID. Must be greater than 0.
///
/// # Errors
///
/// * [`Error::Api`] – Returned if `id == 0`.
/// * [`Error::Reqwest`] – Network error.
/// * [`Error::Api`] – Server returned an error.
#[instrument(skip(client))]
pub async fn edit(client: &HttpClient, id: u64) -> Result<reqwest::Response, Error> {
    if id == 0 {
        warn!("Attempted to edit input user with invalid ID 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    client
        .request(
            HttpMethod::GET,
            &format!("{}/{}{}", PATH_RESOURCE, id, PATH_EDIT),
            None,
        )
        .await
}

/// Updates an input user.
///
/// Sends a `POST` request to `/dashboard/input_user/{id}` with form data and
/// an injected `_method=PUT` to emulate HTTP PUT. CSRF token is added
/// automatically.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `id` – The user ID. Must be greater than 0.
/// * `data` – Updated form data. The map is consumed and a `_method` key is
///   inserted if not already present.
///
/// # Errors
///
/// * [`Error::Api`] – Returned if `id == 0`.
/// * [`Error::Reqwest`] – Network error.
/// * [`Error::CsrfNotFound`] – CSRF token could not be obtained.
/// * [`Error::Api`] – Server returned an error (e.g., validation).
#[instrument(skip(client))]
pub async fn update(
    client: &HttpClient,
    id: u64,
    mut data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    if id == 0 {
        warn!("Attempted to update input user with invalid ID 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    data.entry("_method".to_string())
        .or_insert_with(|| "PUT".to_string());
    client
        .request(
            HttpMethod::POST,
            &format!("{}/{}", PATH_RESOURCE, id),
            Some(data),
        )
        .await
}

/// Deletes an input user.
///
/// Sends a `POST` request to `/dashboard/input_user/{id}` with
/// `_method=DELETE` in the form data to emulate HTTP DELETE. CSRF token is
/// added automatically.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `id` – The user ID to delete. Must be greater than 0.
///
/// # Errors
///
/// * [`Error::Api`] – Returned if `id == 0`.
/// * [`Error::Reqwest`] – Network error.
/// * [`Error::CsrfNotFound`] – CSRF token could not be obtained.
/// * [`Error::Api`] – Server returned an error.
#[instrument(skip(client))]
pub async fn destroy(client: &HttpClient, id: u64) -> Result<reqwest::Response, Error> {
    if id == 0 {
        warn!("Attempted to destroy input user with invalid ID 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    let mut data = HashMap::with_capacity(1);
    data.insert("_method".to_string(), "DELETE".to_string());
    client
        .request(
            HttpMethod::POST,
            &format!("{}/{}", PATH_RESOURCE, id),
            Some(data),
        )
        .await
}

/// Exports input user data with query parameters.
///
/// Sends a `GET` request to `/dashboard/input_user/export` with the given
/// `params` as query string. The server is expected to return a file download
/// or a data stream.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `params` – Export options (format, sorting, filters, etc.).
///
/// # Errors
///
/// * [`Error::Reqwest`] – Network error.
/// * [`Error::Api`] – Server returned an error.
#[instrument(skip(client))]
pub async fn export(
    client: &HttpClient,
    params: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, PATH_EXPORT, Some(params))
        .await
}

/// Imports input user data.
///
/// Sends a `POST` request to `/dashboard/input_user/insert` with the provided
/// form data. CSRF token is automatically included.
///
/// # Arguments
///
/// * `client` – Reference to the shared [`HttpClient`].
/// * `data` – Form fields representing the import data.
///
/// # Errors
///
/// * [`Error::Reqwest`] – Network error.
/// * [`Error::CsrfNotFound`] – CSRF token could not be obtained.
/// * [`Error::Api`] – Server returned an error (e.g., validation).
#[instrument(skip(client))]
pub async fn import(
    client: &HttpClient,
    data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::POST, PATH_IMPORT, Some(data))
        .await
}
