//! HTTP client with cookie persistence and CSRF token management.
//!
//! This module provides `HttpClient`, a wrapper around `reqwest::Client` that adds:
//! * Automatic cookie saving/loading to/from a file
//! * CSRF token extraction and injection for forms
//! * Request tracing via `tracing`
//! * Builder‑style configuration via `Config`
//!
//! # Examples
//!
//! ```no_run
//! use librcekunit::{Config, HttpClient};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::new("https://example.com");
//! let client = HttpClient::new(&config).await?;
//!
//! // Fetch a page and extract CSRF token
//! let token = client.fetch_csrf_token("/login").await?;
//!
//! // Send a POST request with form data (CSRF token auto‑injected)
//! let response = client.request(
//!     librcekunit::HttpMethod::POST,
//!     "/submit",
//!     Some([("field", "value")].into_iter().map(|(k,v)| (k.to_string(), v.to_string())).collect())
//! ).await?;
//! # Ok(())
//! # }
//! ```

use crate::config::CookieStore;
use crate::cookies;
use crate::error::Error;
use crate::types::{HttpMethod, join_url};
use reqwest::Client as ReqwestClient;
use reqwest::cookie::Jar;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, instrument, warn};

/// HTTP client with automatic CSRF token handling and cookie persistence.
///
/// `HttpClient` wraps a `reqwest::Client` and adds:
/// * A cookie jar (`Arc<Jar>`) that can be persisted to disk.
/// * An optional CSRF token stored in an `RwLock`.
/// * Automatic token injection for non‑GET/HEAD requests.
/// * Cookie saving after each request when persistent storage is configured.
///
/// # Fields (private)
/// * `client` – The underlying reqwest client.
/// * `base_url` – Base URL for all requests.
/// * `cookie_jar` – Shared cookie jar.
/// * `csrf_token` – Optionally stored CSRF token.
/// * `cookie_file` – Path to cookie file if persistence is enabled.
///
/// # Examples
///
/// ```no_run
/// # use librcekunit::{Config, HttpClient};
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// let config = Config::new("https://example.com");
/// let client = HttpClient::new(&config).await?;
/// assert_eq!(client.base_url(), "https://example.com");
/// # Ok(())
/// # }
/// ```
pub struct HttpClient {
    client: ReqwestClient,
    base_url: String,
    cookie_jar: Arc<Jar>,
    csrf_token: RwLock<Option<String>>,
    cookie_file: Option<PathBuf>,
}

impl HttpClient {
    /// Creates a new `HttpClient` from a configuration.
    ///
    /// Builds a `reqwest::Client` with the provided user agent, timeout, and
    /// cookie provider. If `config.cookie_store` is `Persistent(path)`,
    /// cookies are loaded from that file (if exists) and will be saved after
    /// every request.
    ///
    /// # Arguments
    ///
    /// * `config` – Configuration object containing base URL, timeout, user agent,
    ///   and cookie store policy.
    ///
    /// # Errors
    ///
    /// Returns `Error::Reqwest` if building the underlying client fails.
    /// Any error while loading cookies is logged as a warning but does not
    /// cause the construction to fail.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use librcekunit::{Config, HttpClient};
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = Config::new("https://example.com");
    /// let client = HttpClient::new(&config).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(config))]
    pub async fn new(config: &crate::Config) -> Result<Self, Error> {
        let cookie_jar = Arc::new(Jar::default());
        let client = ReqwestClient::builder()
            .user_agent(&config.user_agent)
            .timeout(config.timeout)
            .cookie_provider(Arc::clone(&cookie_jar))
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()?;

        let base_url = config.base_url.clone();
        let cookie_file = match &config.cookie_store {
            CookieStore::Persistent(path) => Some(path.clone()),
            _ => None,
        };

        let http = Self {
            client,
            base_url,
            cookie_jar,
            csrf_token: RwLock::new(None),
            cookie_file,
        };

        if let Some(ref path) = http.cookie_file {
            if let Err(e) = cookies::load_cookies_from_file(&http.cookie_jar, &http.base_url, path)
            {
                warn!("Failed to load cookies: {}", e);
            }
        }

        Ok(http)
    }

    /// Returns the base URL used for all requests.
    ///
    /// # Examples
    ///
    /// ```
    /// # use librcekunit::HttpClient;
    /// # // In real code, you'd have an instance. Here we just show the call.
    /// // let base = client.base_url();
    /// ```
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Returns a reference to the internal cookie jar.
    ///
    /// This can be used to inspect or modify cookies directly.
    ///
    /// # Examples
    ///
    /// ```
    /// # use librcekunit::HttpClient;
    /// // let jar = client.cookie_jar();
    /// ```
    pub fn cookie_jar(&self) -> &Arc<Jar> {
        &self.cookie_jar
    }

    /// Fetches a CSRF token from a given URL by parsing the HTML response.
    ///
    /// Sends a GET request to the specified URL, parses the HTML for an
    /// `<input name="_token" value="...">`, stores the token internally,
    /// and returns it.
    ///
    /// # Arguments
    ///
    /// * `url` – Path or absolute URL (joined with `base_url`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Reqwest` if the request fails.
    /// Returns `Error::CsrfNotFound` if the token cannot be found in the HTML.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use librcekunit::HttpClient;
    /// # async fn run(client: &HttpClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let token = client.fetch_csrf_token("/login").await?;
    /// println!("Token: {}", token);
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self))]
    pub async fn fetch_csrf_token(&self, url: &str) -> Result<String, Error> {
        debug!("Fetching CSRF token from {}", url);
        let full_url = join_url(&self.base_url, url);
        let resp = self.client.get(&full_url).send().await?;
        let body = resp.text().await?;
        let token = Self::parse_csrf_from_html(&body)?;
        self.set_csrf_token(token.clone()).await;
        debug!("CSRF token obtained and stored");
        Ok(token)
    }

    /// Parses a CSRF token from HTML content.
    ///
    /// Looks for an `<input name="_token" value="...">` element and extracts
    /// the `value` attribute.
    ///
    /// # Arguments
    ///
    /// * `html` – HTML string to parse.
    ///
    /// # Errors
    ///
    /// Returns `Error::CsrfNotFound` if the token input is missing.
    ///
    /// # Examples
    ///
    /// ```
    /// # use librcekunit::HttpClient;
    /// let html = r#"<input type="hidden" name="_token" value="abc123">"#;
    /// let token = HttpClient::parse_csrf_from_html(html).unwrap();
    /// assert_eq!(token, "abc123");
    /// ```
    pub fn parse_csrf_from_html(html: &str) -> Result<String, Error> {
        let document = Html::parse_document(html);
        let selector = Selector::parse("input[name='_token']").unwrap();
        document
            .select(&selector)
            .next()
            .and_then(|e| e.value().attr("value").map(String::from))
            .ok_or(Error::CsrfNotFound)
    }

    /// Stores the CSRF token for future requests.
    ///
    /// # Arguments
    ///
    /// * `token` – The token string to store.
    pub async fn set_csrf_token(&self, token: String) {
        *self.csrf_token.write().await = Some(token);
    }

    /// Returns the currently stored CSRF token, if any.
    pub async fn get_csrf_token(&self) -> Option<String> {
        self.csrf_token.read().await.clone()
    }

    /// Ensures a CSRF token is present, fetching one if missing.
    ///
    /// If no token is stored, it fetches one from `/` (the root path).
    ///
    /// # Errors
    ///
    /// Returns any error from `fetch_csrf_token`.
    async fn ensure_csrf_token(&self) -> Result<(), Error> {
        if self.get_csrf_token().await.is_none() {
            debug!("CSRF token missing, auto-fetching from /");
            self.fetch_csrf_token("/").await?;
        }
        Ok(())
    }

    /// Sends an HTTP request with automatic CSRF token injection and cookie persistence.
    ///
    /// For `GET` and `HEAD` requests, any provided `body` is treated as query parameters.
    /// For other methods (`POST`, `PUT`, `PATCH`, `DELETE`, `OPTIONS`), the request
    /// is sent as a form‑encoded body. A CSRF token is automatically added as
    /// `_token` unless already present in the form.
    ///
    /// After a successful request, cookies are saved to the persistent file if
    /// `cookie_file` is configured.
    ///
    /// # Arguments
    ///
    /// * `method` – HTTP method.
    /// * `path` – Path relative to `base_url` (or absolute).
    /// * `body` – For `GET/HEAD`: query parameters as `HashMap`.
    ///            For others: form data.
    ///
    /// # Errors
    ///
    /// Returns `Error::Reqwest` for network failures.
    /// Returns `Error::CsrfNotFound` if CSRF token is needed but cannot be fetched.
    /// Returns `Error::Io` or `Error::Json` if cookie saving fails (logged as warning).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use librcekunit::{HttpClient, HttpMethod};
    /// # use std::collections::HashMap;
    /// # async fn run(client: &HttpClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let mut params = HashMap::new();
    /// params.insert("search".to_string(), "rust".to_string());
    /// let resp = client.request(HttpMethod::GET, "/search", Some(params)).await?;
    ///
    /// let mut form = HashMap::new();
    /// form.insert("name".to_string(), "value".to_string());
    /// let resp = client.request(HttpMethod::POST, "/submit", Some(form)).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, body))]
    pub async fn request(
        &self,
        method: HttpMethod,
        path: &str,
        body: Option<HashMap<String, String>>,
    ) -> Result<reqwest::Response, Error> {
        let url = join_url(&self.base_url, path);
        debug!(%method, %url, "Sending request");

        let mut req_builder = match method {
            HttpMethod::GET => self.client.get(&url),
            HttpMethod::POST => self.client.post(&url),
            HttpMethod::PUT => self.client.put(&url),
            HttpMethod::PATCH => self.client.patch(&url),
            HttpMethod::DELETE => self.client.delete(&url),
            HttpMethod::HEAD => self.client.head(&url),
            HttpMethod::OPTIONS => self.client.request(reqwest::Method::OPTIONS, &url),
        };

        if method != HttpMethod::GET && method != HttpMethod::HEAD {
            self.ensure_csrf_token().await?;
            if let Some(token) = self.get_csrf_token().await {
                let mut form = body.unwrap_or_default();
                form.entry("_token".to_string()).or_insert(token);
                req_builder = req_builder.form(&form);
            } else {
                if let Some(form) = body {
                    req_builder = req_builder.form(&form);
                }
            }
        } else if let Some(params) = body {
            req_builder = req_builder.query(&params);
        }

        let resp = req_builder.send().await?;
        debug!(status = %resp.status(), "Response received");

        if let Some(ref path) = self.cookie_file {
            if let Err(e) = cookies::save_cookies_to_file(&self.cookie_jar, &self.base_url, path) {
                warn!("Failed to save cookies: {}", e);
            }
        }

        Ok(resp)
    }

    /// Clears the session: resets CSRF token and removes the cookie file (if any).
    ///
    /// This method:
    /// 1. Sets the stored CSRF token to an empty string.
    /// 2. If a persistent cookie file exists, deletes it.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use librcekunit::HttpClient;
    /// # async fn run(client: &HttpClient) {
    /// client.clear_session().await;
    /// # }
    /// ```
    pub async fn clear_session(&self) {
        self.set_csrf_token(String::new()).await;
        if let Some(ref path) = self.cookie_file {
            if path.exists() {
                let _ = std::fs::remove_file(path);
            }
        }
    }

    /// Resets the stored CSRF token to `None`.
    ///
    /// This forces the next non‑GET request to fetch a fresh token.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use librcekunit::HttpClient;
    /// # async fn run(client: &HttpClient) {
    /// client.reset_csrf_token().await;
    /// # }
    /// ```
    pub async fn reset_csrf_token(&self) {
        *self.csrf_token.write().await = None;
    }
}
