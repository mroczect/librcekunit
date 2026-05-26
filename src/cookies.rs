//! Cookie persistence utilities.
//!
//! This module provides functions to save and load cookies to/from a JSON
//! file. It integrates with [`reqwest::cookie::Jar`] and uses
//! [`serde_json`] for serialization.
//!
//! # Security
//!
//! * Cookie files are written with default filesystem permissions; the
//!   caller should ensure the path is in a secure location.
//! * The JSON format stores cookie strings as‑is; sensitive session tokens
//!   will be written to disk. Use [`CookieStore::None`] for in‑memory only.
//!
//! # Example
//!
//! ```
//! use librcekunit::cookies::{save_cookies_to_file, load_cookies_from_file};
//! use reqwest::cookie::Jar;
//! use std::path::Path;
//! use std::sync::Arc;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let jar = Arc::new(Jar::default());
//! let url = "https://example.com";
//! let path = Path::new("cookies.json");
//!
//! // Save cookies (if any)
//! save_cookies_to_file(&jar, url, path)?;
//!
//! // Load cookies from file
//! load_cookies_from_file(&jar, url, path)?;
//! # Ok(())
//! # }
//! ```

use crate::error::Error;
use reqwest::cookie::{CookieStore, Jar};
use std::fs;
use std::path::Path;
use tracing::{debug, instrument};

/// Saves all cookies from a [`Jar`] to a JSON file.
///
/// Cookies are extracted for the given URL and serialized as a list of
/// strings. If no cookies are present, the file is **removed** (if it
/// exists) to avoid stale data.
///
/// # Arguments
///
/// * `jar` – The cookie jar to read from.
/// * `url` – The URL used to retrieve cookies. Must be a valid absolute
///   URL; panics if parsing fails.
/// * `path` – Filesystem path where the JSON file will be written.
///
/// # Errors
///
/// * [`Error::Io`] – If file operations (remove, write) fail.
/// * [`Error::Json`] – If serialization to JSON fails.
///
/// # Panics
///
/// This function **panics** if the provided `url` cannot be parsed as a
/// valid URL. This is intentional because the URL is expected to come from
/// a known valid source (e.g., [`Config::base_url`](crate::Config::base_url)).
///
/// # Examples
///
/// ```no_run
/// use librcekunit::cookies::save_cookies_to_file;
/// use reqwest::cookie::Jar;
/// use std::sync::Arc;
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let jar = Arc::new(Jar::default());
/// let url = "https://example.com";
/// let path = Path::new("cookies.json");
/// save_cookies_to_file(&jar, url, path)?;
/// # Ok(())
/// # }
/// ```
#[instrument(skip(jar))]
pub fn save_cookies_to_file(jar: &Jar, url: &str, path: &Path) -> Result<(), Error> {
    let url_parsed = url.parse().expect("Invalid base URL for cookie saving");
    let cookies: Vec<String> = jar
        .cookies(&url_parsed)
        .into_iter()
        .map(|h| h.to_str().unwrap_or_default().to_string())
        .collect();

    if cookies.is_empty() {
        debug!("No cookies to save, removing file if exists");
        if path.exists() {
            fs::remove_file(path)?;
        }
        return Ok(());
    }

    let json = serde_json::to_string(&cookies)?;
    fs::write(path, &json)?;
    debug!(?path, "Saved {} cookies to file", cookies.len());
    Ok(())
}

/// Loads cookies from a JSON file into a [`Jar`].
///
/// If the file does not exist, the function does nothing and returns
/// `Ok(())`. Otherwise, it reads the file, deserializes a list of cookie
/// strings, and adds each one to the jar for the given URL.
///
/// # Arguments
///
/// * `jar` – The cookie jar to populate.
/// * `url` – The URL associated with the cookies (must be valid; panics
///   on parse failure).
/// * `path` – Path to the JSON file.
///
/// # Errors
///
/// * [`Error::Io`] – If reading the file fails.
/// * [`Error::Json`] – If deserialization fails.
///
/// # Panics
///
/// This function panics if the provided `url` cannot be parsed. See
/// [`save_cookies_to_file`] for rationale.
///
/// # Examples
///
/// ```no_run
/// use librcekunit::cookies::load_cookies_from_file;
/// use reqwest::cookie::Jar;
/// use std::sync::Arc;
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let jar = Arc::new(Jar::default());
/// let url = "https://example.com";
/// let path = Path::new("cookies.json");
/// load_cookies_from_file(&jar, url, path)?;
/// # Ok(())
/// # }
/// ```
#[instrument(skip(jar))]
pub fn load_cookies_from_file(jar: &Jar, url: &str, path: &Path) -> Result<(), Error> {
    if !path.exists() {
        debug!("Cookie file not found, skipping load");
        return Ok(());
    }

    let data = fs::read_to_string(path)?;
    let cookies: Vec<String> = serde_json::from_str(&data)?;
    let url_parsed = url.parse().expect("Invalid base URL for cookie loading");

    for cookie_str in &cookies {
        jar.add_cookie_str(cookie_str, &url_parsed);
    }

    debug!(?path, "Loaded {} cookies from file", cookies.len());
    Ok(())
}
