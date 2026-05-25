use crate::error::Error;
use crate::http_client::HttpClient;
use crate::types::HttpMethod;
use std::collections::HashMap;
use tracing::instrument;

#[instrument(skip(client))]
pub async fn logout(client: &HttpClient) -> Result<(), Error> {
    let form = HashMap::new();
    let resp = client
        .request(HttpMethod::POST, "/logout", Some(form))
        .await?;

    if resp.status().is_redirection() || resp.status().is_success() {
        client.clear_session().await;
        Ok(())
    } else {
        Err(Error::Api(resp.status().as_u16(), "Logout failed".into()))
    }
}
