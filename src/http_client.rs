//! HTTP client with cookie persistence and CSRF token management.
//!
//! This module provides [`HttpClient`], a wrapper around
//! [`reqwest::Client`] that adds:
//!
//! * Automatic cookie saving / loading to / from a JSON file.
//! * CSRF token extraction from HTML forms and automatic injection into
//!   mutating requests.
//! * Request tracing via [`tracing`].
//! * Builder‑style configuration through [`Config`].
//!
//! # Security
//!
//! * CSRF tokens are fetched on demand and stored in an asynchronous
//!   lock. They are never logged.
//! * Persistent cookie files are written with standard filesystem
//!   permissions. Ensure the path is secure.
//! * The underlying `reqwest` client enforces a maximum of 5 redirects.
//!
//! # Usage
//!
//! ```no_run
//! use librcekunit::http_client::HttpClient;
//! use librcekunit::Config;
//! use librcekunit::types::HttpMethod;
//! use std::collections::HashMap;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::new("https://example.com");
//! let client = HttpClient::new(&config).await?;
//!
//! // Fetch a page and extract CSRF token
//! let token = client.fetch_csrf_token("/login").await?;
//!
//! // Send a POST request with form data (CSRF token auto‑injected)
//! let mut form = HashMap::new();
//! form.insert("field".to_string(), "value".to_string());
//! let response = client.request(HttpMethod::POST, "/submit", Some(form)).await?;
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

/// Default URL path used to obtain a CSRF token when none is stored.
const DEFAULT_CSRF_PATH: &str = "/";

/// HTTP client with automatic CSRF token handling and cookie persistence.
///
/// `HttpClient` wraps a [`reqwest::Client`] and augments it with:
///
/// * A shared cookie jar ([`Arc<Jar>`]) that can be persisted to disk.
/// * An optional CSRF token guarded by an [`RwLock`].
/// * Automatic token injection for requests that modify state
///   (`POST`, `PUT`, `PATCH`, `DELETE`, `OPTIONS`).
/// * Cookie saving after each request when a persistent store is configured.
///
/// # Fields (private)
///
/// * `client` – The underlying `reqwest` client.
/// * `base_url` – Base URL for all requests (trimmed, no trailing slash).
/// * `cookie_jar` – Shared cookie jar.
/// * `csrf_token` – Optionally stored CSRF token.
/// * `cookie_file` – Path to the JSON cookie file, if persistence is enabled.
///
/// # Examples
///
/// ```no_run
/// use librcekunit::http_client::HttpClient;
/// use librcekunit::Config;
///
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
    /// Creates a new `HttpClient` from a [`Config`].
    ///
    /// The underlying `reqwest::Client` is built with:
    ///
    /// * The user agent and timeout from the config.
    /// * A cookie provider pointing to an internal [`Arc<Jar>`].
    /// * A redirect policy of at most 5 hops.
    ///
    /// If `config.cookie_store` is [`CookieStore::Persistent`], cookies are
    /// loaded from the given file (if it exists). Failures during loading are
    /// logged as warnings and do **not** abort construction.
    ///
    /// # Arguments
    ///
    /// * `config` – Configuration object.
    ///
    /// # Errors
    ///
    /// * [`Error::Reqwest`] – If the underlying client cannot be built.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use librcekunit::http_client::HttpClient;
    /// use librcekunit::Config;
    ///
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
    /// The returned string has no trailing slash.
    ///
    /// # Examples
    ///
    /// ```
    /// use librcekunit::http_client::HttpClient;
    /// // In real code you would have an instance.
    /// // let base = client.base_url();
    /// ```
    #[inline]
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
    /// use librcekunit::http_client::HttpClient;
    /// // let jar = client.cookie_jar();
    /// ```
    #[inline]
    pub fn cookie_jar(&self) -> &Arc<Jar> {
        &self.cookie_jar
    }

    /// Fetches a CSRF token from the given `url` by parsing the HTML
    /// response.
    ///
    /// The path (or absolute URL) is joined with [`base_url`] if it is
    /// relative. A `GET` request is sent, and the response body is searched
    /// for an `<input name="_token" value="...">` element. The extracted
    /// token is stored internally and returned.
    ///
    /// # Arguments
    ///
    /// * `url` – A path (e.g., `"/login"`) or an absolute URL. An empty
    ///   string is rejected before any network call.
    ///
    /// # Errors
    ///
    /// * [`Error::Api`] – Returned immediately if `url` is empty.
    /// * [`Error::Reqwest`] – Network error, timeout, or invalid URL.
    /// * [`Error::CsrfNotFound`] – The login page did not contain a CSRF
    ///   token.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use librcekunit::http_client::HttpClient;
    ///
    /// # async fn run(client: &HttpClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let token = client.fetch_csrf_token("/login").await?;
    /// println!("Token: {}", token);
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self))]
    pub async fn fetch_csrf_token(&self, url: &str) -> Result<String, Error> {
        if url.trim().is_empty() {
            warn!("fetch_csrf_token called with empty URL");
            return Err(Error::Api(400, "CSRF token URL cannot be empty".into()));
        }

        let full_url = join_url(&self.base_url, url);
        debug!("Fetching CSRF token from {}", full_url);
        let resp = self.client.get(&full_url).send().await?;
        let body = resp.text().await?;
        let token = Self::parse_csrf_from_html(&body)?;
        self.set_csrf_token(token.clone()).await;
        debug!("CSRF token obtained and stored");
        Ok(token)
    }

    /// Parses a CSRF token from an HTML string.
    ///
    /// Looks for the first `<input name="_token" value="...">` element and
    /// extracts the `value` attribute.
    ///
    /// # Arguments
    ///
    /// * `html` – Raw HTML content.
    ///
    /// # Errors
    ///
    /// * [`Error::CsrfNotFound`] – No matching element was found.
    ///
    /// # Examples
    ///
    /// ```
    /// use librcekunit::http_client::HttpClient;
    ///
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
    #[inline]
    pub async fn set_csrf_token(&self, token: String) {
        *self.csrf_token.write().await = Some(token);
    }

    /// Returns the currently stored CSRF token, if any.
    #[inline]
    pub async fn get_csrf_token(&self) -> Option<String> {
        self.csrf_token.read().await.clone()
    }

    /// Ensures a CSRF token is present, fetching one from
    /// [`DEFAULT_CSRF_PATH`] if missing.
    ///
    /// # Errors
    ///
    /// Propagates any error from [`fetch_csrf_token`].
    async fn ensure_csrf_token(&self) -> Result<(), Error> {
        if self.get_csrf_token().await.is_none() {
            debug!(
                "CSRF token missing, auto-fetching from {}",
                DEFAULT_CSRF_PATH
            );
            self.fetch_csrf_token(DEFAULT_CSRF_PATH).await?;
        }
        Ok(())
    }

    /// Sends an HTTP request with automatic CSRF token injection and cookie
    /// persistence.
    ///
    /// * For `GET` and `HEAD` requests, the optional `body` map is sent as
    ///   query parameters.
    /// * For other methods (`POST`, `PUT`, `PATCH`, `DELETE`, `OPTIONS`),
    ///   the map is sent as `application/x-www-form-urlencoded` body. A CSRF
    ///   token is automatically added as the `_token` field unless it is
    ///   already present in the body.
    ///
    /// If a persistent cookie file is configured, cookies are saved after
    /// the response is received, regardless of the HTTP status. Save errors
    /// are logged as warnings.
    ///
    /// # Arguments
    ///
    /// * `method` – The HTTP method.
    /// * `path` – Path relative to `base_url` (or absolute). An empty
    ///   string is rejected.
    /// * `body` – For `GET`/`HEAD`: query parameters. For others: form data.
    ///
    /// # Errors
    ///
    /// * [`Error::Api`] – Returned immediately if `path` is empty.
    /// * [`Error::Reqwest`] – Network failure.
    /// * [`Error::CsrfNotFound`] – A CSRF token was required but could not
    ///   be fetched.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use librcekunit::http_client::HttpClient;
    /// use librcekunit::types::HttpMethod;
    /// use std::collections::HashMap;
    ///
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
        if path.trim().is_empty() {
            warn!("request called with empty path");
            return Err(Error::Api(400, "Request path cannot be empty".into()));
        }

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

    /// Clears the session: resets CSRF token and removes the cookie file (if
    /// any).
    ///
    /// This method:
    /// 1. Sets the stored CSRF token to an empty string.
    /// 2. If a persistent cookie file exists, deletes it. Deletion errors
    ///    are silently ignored.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use librcekunit::http_client::HttpClient;
    ///
    /// # async fn run(client: &HttpClient) {
    /// client.clear_session().await;
    /// # }
    /// ```
    pub async fn clear_session(&self) {
        self.set_csrf_token(String::new()).await;
        if let Some(ref path) = self.cookie_file {
            if path.exists() {
                if let Err(e) = std::fs::remove_file(path) {
                    warn!("Failed to remove cookie file during session clear: {}", e);
                }
            }
        }
    }

    /// Resets the stored CSRF token to `None`.
    ///
    /// This forces the next mutating request to fetch a fresh token.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use librcekunit::http_client::HttpClient;
    ///
    /// # async fn run(client: &HttpClient) {
    /// client.reset_csrf_token().await;
    /// # }
    /// ```
    pub async fn reset_csrf_token(&self) {
        *self.csrf_token.write().await = None;
    }
}
