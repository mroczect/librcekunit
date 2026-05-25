use crate::error::Error;
use crate::http_client::HttpClient;
use crate::types::HttpMethod;
use std::collections::HashMap;
use tracing::instrument;

#[instrument(skip(client))]
pub async fn index(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::GET, "/pic", None).await
}

#[instrument(skip(client))]
pub async fn index_with_params(
    client: &HttpClient,
    params: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::GET, "/pic", Some(params)).await
}

#[instrument(skip(client))]
pub async fn create(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::GET, "/pic/create", None).await
}

#[instrument(skip(client))]
pub async fn store(
    client: &HttpClient,
    data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::POST, "/pic", Some(data)).await
}

#[instrument(skip(client))]
pub async fn show(client: &HttpClient, nomor: u64) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, &format!("/pic/{}", nomor), None)
        .await
}

#[instrument(skip(client))]
pub async fn edit(client: &HttpClient, nomor: u64) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, &format!("/pic/{}/edit", nomor), None)
        .await
}

#[instrument(skip(client))]
pub async fn update(
    client: &HttpClient,
    nomor: u64,
    mut data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    data.entry("_method".to_string())
        .or_insert_with(|| "PUT".to_string());
    client
        .request(HttpMethod::POST, &format!("/pic/{}", nomor), Some(data))
        .await
}

#[instrument(skip(client))]
pub async fn destroy(client: &HttpClient, nomor: u64) -> Result<reqwest::Response, Error> {
    let mut data = HashMap::new();
    data.insert("_method".to_string(), "DELETE".to_string());
    client
        .request(HttpMethod::POST, &format!("/pic/{}", nomor), Some(data))
        .await
}

#[instrument(skip(client))]
pub async fn dashboard_index(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, "/dashboard/pic", None)
        .await
}

#[instrument(skip(client))]
pub async fn dashboard_index_with_params(
    client: &HttpClient,
    params: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, "/dashboard/pic", Some(params))
        .await
}

#[instrument(skip(client))]
pub async fn input_create(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, "/dashboard/input-PIC", None)
        .await
}

#[instrument(skip(client))]
pub async fn input_store(
    client: &HttpClient,
    data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::POST, "/dashboard/input-PIC", Some(data))
        .await
}
