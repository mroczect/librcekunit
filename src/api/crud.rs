use crate::error::Error;
use crate::http_client::HttpClient;
use crate::types::HttpMethod;
use std::collections::HashMap;
use tracing::{instrument, warn};


const PATH_CEKUNIT: &str = "/cekunit";
const PATH_CREATE: &str = "/create";
const PATH_EDIT: &str = "/edit";


#[instrument(skip(client))]
pub async fn index(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::GET, PATH_CEKUNIT, None).await
}

#[instrument(skip(client))]
pub async fn index_with_params(
    client: &HttpClient,
    params: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, PATH_CEKUNIT, Some(params))
        .await
}

#[instrument(skip(client))]
pub async fn create(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client
        .request(
            HttpMethod::GET,
            &format!("{}{}", PATH_CEKUNIT, PATH_CREATE),
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
        .request(HttpMethod::POST, PATH_CEKUNIT, Some(data))
        .await
}

#[instrument(skip(client))]
pub async fn show(client: &HttpClient, no: u64) -> Result<reqwest::Response, Error> {
    if no == 0 {
        warn!("Attempted to show cekunit with invalid ID 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    client
        .request(HttpMethod::GET, &format!("{}/{}", PATH_CEKUNIT, no), None)
        .await
}

#[instrument(skip(client))]
pub async fn edit(client: &HttpClient, no: u64) -> Result<reqwest::Response, Error> {
    if no == 0 {
        warn!("Attempted to edit cekunit with invalid ID 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    client
        .request(
            HttpMethod::GET,
            &format!("{}/{}{}", PATH_CEKUNIT, no, PATH_EDIT),
            None,
        )
        .await
}

#[instrument(skip(client))]
pub async fn update(
    client: &HttpClient,
    no: u64,
    mut data: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    if no == 0 {
        warn!("Attempted to update cekunit with invalid ID 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    data.entry("_method".to_string())
        .or_insert_with(|| "PUT".to_string());
    client
        .request(
            HttpMethod::POST,
            &format!("{}/{}", PATH_CEKUNIT, no),
            Some(data),
        )
        .await
}

#[instrument(skip(client))]
pub async fn destroy(client: &HttpClient, no: u64) -> Result<reqwest::Response, Error> {
    if no == 0 {
        warn!("Attempted to destroy cekunit with invalid ID 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    let mut data = HashMap::with_capacity(1);
    data.insert("_method".to_string(), "DELETE".to_string());
    client
        .request(
            HttpMethod::POST,
            &format!("{}/{}", PATH_CEKUNIT, no),
            Some(data),
        )
        .await
}
