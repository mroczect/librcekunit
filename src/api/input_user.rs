use crate::error::Error;
use crate::http_client::HttpClient;
use crate::types::HttpMethod;
use std::collections::HashMap;
use tracing::{instrument, warn};


const PATH_INDEX: &str = "/dashboard/input-user";
const PATH_RESOURCE: &str = "/dashboard/input_user";
const PATH_CREATE: &str = "/create";
const PATH_EDIT: &str = "/edit";
const PATH_EXPORT: &str = "/dashboard/input_user/export";
const PATH_IMPORT: &str = "/dashboard/input_user/insert";


#[instrument(skip(client))]
pub async fn index(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::GET, PATH_INDEX, None).await
}

#[instrument(skip(client))]
pub async fn index_with_params(
    client: &HttpClient,
    params: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, PATH_INDEX, Some(params))
        .await
}

#[instrument(skip(client))]
pub async fn create(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client
        .request(
            HttpMethod::GET,
            &format!("{}{}", PATH_RESOURCE, PATH_CREATE),
            None,
        )
        .await
}

#[instrument(skip(client))]
pub async fn store(
    client: &HttpClient,
    data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::POST, PATH_RESOURCE, Some(data))
        .await
}

#[instrument(skip(client))]
pub async fn show(client: &HttpClient, id: u64) -> Result<reqwest::Response, Error> {
    if id == 0 {
        warn!("Attempted to show input user with invalid ID 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    client
        .request(HttpMethod::GET, &format!("{}/{}", PATH_RESOURCE, id), None)
        .await
}

#[instrument(skip(client))]
pub async fn edit(client: &HttpClient, id: u64) -> Result<reqwest::Response, Error> {
    if id == 0 {
        warn!("Attempted to edit input user with invalid ID 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    client
        .request(
            HttpMethod::GET,
            &format!("{}/{}{}", PATH_RESOURCE, id, PATH_EDIT),
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
    if id == 0 {
        warn!("Attempted to update input user with invalid ID 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    data.entry("_method".to_string())
        .or_insert_with(|| "PUT".to_string());
    client
        .request(
            HttpMethod::POST,
            &format!("{}/{}", PATH_RESOURCE, id),
            Some(data),
        )
        .await
}

#[instrument(skip(client))]
pub async fn destroy(client: &HttpClient, id: u64) -> Result<reqwest::Response, Error> {
    if id == 0 {
        warn!("Attempted to destroy input user with invalid ID 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    let mut data = HashMap::with_capacity(1);
    data.insert("_method".to_string(), "DELETE".to_string());
    client
        .request(
            HttpMethod::POST,
            &format!("{}/{}", PATH_RESOURCE, id),
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
        .request(HttpMethod::GET, PATH_EXPORT, Some(params))
        .await
}

#[instrument(skip(client))]
pub async fn import(
    client: &HttpClient,
    data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::POST, PATH_IMPORT, Some(data))
        .await
}
