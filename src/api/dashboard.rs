use crate::error::Error;
use crate::http_client::HttpClient;
use crate::types::HttpMethod;
use std::collections::HashMap;
use tracing::instrument;

#[instrument(skip(client))]
pub async fn dashboard_index(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::GET, "/dashboard", None).await
}

#[instrument(skip(client))]
pub async fn dashboard_index_with_params(
    client: &HttpClient,
    params: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, "/dashboard", Some(params))
        .await
}

#[instrument(skip(client))]
pub async fn delete_all(client: &HttpClient) -> Result<reqwest::Response, Error> {
    let mut data = HashMap::new();
    data.insert("_method".to_string(), "DELETE".to_string());
    client
        .request(HttpMethod::POST, "/dashboard/delete-all", Some(data))
        .await
}

#[instrument(skip(client))]
pub async fn delete_by_category(
    client: &HttpClient,
    column: &str,
    value: &str,
) -> Result<reqwest::Response, Error> {
    let mut data = HashMap::new();
    data.insert("column".to_string(), column.to_string());
    data.insert("value".to_string(), value.to_string());
    client
        .request(
            HttpMethod::POST,
            "/dashboard/cekunit/delete-by-category",
            Some(data),
        )
        .await
}

#[instrument(skip(client))]
pub async fn export(
    client: &HttpClient,
    format: &str,
    sort: &str,
    direction: &str,
) -> Result<reqwest::Response, Error> {
    let mut params = HashMap::new();
    params.insert("format".to_string(), format.to_string());
    params.insert("sort".to_string(), sort.to_string());
    params.insert("direction".to_string(), direction.to_string());
    client
        .request(HttpMethod::GET, "/dashboard/cekunit/export", Some(params))
        .await
}

#[instrument(skip(client))]
pub async fn get_unique_values(
    client: &HttpClient,
    column: &str,
) -> Result<reqwest::Response, Error> {
    let mut params = HashMap::new();
    params.insert("column".to_string(), column.to_string());
    client
        .request(
            HttpMethod::GET,
            "/dashboard/cekunit/get-unique-values",
            Some(params),
        )
        .await
}
