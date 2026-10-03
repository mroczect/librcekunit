use crate::error::Error;
use crate::http_client::HttpClient;
use crate::types::HttpMethod;
use std::collections::HashMap;
use tracing::{info, instrument, warn};

const LOGOUT_PATH: &str = "/logout";

const MAX_ERROR_BODY_BYTES: usize = 64 * 1024; 

#[instrument(skip(client))]
pub async fn logout(client: &HttpClient) -> Result<(), Error> {
    client.reset_csrf_token().await;

    let form = HashMap::new();
    let resp = client
        .request(HttpMethod::POST, LOGOUT_PATH, Some(form))
        .await?;

    let status = resp.status();

    if status.is_success() || status.is_redirection() {
        info!("Logout successful – clearing session state");
        client.clear_session().await;
        return Ok(());
    }

    let body_bytes = resp.bytes().await.unwrap_or_default();
    let limit = body_bytes.len().min(MAX_ERROR_BODY_BYTES);
    let body_snippet = String::from_utf8_lossy(&body_bytes[..limit]);

    warn!(
        status = %status,
        body = %body_snippet,
        "Logout request failed"
    );

    Err(Error::Api(
        status.as_u16(),
        if body_snippet.is_empty() {
            "Logout failed".into()
        } else {
            body_snippet.into_owned()
        },
    ))
}
