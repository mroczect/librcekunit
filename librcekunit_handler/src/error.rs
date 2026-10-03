use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("IO error: {0}")]
    Io(String),
    #[error("JSON error: {0}")]
    Json(String),
    #[error("Authentication failed: {0}")]
    Auth(String),
    #[error("API error ({status}): {message}", status = .0, message = .1)]
    Api(u16, String),
    #[error("CSRF token not found in HTML")]
    CsrfNotFound,
    #[error("Not logged in")]
    NotLoggedIn,
    #[error("Cookie store error: {0}")]
    CookieStore(String),
}

pub type Result<T, E = Error> = core::result::Result<T, E>;
