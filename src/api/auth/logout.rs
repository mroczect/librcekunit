use crate::handler::{Error, HttpClient, HttpMethod};
use std::collections::HashMap;

pub async fn logout(client: &HttpClient) -> Result<(), Error> {
    let form = HashMap::new();
    let resp = client
        .request(HttpMethod::POST, "/logout", Some(form))
        .await?;

    if resp.status().is_redirection() || resp.status().is_success() {
        client.set_csrf_token(String::new()).await;
        Ok(())
    } else {
        Err(Error::Auth(format!("Logout gagal: {}", resp.status())))
    }
}
