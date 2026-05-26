//! Common types and utilities used across the library.
//!
//! This module defines:
//! * HTTP method enum `HttpMethod`
//! * URL joining utility `join_url`
//! * Generic API response wrapper `ApiResponse<T>`
//!
//! # Examples
//!
//! ```
//! use librcekunit::{HttpMethod, join_url, ApiResponse};
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
/// requests in `HttpClient` and throughout the API.
///
/// # Variants
///
/// * `GET` - Retrieve data
/// * `POST` - Submit data
/// * `PUT` - Replace entire resource
/// * `PATCH` - Partial update
/// * `DELETE` - Remove resource
/// * `HEAD` - Request headers only
/// * `OPTIONS` - Request allowed methods
///
/// # Examples
///
/// ```
/// # use librcekunit::HttpMethod;
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
    /// This is useful for constructing HTTP requests where a string representation
    /// is needed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use librcekunit::HttpMethod;
    /// assert_eq!(HttpMethod::GET.as_str(), "GET");
    /// assert_eq!(HttpMethod::POST.as_str(), "POST");
    /// ```
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
    /// Delegates to `as_str()`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Joins a base URL and a path, handling leading and trailing slashes.
///
/// If the `path` already starts with `"http"`, it is returned as‑is (absolute URL).
/// Otherwise, the base URL is trimmed of trailing slashes, the path is trimmed
/// of leading slashes, and they are joined with a single slash.
///
/// # Arguments
///
/// * `base` - Base URL, e.g., `"https://example.com/"`
/// * `path` - Relative or absolute path, e.g., `"api/v1"`
///
/// # Returns
///
/// A properly joined URL string.
///
/// # Examples
///
/// ```
/// # use librcekunit::join_url;
/// assert_eq!(join_url("https://example.com/", "api/v1"), "https://example.com/api/v1");
/// assert_eq!(join_url("https://example.com", "/foo"), "https://example.com/foo");
/// assert_eq!(join_url("https://example.com", "https://elsewhere.com/path"), "https://elsewhere.com/path");
/// ```
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
/// The API typically responds with a structured JSON object containing a status
/// string, optional data payload, and an optional human‑readable message.
///
/// # Type Parameters
///
/// * `T` - The type of the `data` field when present.
///
/// # Fields
///
/// * `status` - Usually `"success"` or `"error"`.
/// * `data` - The actual response payload, if any.
/// * `message` - An optional message (e.g., error description).
///
/// # Examples
///
/// ```
/// # use librcekunit::ApiResponse;
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
