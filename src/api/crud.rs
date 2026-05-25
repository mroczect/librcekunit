use crate::error::Error;
use crate::http_client::HttpClient;
use crate::types::HttpMethod;
use std::collections::HashMap;
use tracing::instrument;

#[instrument(skip(client))]
pub async fn index(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::GET, "/cekunit", None).await
}

#[instrument(skip(client))]
pub async fn index_with_params(
    client: &HttpClient,
    params: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, "/cekunit", Some(params))
        .await
}

#[instrument(skip(client))]
pub async fn create(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, "/cekunit/create", None)
        .await
}

#[instrument(skip(client))]
pub async fn store(
    client: &HttpClient,
    data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::POST, "/cekunit", Some(data))
        .await
}

#[instrument(skip(client))]
pub async fn show(client: &HttpClient, no: u64) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, &format!("/cekunit/{}", no), None)
        .await
}

#[instrument(skip(client))]
pub async fn edit(client: &HttpClient, no: u64) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, &format!("/cekunit/{}/edit", no), None)
        .await
}

#[instrument(skip(client))]
pub async fn update(
    client: &HttpClient,
    no: u64,
    mut data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    data.entry("_method".to_string())
        .or_insert_with(|| "PUT".to_string());
    client
        .request(HttpMethod::POST, &format!("/cekunit/{}", no), Some(data))
        .await
}

#[instrument(skip(client))]
pub async fn destroy(client: &HttpClient, no: u64) -> Result<reqwest::Response, Error> {
    let mut data = HashMap::new();
    data.insert("_method".to_string(), "DELETE".to_string());
    client
        .request(HttpMethod::POST, &format!("/cekunit/{}", no), Some(data))
        .await
}
