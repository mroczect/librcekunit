use librcekunit_client::prelude::{Client, ClientBuilder, CookieStore, Result};

pub const ENV_BASE_URL: &str = "LIBRCEKUNIT_BASE_URL";
pub const ENV_COOKIE_FILE: &str = "LIBRCEKUNIT_COOKIE_FILE";
pub const ENV_USER_AGENT: &str = "LIBRCEKUNIT_USER_AGENT";
pub const ENV_TIMEOUT_SECS: &str = "LIBRCEKUNIT_TIMEOUT_SECS";

pub async fn from_env() -> Result<Client> {
    librcekunit_client::env::from_env().await
}

pub async fn ephemeral(base_url: &str) -> Result<Client> {
    ClientBuilder::new()
        .base_url(base_url)
        .cookie_store(CookieStore::Memory)
        .build()
        .await
}

pub async fn persistent(base_url: &str, cookie_file: &str) -> Result<Client> {
    ClientBuilder::new()
        .base_url(base_url)
        .cookie_store(CookieStore::Persistent(cookie_file.into()))
        .build()
        .await
}
