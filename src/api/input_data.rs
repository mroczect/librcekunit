//! Input data management (create and store).
//!
//! This module provides functions to display the input data form and submit
//! new input data entries.
//!
//! # Example
//!
//! ```no_run
//! use librcekunit::{HttpClient, Config};
//! use librcekunit::api::input_data;
//! use std::collections::HashMap;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::new("https://example.com");
//! let client = HttpClient::new(&config).await?;
//!
//! // Show creation form
//! let form = input_data::create(&client).await?;
//!
//! // Submit new data
//! let mut data = HashMap::new();
//! data.insert("value".to_string(), "42".to_string());
//! let response = input_data::store(&client, data).await?;
//! # Ok(())
//! # }
//! ```

use crate::error::Error;
use crate::http_client::HttpClient;
use crate::types::HttpMethod;
use std::collections::HashMap;
use tracing::instrument;

/// Displays the input data creation form.
///
/// Sends a GET request to `/dashboard/input-data`. The response is typically
/// an HTML form for entering new input data.
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
        .request(HttpMethod::GET, "/dashboard/input-data", None)
        .await
}

/// Stores new input data.
///
/// Sends a POST request to `/dashboard/input-data` with the provided form data.
/// CSRF token is automatically injected.
///
/// # Arguments
///
/// * `client` - Shared HTTP client.
/// * `data` - Form data to submit.
///
/// # Errors
///
/// Propagates network or HTTP errors, including CSRF token fetch failures.
///
/// # Examples
///
/// ```no_run
/// # use librcekunit::{HttpClient, Config, api::input_data};
/// # use std::collections::HashMap;
/// # async fn run(client: &HttpClient) -> Result<(), Box<dyn std::error::Error>> {
/// let mut data = HashMap::new();
/// data.insert("name".to_string(), "Sample".to_string());
/// data.insert("value".to_string(), "100".to_string());
/// let response = input_data::store(client, data).await?;
/// # Ok(())
/// # }
/// ```
#[instrument(skip(client))]
pub async fn store(
    client: &HttpClient,
    data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::POST, "/dashboard/input-data", Some(data))
        .await
}
