use core::fmt::Write as _;
use std::path::PathBuf;

use librcekunit_api::Client;
use librcekunit_handler::{Auth, CookieStore, Error, Result};

use crate::builder::ClientBuilder;

pub const ENV_BASE_URL: &str = "LIBRCEKUNIT_BASE_URL";
pub const ENV_COOKIE_FILE: &str = "LIBRCEKUNIT_COOKIE_FILE";
pub const ENV_USER_AGENT: &str = "LIBRCEKUNIT_USER_AGENT";
pub const ENV_TIMEOUT_SECS: &str = "LIBRCEKUNIT_TIMEOUT_SECS";
pub const ENV_EMAIL: &str = "LIBRCEKUNIT_EMAIL";
pub const ENV_PASSWORD: &str = "LIBRCEKUNIT_PASSWORD";

fn missing(key: &str) -> Error {
    let mut msg = String::with_capacity(key.len().saturating_add(12));
    let _ = write!(msg, "{key} not set");
    Error::Config(msg)
}

fn invalid(key: &str, reason: &str) -> Error {
    let mut msg = String::with_capacity(key.len().saturating_add(reason.len()).saturating_add(10));
    let _ = write!(msg, "{key} invalid: {reason}");
    Error::Config(msg)
}

pub async fn from_env() -> Result<Client> {
    from_env_with(|key| std::env::var(key).ok()).await
}

pub async fn from_env_with<F>(lookup: F) -> Result<Client>
where
    F: Fn(&str) -> Option<String>,
{
    let base_url = lookup(ENV_BASE_URL).ok_or_else(|| missing(ENV_BASE_URL))?;
    let mut builder = ClientBuilder::new().base_url(base_url);

    if let Some(cookie_file) = lookup(ENV_COOKIE_FILE) {
        builder = builder.cookie_store(CookieStore::Persistent(PathBuf::from(cookie_file)));
    }

    if let Some(ua) = lookup(ENV_USER_AGENT) {
        builder = builder.user_agent(ua);
    }

    if let Some(raw) = lookup(ENV_TIMEOUT_SECS) {
        let secs = raw
            .parse::<u64>()
            .map_err(|e| invalid(ENV_TIMEOUT_SECS, &e.to_string()))?;
        builder = builder.timeout_secs(secs);
    }

    builder.build().await
}

pub async fn login_from_env(client: &Client) -> Result<()> {
    login_from_env_with(client, |key| std::env::var(key).ok()).await
}

pub async fn login_from_env_with<F>(client: &Client, lookup: F) -> Result<()>
where
    F: Fn(&str) -> Option<String>,
{
    let email = lookup(ENV_EMAIL).ok_or_else(|| missing(ENV_EMAIL))?;
    let password = lookup(ENV_PASSWORD).ok_or_else(|| missing(ENV_PASSWORD))?;
    client.login(&email, &password).await
}
