use crate::error::Error;
use crate::http_client::HttpClient;
use crate::types::HttpMethod;
use std::collections::HashMap;
use tracing::{instrument, warn};


const PATH_PIC: &str = "/pic";
const PATH_CREATE: &str = "/create";
const PATH_EDIT: &str = "/edit";
const PATH_DASHBOARD_PIC: &str = "/dashboard/pic";
const PATH_INPUT_PIC: &str = "/dashboard/input-PIC";


#[instrument(skip(client))]
pub async fn index(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::GET, PATH_PIC, None).await
}

#[instrument(skip(client))]
pub async fn index_with_params(
    client: &HttpClient,
    params: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, PATH_PIC, Some(params))
        .await
}

#[instrument(skip(client))]
pub async fn create(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client
        .request(
            HttpMethod::GET,
            &format!("{}{}", PATH_PIC, PATH_CREATE),
            None,
        )
        .await
}

#[instrument(skip(client))]
pub async fn store(
    client: &HttpClient,
    data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::POST, PATH_PIC, Some(data)).await
}

#[instrument(skip(client))]
pub async fn show(client: &HttpClient, nomor: u64) -> Result<reqwest::Response, Error> {
    if nomor == 0 {
        warn!("Attempted to show PIC with invalid nomor 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    client
        .request(HttpMethod::GET, &format!("{}/{}", PATH_PIC, nomor), None)
        .await
}

#[instrument(skip(client))]
pub async fn edit(client: &HttpClient, nomor: u64) -> Result<reqwest::Response, Error> {
    if nomor == 0 {
        warn!("Attempted to edit PIC with invalid nomor 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    client
        .request(
            HttpMethod::GET,
            &format!("{}/{}{}", PATH_PIC, nomor, PATH_EDIT),
            None,
        )
        .await
}

#[instrument(skip(client))]
pub async fn update(
    client: &HttpClient,
    nomor: u64,
    mut data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    if nomor == 0 {
        warn!("Attempted to update PIC with invalid nomor 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    data.entry("_method".to_string())
        .or_insert_with(|| "PUT".to_string());
    client
        .request(
            HttpMethod::POST,
            &format!("{}/{}", PATH_PIC, nomor),
            Some(data),
        )
        .await
}

#[instrument(skip(client))]
pub async fn destroy(client: &HttpClient, nomor: u64) -> Result<reqwest::Response, Error> {
    if nomor == 0 {
        warn!("Attempted to destroy PIC with invalid nomor 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    let mut data = HashMap::with_capacity(1);
    data.insert("_method".to_string(), "DELETE".to_string());
    client
        .request(
            HttpMethod::POST,
            &format!("{}/{}", PATH_PIC, nomor),
            Some(data),
        )
        .await
}

#[instrument(skip(client))]
pub async fn dashboard_index(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, PATH_DASHBOARD_PIC, None)
        .await
}

#[instrument(skip(client))]
pub async fn dashboard_index_with_params(
    client: &HttpClient,
    params: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, PATH_DASHBOARD_PIC, Some(params))
        .await
}

#[instrument(skip(client))]
pub async fn input_create(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::GET, PATH_INPUT_PIC, None).await
}

#[instrument(skip(client))]
pub async fn input_store(
    client: &HttpClient,
    data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::POST, PATH_INPUT_PIC, Some(data))
        .await
}
