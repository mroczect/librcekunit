use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("API returned error: {0}")]
    Api(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("CSRF token tidak ditemukan di halaman")]
    CsrfNotFound,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Environment variable error: {0}")]
    EnvVar(#[from] std::env::VarError),

    #[error("Cookie store error: {0}")]
    CookieStore(String),

    #[error("Not logged in")]
    NotLoggedIn,
}
