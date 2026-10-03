use crate::error::Error;
use crate::http_client::HttpClient;
use crate::types::HttpMethod;
use std::collections::HashMap;
use tracing::{instrument, warn};


const PATH_USERS: &str = "/users";
const PATH_CREATE: &str = "/create";
const PATH_EDIT: &str = "/edit";
const PATH_DASHBOARD_USERS: &str = "/dashboard/users";


#[instrument(skip(client))]
pub async fn index(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::GET, PATH_USERS, None).await
}

#[instrument(skip(client))]
pub async fn index_with_params(
    client: &HttpClient,
    params: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, PATH_USERS, Some(params))
        .await
}

#[instrument(skip(client))]
pub async fn create(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client
        .request(
            HttpMethod::GET,
            &format!("{}{}", PATH_USERS, PATH_CREATE),
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
        .request(HttpMethod::POST, PATH_USERS, Some(data))
        .await
}

#[instrument(skip(client))]
pub async fn show(client: &HttpClient, nomor: u64) -> Result<reqwest::Response, Error> {
    if nomor == 0 {
        warn!("Attempted to show user with invalid nomor 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    client
        .request(HttpMethod::GET, &format!("{}/{}", PATH_USERS, nomor), None)
        .await
}

#[instrument(skip(client))]
pub async fn edit(client: &HttpClient, nomor: u64) -> Result<reqwest::Response, Error> {
    if nomor == 0 {
        warn!("Attempted to edit user with invalid nomor 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    client
        .request(
            HttpMethod::GET,
            &format!("{}/{}{}", PATH_USERS, nomor, PATH_EDIT),
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
        warn!("Attempted to update user with invalid nomor 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    data.entry("_method".to_string())
        .or_insert_with(|| "PUT".to_string());
    client
        .request(
            HttpMethod::POST,
            &format!("{}/{}", PATH_USERS, nomor),
            Some(data),
        )
        .await
}

#[instrument(skip(client))]
pub async fn destroy(client: &HttpClient, nomor: u64) -> Result<reqwest::Response, Error> {
    if nomor == 0 {
        warn!("Attempted to destroy user with invalid nomor 0");
        return Err(Error::Api(400, "Invalid resource ID: must be > 0".into()));
    }
    let mut data = HashMap::with_capacity(1);
    data.insert("_method".to_string(), "DELETE".to_string());
    client
        .request(
            HttpMethod::POST,
            &format!("{}/{}", PATH_USERS, nomor),
            Some(data),
        )
        .await
}

#[instrument(skip(client))]
pub async fn dashboard_index(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, PATH_DASHBOARD_USERS, None)
        .await
}

#[instrument(skip(client))]
pub async fn dashboard_index_with_params(
    client: &HttpClient,
    params: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, PATH_DASHBOARD_USERS, Some(params))
        .await
}
