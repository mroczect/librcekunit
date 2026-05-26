//! Common types and utilities used across the library.
//!
//! This module defines:
//!
//! * [`HttpMethod`] – An enum representing standard HTTP verbs.
//! * [`join_url`] – A function to safely join a base URL and a relative path.
//! * [`ApiResponse<T>`] – A generic wrapper for structured API responses.
//!
//! # Usage
//!
//! These types are used internally by [`HttpClient`] and are also available
//! for consumers to parse responses.
//!
//! # Example
//!
//! ```
//! use librcekunit::types::{HttpMethod, join_url, ApiResponse};
//!
//! let method = HttpMethod::POST;
//! assert_eq!(method.as_str(), "POST");
//!
//! let url = join_url("https://example.com", "/api/v1");
//! assert_eq!(url, "https://example.com/api/v1");
//!
//! let response: ApiResponse<i32> = ApiResponse {
//!     status: "success".to_string(),
//!     data: Some(42),
//!     message: None,
//! };
//! ```

use serde::{Deserialize, Serialize};

/// HTTP method supported by the client.
///
/// This enum represents the standard HTTP verbs. It is used to construct
/// requests in [`HttpClient`](crate::http_client::HttpClient) and throughout
/// the API.
///
/// # Variants
///
/// | Variant | Purpose |
/// |---------|---------|
/// | `GET` | Retrieve data |
/// | `POST` | Submit data |
/// | `PUT` | Replace entire resource |
/// | `PATCH` | Partial update |
/// | `DELETE` | Remove resource |
/// | `HEAD` | Request headers only |
/// | `OPTIONS` | Request allowed methods |
///
/// # Examples
///
/// ```
/// use librcekunit::types::HttpMethod;
///
/// let method = HttpMethod::GET;
/// assert_eq!(method.as_str(), "GET");
/// assert_eq!(method.to_string(), "GET");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
}

impl HttpMethod {
    /// Returns the method name as a static string slice.
    ///
    /// This is useful for constructing HTTP requests where a string
    /// representation is needed.
    ///
    /// # Examples
    ///
    /// ```
    /// use librcekunit::types::HttpMethod;
    ///
    /// assert_eq!(HttpMethod::GET.as_str(), "GET");
    /// assert_eq!(HttpMethod::POST.as_str(), "POST");
    /// ```
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::OPTIONS => "OPTIONS",
        }
    }
}

impl std::fmt::Display for HttpMethod {
    /// Formats the HTTP method as a string (e.g., "GET", "POST").
    ///
    /// Delegates to [`as_str`](HttpMethod::as_str).
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Joins a base URL and a path, handling leading and trailing slashes
/// gracefully.
///
/// If `path` already starts with `"http"`, it is treated as an absolute URL
/// and returned unchanged. Otherwise, the base URL has its trailing slashes
/// removed, the path has its leading slashes removed, and the two are joined
/// with a single slash.
///
/// # Arguments
///
/// * `base` – Base URL (e.g., `"https://example.com/"`).
/// * `path` – Relative or absolute path (e.g., `"api/v1"`).
///
/// # Returns
///
/// A properly joined URL as a `String`.
///
/// # Examples
///
/// ```
/// use librcekunit::types::join_url;
///
/// assert_eq!(join_url("https://example.com/", "api/v1"), "https://example.com/api/v1");
/// assert_eq!(join_url("https://example.com", "/foo"), "https://example.com/foo");
/// assert_eq!(join_url("https://example.com", "https://elsewhere.com/path"), "https://elsewhere.com/path");
/// ```
///
/// # Panics
///
/// This function does **not** panic. It operates purely on string slices.
#[inline]
pub fn join_url(base: &str, path: &str) -> String {
    if path.starts_with("http") {
        return path.to_string();
    }
    let base = base.trim_end_matches('/');
    let path = path.trim_start_matches('/');
    format!("{}/{}", base, path)
}

/// Generic API response wrapper returned by many endpoints.
///
/// The server typically responds with a JSON object containing a status
/// string, an optional data payload, and an optional human‑readable message.
/// This struct can be used to deserialize such responses.
///
/// # Type Parameters
///
/// * `T` – The type of the `data` field when present.
///
/// # Fields
///
/// * `status` – Usually `"success"` or `"error"`.
/// * `data` – The actual response payload, if any.
/// * `message` – An optional message (e.g., error description).
///
/// # Examples
///
/// ```
/// use librcekunit::types::ApiResponse;
///
/// #[derive(Debug)]
/// struct User { id: u64, name: String }
///
/// let response: ApiResponse<User> = ApiResponse {
///     status: "success".to_string(),
///     data: Some(User { id: 1, name: "Alice".into() }),
///     message: None,
/// };
///
/// if response.status == "success" {
///     if let Some(user) = response.data {
///         println!("User: {}", user.name);
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Response status, e.g., `"success"` or `"error"`.
    pub status: String,
    /// Optional payload of type `T`.
    pub data: Option<T>,
    /// Optional human‑readable message, often used for error details.
    pub message: Option<String>,
}
