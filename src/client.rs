//! Main client module providing high‑level API access.
//!
//! This module exports the `Client` struct, which wraps an `HttpClient` and
//! exposes convenient methods for every endpoint of the target web application.
//! All methods delegate to the corresponding functions in the `api` submodules.
//!
//! # Examples
//!
//! ```no_run
//! use librcekunit::{Client, Config};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::from_env()?;
//! let client = Client::new(config).await?;
//!
//! // Login
//! client.login("user@example.com", "password").await?;
//!
//! // Fetch dashboard
//! let dashboard = client.dashboard_index().await?;
//! # Ok(())
//! # }
//! ```

use crate::config::Config;
use crate::error::Error;
use crate::http_client::HttpClient;
use crate::types::HttpMethod;
use std::collections::HashMap;
use tracing::instrument;

/// Main client for all API interactions.
///
/// `Client` holds an `HttpClient` instance and provides one method per
/// endpoint. It handles authentication, CSRF, cookies, and request building
/// automatically.
///
/// # Example
///
/// ```no_run
/// # use librcekunit::{Client, Config};
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// let config = Config::new("https://example.com");
/// let client = Client::new(config).await?;
///
/// // Use any endpoint method
/// let response = client.cekunit_index().await?;
/// # Ok(())
/// # }
/// ```
pub struct Client {
    http: HttpClient,
}

impl Client {
    /// Creates a new `Client` from a configuration.
    ///
    /// This constructs an `HttpClient` internally using the provided `Config`.
    ///
    /// # Arguments
    ///
    /// * `config` – Configuration object (base URL, timeout, user agent, cookie store).
    ///
    /// # Errors
    ///
    /// Propagates any error from `HttpClient::new`, such as network errors
    /// or configuration issues.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use librcekunit::{Client, Config};
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = Config::new("https://api.example.com");
    /// let client = Client::new(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument]
    pub async fn new(config: Config) -> Result<Self, Error> {
        let http = HttpClient::new(&config).await?;
        Ok(Self { http })
    }

    /// Authenticates a user with email and password.
    ///
    /// This method fetches a CSRF token, submits the login form, and stores
    /// session cookies automatically.
    ///
    /// # Arguments
    ///
    /// * `email` – User's email address.
    /// * `password` – User's password.
    ///
    /// # Errors
    ///
    /// Returns `Error::Auth` if credentials are invalid.
    /// Returns `Error::Api` or `Error::Reqwest` for other failures.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use librcekunit::Client;
    /// # async fn run(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    /// client.login("admin@example.com", "secret").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn login(&self, email: &str, password: &str) -> Result<(), Error> {
        crate::api::auth::login::login(&self.http, email, password).await
    }

    /// Logs out the current user.
    ///
    /// Sends a POST request to `/logout`, clears the CSRF token, and removes
    /// the persistent cookie file if it exists.
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` if the logout request fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use librcekunit::Client;
    /// # async fn run(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    /// client.logout().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn logout(&self) -> Result<(), Error> {
        crate::api::auth::logout::logout(&self.http).await
    }

    /// Returns a reference to the inner `HttpClient`.
    ///
    /// This can be used for low‑level access or to inspect cookies.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use librcekunit::Client;
    /// # async fn run(client: &Client) {
    /// let http = client.http();
    /// let base = http.base_url();
    /// # }
    /// ```
    pub fn http(&self) -> &HttpClient {
        &self.http
    }

    /// Sends a generic HTTP request.
    ///
    /// This method is a passthrough to `HttpClient::request`. It is useful
    /// for endpoints that are not explicitly defined as methods.
    ///
    /// # Arguments
    ///
    /// * `method` – HTTP method.
    /// * `path` – Path relative to the base URL.
    /// * `body` – Optional query parameters (for GET/HEAD) or form data (for others).
    ///
    /// # Errors
    ///
    /// Propagates errors from the underlying `HttpClient`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use librcekunit::{Client, HttpMethod};
    /// # use std::collections::HashMap;
    /// # async fn run(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    /// let mut params = HashMap::new();
    /// params.insert("id".to_string(), "42".to_string());
    /// let resp = client.request(HttpMethod::GET, "/custom", Some(params)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn request(
        &self,
        method: HttpMethod,
        path: &str,
        body: Option<HashMap<String, String>>,
    ) -> Result<reqwest::Response, Error> {
        self.http.request(method, path, body).await
    }

    // ------------------------------------------------------------------------
    // Dashboard endpoints
    // ------------------------------------------------------------------------

    /// Fetches the main dashboard page.
    ///
    /// Sends a GET request to `/dashboard`.
    ///
    /// # Errors
    ///
    /// Returns any network or HTTP error.
    pub async fn dashboard_index(&self) -> Result<reqwest::Response, Error> {
        crate::api::dashboard::dashboard_index(&self.http).await
    }

    /// Fetches the dashboard with query parameters.
    ///
    /// # Arguments
    ///
    /// * `params` – Query parameters as key‑value pairs.
    pub async fn dashboard_index_with_params(
        &self,
        params: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::dashboard::dashboard_index_with_params(&self.http, params).await
    }

    // ------------------------------------------------------------------------
    // Cekunit (CRUD) endpoints
    // ------------------------------------------------------------------------

    /// Lists all `cekunit` entries.
    pub async fn cekunit_index(&self) -> Result<reqwest::Response, Error> {
        crate::api::crud::index(&self.http).await
    }

    /// Lists `cekunit` entries with query parameters.
    pub async fn cekunit_index_with_params(
        &self,
        params: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::crud::index_with_params(&self.http, params).await
    }

    /// Displays the form to create a new `cekunit`.
    pub async fn cekunit_create(&self) -> Result<reqwest::Response, Error> {
        crate::api::crud::create(&self.http).await
    }

    /// Stores a new `cekunit`.
    ///
    /// # Arguments
    ///
    /// * `data` – Form data to submit.
    pub async fn cekunit_store(
        &self,
        data: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::crud::store(&self.http, data).await
    }

    /// Shows a specific `cekunit` by its number `no`.
    ///
    /// # Arguments
    ///
    /// * `no` – The record number.
    pub async fn cekunit_show(&self, no: u64) -> Result<reqwest::Response, Error> {
        crate::api::crud::show(&self.http, no).await
    }

    /// Displays the edit form for a `cekunit`.
    pub async fn cekunit_edit(&self, no: u64) -> Result<reqwest::Response, Error> {
        crate::api::crud::edit(&self.http, no).await
    }

    /// Updates a `cekunit`.
    ///
    /// # Arguments
    ///
    /// * `no` – Record number.
    /// * `data` – Updated form data.
    pub async fn cekunit_update(
        &self,
        no: u64,
        data: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::crud::update(&self.http, no, data).await
    }

    /// Deletes a `cekunit`.
    pub async fn cekunit_destroy(&self, no: u64) -> Result<reqwest::Response, Error> {
        crate::api::crud::destroy(&self.http, no).await
    }

    // ------------------------------------------------------------------------
    // Additional dashboard actions
    // ------------------------------------------------------------------------

    /// Deletes all entries (mass delete).
    pub async fn dashboard_delete_all(&self) -> Result<reqwest::Response, Error> {
        crate::api::dashboard::delete_all(&self.http).await
    }

    /// Deletes entries by a category column and value.
    ///
    /// # Arguments
    ///
    /// * `column` – Column name to match.
    /// * `value` – Value to match.
    pub async fn cekunit_delete_by_category(
        &self,
        column: &str,
        value: &str,
    ) -> Result<reqwest::Response, Error> {
        crate::api::dashboard::delete_by_category(&self.http, column, value).await
    }

    /// Exports data in the specified format.
    ///
    /// # Arguments
    ///
    /// * `format` – Export format (e.g., "csv", "json").
    /// * `sort` – Column to sort by.
    /// * `direction` – Sort direction ("asc" or "desc").
    pub async fn cekunit_export(
        &self,
        format: &str,
        sort: &str,
        direction: &str,
    ) -> Result<reqwest::Response, Error> {
        crate::api::dashboard::export(&self.http, format, sort, direction).await
    }

    /// Retrieves unique values for a given column.
    ///
    /// # Arguments
    ///
    /// * `column` – Column name.
    pub async fn cekunit_get_unique_values(
        &self,
        column: &str,
    ) -> Result<reqwest::Response, Error> {
        crate::api::dashboard::get_unique_values(&self.http, column).await
    }

    // ------------------------------------------------------------------------
    // Input User endpoints
    // ------------------------------------------------------------------------

    pub async fn input_user_index(&self) -> Result<reqwest::Response, Error> {
        crate::api::input_user::index(&self.http).await
    }

    pub async fn input_user_index_with_params(
        &self,
        params: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::input_user::index_with_params(&self.http, params).await
    }

    pub async fn input_user_create(&self) -> Result<reqwest::Response, Error> {
        crate::api::input_user::create(&self.http).await
    }

    pub async fn input_user_store(
        &self,
        data: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::input_user::store(&self.http, data).await
    }

    pub async fn input_user_show(&self, id: u64) -> Result<reqwest::Response, Error> {
        crate::api::input_user::show(&self.http, id).await
    }

    pub async fn input_user_edit(&self, id: u64) -> Result<reqwest::Response, Error> {
        crate::api::input_user::edit(&self.http, id).await
    }

    pub async fn input_user_update(
        &self,
        id: u64,
        data: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::input_user::update(&self.http, id, data).await
    }

    pub async fn input_user_destroy(&self, id: u64) -> Result<reqwest::Response, Error> {
        crate::api::input_user::destroy(&self.http, id).await
    }

    pub async fn input_user_export(
        &self,
        params: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::input_user::export(&self.http, params).await
    }

    pub async fn input_user_import(
        &self,
        data: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::input_user::import(&self.http, data).await
    }

    // ------------------------------------------------------------------------
    // Input Data endpoints
    // ------------------------------------------------------------------------

    pub async fn input_data_create(&self) -> Result<reqwest::Response, Error> {
        crate::api::input_data::create(&self.http).await
    }

    pub async fn input_data_store(
        &self,
        data: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::input_data::store(&self.http, data).await
    }

    // ------------------------------------------------------------------------
    // PIC (Person In Charge) endpoints
    // ------------------------------------------------------------------------

    pub async fn pic_index(&self) -> Result<reqwest::Response, Error> {
        crate::api::pic::index(&self.http).await
    }

    pub async fn pic_index_with_params(
        &self,
        params: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::pic::index_with_params(&self.http, params).await
    }

    pub async fn pic_create(&self) -> Result<reqwest::Response, Error> {
        crate::api::pic::create(&self.http).await
    }

    pub async fn pic_store(
        &self,
        data: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::pic::store(&self.http, data).await
    }

    pub async fn pic_show(&self, nomor: u64) -> Result<reqwest::Response, Error> {
        crate::api::pic::show(&self.http, nomor).await
    }

    pub async fn pic_edit(&self, nomor: u64) -> Result<reqwest::Response, Error> {
        crate::api::pic::edit(&self.http, nomor).await
    }

    pub async fn pic_update(
        &self,
        nomor: u64,
        data: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::pic::update(&self.http, nomor, data).await
    }

    pub async fn pic_destroy(&self, nomor: u64) -> Result<reqwest::Response, Error> {
        crate::api::pic::destroy(&self.http, nomor).await
    }

    pub async fn dashboard_pic_index(&self) -> Result<reqwest::Response, Error> {
        crate::api::pic::dashboard_index(&self.http).await
    }

    pub async fn dashboard_pic_index_with_params(
        &self,
        params: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::pic::dashboard_index_with_params(&self.http, params).await
    }

    pub async fn input_pic_create(&self) -> Result<reqwest::Response, Error> {
        crate::api::pic::input_create(&self.http).await
    }

    pub async fn input_pic_store(
        &self,
        data: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::pic::input_store(&self.http, data).await
    }

    // ------------------------------------------------------------------------
    // Users endpoints
    // ------------------------------------------------------------------------

    pub async fn users_index(&self) -> Result<reqwest::Response, Error> {
        crate::api::users::index(&self.http).await
    }

    pub async fn users_index_with_params(
        &self,
        params: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::users::index_with_params(&self.http, params).await
    }

    pub async fn users_create(&self) -> Result<reqwest::Response, Error> {
        crate::api::users::create(&self.http).await
    }

    pub async fn users_store(
        &self,
        data: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::users::store(&self.http, data).await
    }

    pub async fn users_show(&self, nomor: u64) -> Result<reqwest::Response, Error> {
        crate::api::users::show(&self.http, nomor).await
    }

    pub async fn users_edit(&self, nomor: u64) -> Result<reqwest::Response, Error> {
        crate::api::users::edit(&self.http, nomor).await
    }

    pub async fn users_update(
        &self,
        nomor: u64,
        data: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::users::update(&self.http, nomor, data).await
    }

    pub async fn users_destroy(&self, nomor: u64) -> Result<reqwest::Response, Error> {
        crate::api::users::destroy(&self.http, nomor).await
    }

    pub async fn dashboard_users_index(&self) -> Result<reqwest::Response, Error> {
        crate::api::users::dashboard_index(&self.http).await
    }

    pub async fn dashboard_users_index_with_params(
        &self,
        params: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::users::dashboard_index_with_params(&self.http, params).await
    }
}
