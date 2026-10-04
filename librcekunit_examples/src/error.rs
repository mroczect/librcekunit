use core::future::Future;
use core::num::NonZeroU32;
use core::time::Duration;

use librcekunit_client::prelude::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Kind {
    Transient,
    Auth,
    Client,
    Fatal,
}

#[must_use]
pub const fn classify(error: &Error) -> Kind {
    match error {
        Error::Network(_) | Error::Io(_) => Kind::Transient,
        Error::Auth(_) | Error::NotLoggedIn | Error::CsrfNotFound => Kind::Auth,
        Error::Api(_, _) => Kind::Client,
        Error::Config(_) | Error::Json(_) | Error::CookieStore(_) | _ => Kind::Fatal,
    }
}

#[must_use]
pub const fn is_retryable(error: &Error) -> bool {
    matches!(classify(error), Kind::Transient)
}

pub async fn retry<F, Fut, T>(attempts: NonZeroU32, mut op: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let max = attempts.get();
    let mut attempt: u32 = 0;

    loop {
        attempt = attempt.saturating_add(1);
        match op().await {
            Ok(value) => return Ok(value),
            Err(err) => {
                if !is_retryable(&err) || attempt >= max {
                    return Err(err);
                }
                let delay_ms = 200_u64.saturating_mul(u64::from(attempt));
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }
        }
    }
}

#[must_use]
pub fn user_message(error: &Error) -> String {
    match error {
        Error::Network(msg) => format!("Network problem: {msg}"),
        Error::Auth(msg) => format!("Login failed: {msg}"),
        Error::Api(status, _) => format!("Server returned HTTP {status}"),
        Error::CsrfNotFound => "Session token missing. Try logging in again.".to_owned(),
        Error::NotLoggedIn => "You are not logged in.".to_owned(),
        Error::Config(msg) => format!("Configuration problem: {msg}"),
        Error::Io(msg) => format!("File error: {msg}"),
        Error::Json(msg) => format!("Data format error: {msg}"),
        Error::CookieStore(msg) => format!("Cookie storage error: {msg}"),
        _ => format!("Error: {error}"),
    }
}
