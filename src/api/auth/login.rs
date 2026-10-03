//!   [`#[instrument(skip(password))]`](tracing::instrument)).
//! login::login(&client, "user@example.com", "password").await?;
use crate::error::Error;
use crate::http_client::HttpClient;
use crate::types::HttpMethod;
use std::collections::HashMap;
use tracing::{info, instrument, warn};

const CREDENTIAL_MISMATCH_PHRASE: &str = "These credentials do not match";

const MAX_ERROR_BODY_BYTES: usize = 64 * 1024; 

/// match login::login(&client, "admin@example.com", "wrong_password").await {
/// * The password is **never** written to logs. The `#[instrument(skip(password))]`
#[instrument(skip(client, password), fields(email = %email))]
pub async fn login(client: &HttpClient, email: &str, password: &str) -> Result<(), Error> {
    let email = email.trim();
    if email.is_empty() {
        warn!("Login attempt with empty email");
        return Err(Error::Auth("Email cannot be empty".into()));
    }

    let password = password.trim();
    if password.is_empty() {
        warn!("Login attempt with empty password (email: {})", email);
        return Err(Error::Auth("Password cannot be empty".into()));
    }

    let token = client.fetch_csrf_token("/login").await?;

    let mut form = HashMap::with_capacity(3);
    form.insert("_token".to_string(), token);
    form.insert("email".to_string(), email.to_string());
    form.insert("password".to_string(), password.to_string());

    let resp = client
        .request(HttpMethod::POST, "/login", Some(form))
        .await?;

    let status = resp.status();

    if status.is_success() || status.is_redirection() {
        info!("Login successful for {}", email);
        client.reset_csrf_token().await;
        return Ok(());
    }

    if status.is_client_error() || status.is_server_error() {
        let body_bytes = resp.bytes().await.unwrap_or_default();
        let limit = body_bytes.len().min(MAX_ERROR_BODY_BYTES);
        let body_snippet = String::from_utf8_lossy(&body_bytes[..limit]);

        if body_snippet.contains(CREDENTIAL_MISMATCH_PHRASE) {
            warn!("Invalid credentials for {}", email);
            return Err(Error::Auth("Invalid email or password".into()));
        }

        return Err(Error::Api(status.as_u16(), body_snippet.into_owned()));
    }

    Err(Error::Api(status.as_u16(), "Unexpected HTTP status".into()))
}
