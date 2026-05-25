use crate::error::Error;
use crate::http_client::HttpClient;
use crate::types::HttpMethod;
use std::collections::HashMap;
use tracing::{info, instrument};

#[instrument(skip(client, password), fields(email = %email))]
pub async fn login(client: &HttpClient, email: &str, password: &str) -> Result<(), Error> {
    let token = client.fetch_csrf_token("/login").await?;

    let mut form = HashMap::new();
    form.insert("_token".to_string(), token);
    form.insert("email".to_string(), email.to_string());
    form.insert("password".to_string(), password.to_string());

    let resp = client
        .request(HttpMethod::POST, "/login", Some(form))
        .await?;

    let status = resp.status();

    if status.is_success() || status.is_redirection() {
        info!("Login successful");
        client.reset_csrf_token().await;
        Ok(())
    } else if status.is_client_error() || status.is_server_error() {
        let body = resp.text().await.unwrap_or_default();
        if body.contains("These credentials do not match") {
            return Err(Error::Auth("Invalid email or password".into()));
        }
        Err(Error::Api(status.as_u16(), body))
    } else {
        Err(Error::Api(status.as_u16(), "Unexpected response".into()))
    }
}
