use crate::error::Error;
use crate::http_client::HttpClient;
use crate::types::HttpMethod;
use std::collections::HashMap;
use tracing::instrument;

#[instrument(skip(client))]
pub async fn index(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, "/dashboard/input-user", None)
        .await
}

#[instrument(skip(client))]
pub async fn index_with_params(
    client: &HttpClient,
    params: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, "/dashboard/input-user", Some(params))
        .await
}

#[instrument(skip(client))]
pub async fn create(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, "/dashboard/input_user/create", None)
        .await
}

#[instrument(skip(client))]
pub async fn store(
    client: &HttpClient,
    data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::POST, "/dashboard/input_user", Some(data))
        .await
}

#[instrument(skip(client))]
pub async fn show(client: &HttpClient, id: u64) -> Result<reqwest::Response, Error> {
    client
        .request(
            HttpMethod::GET,
            &format!("/dashboard/input_user/{}", id),
            None,
        )
        .await
}

#[instrument(skip(client))]
pub async fn edit(client: &HttpClient, id: u64) -> Result<reqwest::Response, Error> {
    client
        .request(
            HttpMethod::GET,
            &format!("/dashboard/input_user/{}/edit", id),
            None,
        )
        .await
}

#[instrument(skip(client))]
pub async fn update(
    client: &HttpClient,
    id: u64,
    mut data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    data.entry("_method".to_string())
        .or_insert_with(|| "PUT".to_string());
    client
        .request(
            HttpMethod::POST,
            &format!("/dashboard/input_user/{}", id),
            Some(data),
        )
        .await
}

#[instrument(skip(client))]
pub async fn destroy(client: &HttpClient, id: u64) -> Result<reqwest::Response, Error> {
    let mut data = HashMap::new();
    data.insert("_method".to_string(), "DELETE".to_string());
    client
        .request(
            HttpMethod::POST,
            &format!("/dashboard/input_user/{}", id),
            Some(data),
        )
        .await
}

#[instrument(skip(client))]
pub async fn export(
    client: &HttpClient,
    params: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(
            HttpMethod::GET,
            "/dashboard/input_user/export",
            Some(params),
        )
        .await
}

#[instrument(skip(client))]
pub async fn import(
    client: &HttpClient,
    data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::POST, "/dashboard/input_user/insert", Some(data))
        .await
}
