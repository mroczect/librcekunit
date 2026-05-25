use crate::error::Error;
use crate::http_client::HttpClient;
use crate::types::HttpMethod;
use std::collections::HashMap;
use tracing::instrument;

#[instrument(skip(client))]
pub async fn create(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, "/dashboard/input-data", None)
        .await
}

#[instrument(skip(client))]
pub async fn store(
    client: &HttpClient,
    data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::POST, "/dashboard/input-data", Some(data))
        .await
}
