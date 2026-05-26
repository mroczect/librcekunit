//! Configuration management for the HTTP client.
//!
//! This module provides the `Config` struct and `CookieStore` enum to control
//! the behavior of the HTTP client, including base URL, timeout, user agent,
//! and cookie persistence strategy.
//!
//! # Examples
//!
//! ```
//! use librcekunit::{Config, CookieStore};
//! use std::path::PathBuf;
//! use std::time::Duration;
//!
//! // Create from environment variable BASE_URL
//! # std::env::set_var("BASE_URL", "https://api.example.com");
//! let config = Config::from_env().unwrap();
//!
//! // Or create manually with defaults
//! let config = Config::new("https://api.example.com")
//!     .with_timeout(60)
//!     .with_user_agent("my-app/1.0")
//!     .with_cookie_store(CookieStore::Memory);
//! ```

use std::path::PathBuf;
use std::time::Duration;

/// Configuration structure for the HTTP client.
///
/// Holds all settings required to create and operate an `HttpClient`.
/// Default values are provided by `Config::new()` and can be overridden
/// using the builder‑style methods `with_timeout`, `with_user_agent`, and
/// `with_cookie_store`.
///
/// # Fields
///
/// * `base_url` - Base URL of the target API. Trailing slashes are trimmed.
/// * `timeout` - Request timeout duration. Default is 30 seconds.
/// * `user_agent` - User‑Agent header value. Default is "librcekunit/2.0".
/// * `cookie_store` - Cookie storage policy.
///
/// # Examples
///
/// ```
/// # use librcekunit::Config;
/// let config = Config::new("https://example.com")
///     .with_timeout(10);
/// assert_eq!(config.base_url, "https://example.com");
/// ```
#[derive(Debug, Clone)]
pub struct Config {
    /// Base URL of the API. Always stored without trailing slash.
    pub base_url: String,
    /// Maximum duration for each HTTP request.
    pub timeout: Duration,
    /// Value of the `User-Agent` header.
    pub user_agent: String,
    /// How cookies are persisted across sessions.
    pub cookie_store: CookieStore,
}

/// Cookie persistence strategy.
///
/// Determines whether cookies are stored only in memory, on disk, or not at all.
///
/// # Variants
///
/// * `None` - No cookie persistence. Cookies are lost when the client drops.
/// * `Memory` - Keep cookies in memory only. Persists across requests within the same process.
/// * `Persistent(PathBuf)` - Save cookies to a file at the given path.
///   Cookies are loaded at client creation and saved after every request.
#[derive(Debug, Clone)]
pub enum CookieStore {
    /// Do not persist cookies at all.
    None,
    /// Keep cookies in memory only (default for non‑persistent usage).
    Memory,
    /// Persist cookies to a file at the specified path.
    Persistent(PathBuf),
}

impl Config {
    /// Creates a `Config` from environment variables.
    ///
    /// Reads the `BASE_URL` environment variable using `dotenvy`. If the variable
    /// is not set, returns a `Config` error.
    ///
    /// All other fields use default values:
    /// - `timeout`: 30 seconds
    /// - `user_agent`: "librcekunit/2.0"
    /// - `cookie_store`: `Persistent("librcekunit_cookies.json")`
    ///
    /// # Errors
    ///
    /// Returns `Error::Config` if `BASE_URL` is not present in the environment.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use librcekunit::Config;
    /// # std::env::set_var("BASE_URL", "https://api.example.com");
    /// let config = Config::from_env().unwrap();
    /// ```
    pub fn from_env() -> Result<Self, crate::Error> {
        dotenvy::dotenv().ok();

        let base_url = std::env::var("BASE_URL")
            .map_err(|_| crate::Error::Config("BASE_URL environment variable is not set".into()))?;

        Ok(Self::new(&base_url))
    }

    /// Creates a new configuration with default values.
    ///
    /// The provided `base_url` is trimmed of any trailing slashes.
    ///
    /// # Arguments
    ///
    /// * `base_url` - Base URL of the API. Example: `"https://example.com/"`
    ///
    /// # Defaults
    ///
    /// - `timeout`: 30 seconds
    /// - `user_agent`: "librcekunit/2.0"
    /// - `cookie_store`: `Persistent("librcekunit_cookies.json")`
    ///
    /// # Examples
    ///
    /// ```
    /// # use librcekunit::Config;
    /// let config = Config::new("https://example.com");
    /// assert_eq!(config.base_url, "https://example.com");
    /// ```
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            timeout: Duration::from_secs(30),
            user_agent: "librcekunit/2.0".into(),
            cookie_store: CookieStore::Persistent(PathBuf::from("librcekunit_cookies.json")),
        }
    }

    /// Sets the request timeout (builder pattern).
    ///
    /// Consumes `self` and returns a modified `Config`.
    ///
    /// # Arguments
    ///
    /// * `secs` - Timeout duration in seconds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use librcekunit::Config;
    /// let config = Config::new("https://example.com").with_timeout(120);
    /// assert_eq!(config.timeout.as_secs(), 120);
    /// ```
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout = Duration::from_secs(secs);
        self
    }

    /// Sets the `User-Agent` header (builder pattern).
    ///
    /// # Arguments
    ///
    /// * `ua` - User agent string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use librcekunit::Config;
    /// let config = Config::new("https://example.com").with_user_agent("my-bot/3.0");
    /// assert_eq!(config.user_agent, "my-bot/3.0");
    /// ```
    pub fn with_user_agent(mut self, ua: &str) -> Self {
        self.user_agent = ua.to_string();
        self
    }

    /// Sets the cookie store policy (builder pattern).
    ///
    /// # Arguments
    ///
    /// * `store` - A `CookieStore` variant.
    ///
    /// # Examples
    ///
    /// ```
    /// # use librcekunit::{Config, CookieStore};
    /// # use std::path::PathBuf;
    /// let config = Config::new("https://example.com")
    ///     .with_cookie_store(CookieStore::None);
    /// ```
    pub fn with_cookie_store(mut self, store: CookieStore) -> Self {
        self.cookie_store = store;
        self
    }
}
