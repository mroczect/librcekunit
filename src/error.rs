//! Error types for the librcekunit crate.
//!
//! This module defines all possible errors that can occur during API operations,
//! including configuration issues, network failures, authentication problems,
//! and serialization errors.
//!
//! # Examples
//!
//! ```
//! use librcekunit::Error;
//!
//! fn handle_error(err: Error) {
//!     match err {
//!         Error::Config(msg) => eprintln!("Configuration error: {}", msg),
//!         Error::Reqwest(e) => eprintln!("Network error: {}", e),
//!         Error::Auth(msg) => eprintln!("Auth failed: {}", msg),
//!         // ... handle other variants
//!         _ => eprintln!("Other error: {}", err),
//!     }
//! }
//! ```

use thiserror::Error;

/// All possible errors returned by this crate.
///
/// The error type is designed to be comprehensive and integrates with
/// `thiserror` for easy conversion from common error types like `reqwest::Error`,
/// `std::io::Error`, and `serde_json::Error`.
///
/// # Variants
///
/// Each variant carries domain-specific information to help callers
/// handle errors appropriately.
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration error, typically from missing environment variables or invalid settings.
    ///
    /// Contains a human-readable message describing the configuration problem.
    ///
    /// # Examples
    ///
    /// ```
    /// # use librcekunit::Error;
    /// let err = Error::Config("BASE_URL is not set".to_string());
    /// assert_eq!(err.to_string(), "Configuration error: BASE_URL is not set");
    /// ```
    #[error("Configuration error: {0}")]
    Config(String),

    /// Network or HTTP error from the underlying `reqwest` client.
    ///
    /// This variant wraps `reqwest::Error` and occurs when the request fails to send,
    /// the connection times out, or the response cannot be processed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use librcekunit::Error;
    /// # use reqwest::Error as ReqwestError;
    /// // Typically created automatically by `?` conversion.
    /// ```
    #[error("Network error: {0}")]
    Reqwest(#[from] reqwest::Error),

    /// Filesystem I/O error, e.g., when reading or writing cookie files.
    ///
    /// Wraps `std::io::Error` from operations like `fs::read_to_string` or `fs::write`.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization or deserialization error.
    ///
    /// Occurs when parsing JSON responses or serializing cookie data.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Authentication failure, such as invalid email/password or expired session.
    ///
    /// Contains a descriptive error message.
    ///
    /// # Examples
    ///
    /// ```
    /// # use librcekunit::Error;
    /// let err = Error::Auth("Invalid email or password".to_string());
    /// ```
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// API responded with an HTTP error status (4xx, 5xx) and an optional message.
    ///
    /// The tuple contains the HTTP status code and a message extracted from the response body.
    ///
    /// # Examples
    ///
    /// ```
    /// # use librcekunit::Error;
    /// let err = Error::Api(404, "Resource not found".to_string());
    /// assert_eq!(err.to_string(), "API error (404): Resource not found");
    /// ```
    #[error("API error ({status}): {message}", status = .0, message = .1)]
    Api(u16, String),

    /// CSRF token was not found in the HTML response.
    ///
    /// This typically indicates that the login page structure has changed or
    /// the target is not a supported web application.
    ///
    /// # Examples
    ///
    /// ```
    /// # use librcekunit::Error;
    /// let err = Error::CsrfNotFound;
    /// assert_eq!(err.to_string(), "CSRF token not found in HTML");
    /// ```
    #[error("CSRF token not found in HTML")]
    CsrfNotFound,

    /// Action requires an authenticated session, but the client is not logged in.
    ///
    /// This error is returned when calling protected endpoints without prior login.
    #[error("Not logged in")]
    NotLoggedIn,

    /// Cookie store configuration or persistence error.
    ///
    /// Contains a message describing the issue with cookie storage, such as
    /// invalid path or serialization failure.
    #[error("Cookie store error: {0}")]
    CookieStore(String),
}
