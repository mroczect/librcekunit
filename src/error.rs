use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
                                                        #[error("Configuration error: {0}")]
    Config(String),

                                                #[error("Network error: {0}")]
    Reqwest(#[from] reqwest::Error),

                    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

                #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

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
