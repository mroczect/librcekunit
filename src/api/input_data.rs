use crate::error::Error;
use crate::http_client::HttpClient;
use crate::types::HttpMethod;
use std::collections::HashMap;
use tracing::instrument;


const PATH_INPUT_DATA: &str = "/dashboard/input-data";


#[instrument(skip(client))]
pub async fn create(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::GET, PATH_INPUT_DATA, None).await
}

#[instrument(skip(client))]
pub async fn store(
    client: &HttpClient,
    data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::POST, PATH_INPUT_DATA, Some(data))
        .await
}
