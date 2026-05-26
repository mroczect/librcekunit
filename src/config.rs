//! Configuration management for the HTTP client.
//!
//! This module provides the [`Config`] struct and [`CookieStore`] enum to
//! control the behaviour of the HTTP client, including base URL, timeout,
//! user agent, and cookie persistence strategy.
//!
//! # Example
//!
//! ```
//! use librcekunit::{Config, CookieStore};
//! use std::path::PathBuf;
//! use std::time::Duration;
//!
//! // Create from environment variable BASE_URL
//! # unsafe { std::env::set_var("BASE_URL", "https://api.example.com"); }
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

/// Default request timeout in seconds.
const DEFAULT_TIMEOUT_SECS: u64 = 30;
/// Default `User-Agent` header value.
const DEFAULT_USER_AGENT: &str = "librcekunit/2.0";
/// Default cookie file name (relative to working directory).
const DEFAULT_COOKIE_FILE: &str = "librcekunit_cookies.json";

/// Configuration structure for the HTTP client.
///
/// Holds all settings required to create and operate an [`HttpClient`].
/// Default values are provided by [`Config::new`] and can be overridden
/// using the builder‑style methods [`with_timeout`], [`with_user_agent`],
/// and [`with_cookie_store`].
///
/// # Fields
///
/// | Field | Description |
/// |-------|-------------|
/// | `base_url` | Base URL of the target API. Trailing slashes are trimmed. |
/// | `timeout` | Request timeout duration. |
/// | `user_agent` | `User-Agent` header value. |
/// | `cookie_store` | Cookie storage policy. |
///
/// # Examples
///
/// ```
/// use librcekunit::Config;
///
/// let config = Config::new("https://example.com")
///     .with_timeout(10);
/// assert_eq!(config.base_url, "https://example.com");
/// ```
#[derive(Debug, Clone)]
pub struct Config {
    /// Base URL of the API. Always stored without a trailing slash.
    pub base_url: String,
    /// Maximum duration for each HTTP request.
    pub timeout: Duration,
    /// Value of the `User-Agent` header sent with every request.
    pub user_agent: String,
    /// How cookies are persisted across sessions.
    pub cookie_store: CookieStore,
}

/// Cookie persistence strategy.
///
/// Determines whether cookies are stored only in memory, on disk, or not
/// at all.
///
/// # Variants
///
/// * `None` – No cookie persistence. Cookies are lost when the client is
///   dropped.
/// * `Memory` – Keep cookies in memory only. Persists across requests
///   within the same process.
/// * `Persistent(PathBuf)` – Save cookies to a file at the given path.
///   Cookies are loaded when the client is created and saved after every
///   request.
///
/// # Examples
///
/// ```
/// use librcekunit::CookieStore;
/// use std::path::PathBuf;
///
/// let memory_store = CookieStore::Memory;
/// let disk_store = CookieStore::Persistent(PathBuf::from("cookies.json"));
/// ```
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
    /// Reads the `BASE_URL` environment variable using [`dotenvy`]. If the
    /// variable is not set, a [`Config`](crate::Error::Config) error is
    /// returned.
    ///
    /// All other fields use default values (see [`Config::new`]).
    ///
    /// # Errors
    ///
    /// * [`Error::Config`] – If `BASE_URL` is not present in the
    ///   environment.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use librcekunit::Config;
    ///
    /// # unsafe { std::env::set_var("BASE_URL", "https://api.example.com"); }
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
    /// The provided `base_url` is trimmed of any trailing slashes. **It must
    /// not be empty**; an empty string will be replaced with `"/"` to avoid
    /// malformed URLs.
    ///
    /// # Arguments
    ///
    /// * `base_url` – Base URL of the API (e.g., `"https://example.com/"`).
    ///
    /// # Defaults
    ///
    /// | Setting | Default value |
    /// |---------|---------------|
    /// | `timeout` | 30 seconds |
    /// | `user_agent` | `"librcekunit/2.0"` |
    /// | `cookie_store` | `Persistent("librcekunit_cookies.json")` |
    ///
    /// # Examples
    ///
    /// ```
    /// use librcekunit::Config;
    ///
    /// let config = Config::new("https://example.com");
    /// assert_eq!(config.base_url, "https://example.com");
    /// ```
    pub fn new(base_url: &str) -> Self {
        let base_url = base_url.trim_end_matches('/');
        // Avoid completely empty base URL – default to "/"
        let base_url = if base_url.is_empty() { "/" } else { base_url };

        Self {
            base_url: base_url.to_string(),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            user_agent: DEFAULT_USER_AGENT.into(),
            cookie_store: CookieStore::Persistent(PathBuf::from(DEFAULT_COOKIE_FILE)),
        }
    }

    /// Sets the request timeout (builder pattern).
    ///
    /// Consumes `self` and returns a modified `Config`.
    ///
    /// # Arguments
    ///
    /// * `secs` – Timeout duration in seconds. If `0`, the timeout will be
    ///   set to 0 (which may mean “no timeout” depending on the underlying
    ///   client).
    ///
    /// # Examples
    ///
    /// ```
    /// use librcekunit::Config;
    ///
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
    /// * `ua` – User agent string. If empty, the default user agent is
    ///   **not** restored; an empty header will be sent.
    ///
    /// # Examples
    ///
    /// ```
    /// use librcekunit::Config;
    ///
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
    /// * `store` – A [`CookieStore`] variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use librcekunit::{Config, CookieStore};
    ///
    /// let config = Config::new("https://example.com")
    ///     .with_cookie_store(CookieStore::None);
    /// ```
    pub fn with_cookie_store(mut self, store: CookieStore) -> Self {
        self.cookie_store = store;
        self
    }
}
