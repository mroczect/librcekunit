use crate::error::Error;
use crate::http_client::HttpClient;
use crate::types::HttpMethod;
use std::collections::HashMap;
use tracing::{instrument, warn};


const PATH_DASHBOARD: &str = "/dashboard";
const PATH_DELETE_ALL: &str = "/dashboard/delete-all";
const PATH_DELETE_BY_CATEGORY: &str = "/dashboard/cekunit/delete-by-category";
const PATH_EXPORT: &str = "/dashboard/cekunit/export";
const PATH_UNIQUE_VALUES: &str = "/dashboard/cekunit/get-unique-values";


#[instrument(skip(client))]
pub async fn dashboard_index(client: &HttpClient) -> Result<reqwest::Response, Error> {
    client.request(HttpMethod::GET, PATH_DASHBOARD, None).await
}

#[instrument(skip(client))]
pub async fn dashboard_index_with_params(
    client: &HttpClient,
    params: HashMap<String, String>,
) -> Result<reqwest::Response, Error> {
    client
        .request(HttpMethod::GET, PATH_DASHBOARD, Some(params))
        .await
}

#[instrument(skip(client))]
pub async fn delete_all(client: &HttpClient) -> Result<reqwest::Response, Error> {
    let mut data = HashMap::with_capacity(1);
    data.insert("_method".to_string(), "DELETE".to_string());
    client
        .request(HttpMethod::POST, PATH_DELETE_ALL, Some(data))
        .await
}

#[instrument(skip(client), fields(column = %column, value = %value))]
pub async fn delete_by_category(
    client: &HttpClient,
    column: &str,
    value: &str,
) -> Result<reqwest::Response, Error> {
    let column = column.trim();
    let value = value.trim();

    if column.is_empty() {
        warn!("delete_by_category called with empty column");
        return Err(Error::Api(400, "Column name cannot be empty".into()));
    }
    if value.is_empty() {
        warn!("delete_by_category called with empty value");
        return Err(Error::Api(400, "Match value cannot be empty".into()));
    }

    let mut data = HashMap::with_capacity(3);
    data.insert("column".to_string(), column.to_string());
    data.insert("value".to_string(), value.to_string());
    data.entry("_method".to_string())
        .or_insert_with(|| "DELETE".to_string());

    client
        .request(HttpMethod::POST, PATH_DELETE_BY_CATEGORY, Some(data))
        .await
}

#[instrument(skip(client), fields(format = %format, sort = %sort, direction = %direction))]
pub async fn export(
    client: &HttpClient,
    format: &str,
    sort: &str,
    direction: &str,
) -> Result<reqwest::Response, Error> {
    let format = format.trim();
    let sort = sort.trim();
    let direction = direction.trim();

    if format.is_empty() {
        warn!("export called with empty format");
        return Err(Error::Api(400, "Export format cannot be empty".into()));
    }
    if sort.is_empty() {
        warn!("export called with empty sort column");
        return Err(Error::Api(400, "Sort column cannot be empty".into()));
    }
    if direction.is_empty() {
        warn!("export called with empty sort direction");
        return Err(Error::Api(400, "Sort direction cannot be empty".into()));
    }

    let mut params = HashMap::with_capacity(3);
    params.insert("format".to_string(), format.to_string());
    params.insert("sort".to_string(), sort.to_string());
    params.insert("direction".to_string(), direction.to_string());

    client
        .request(HttpMethod::GET, PATH_EXPORT, Some(params))
        .await
}

#[instrument(skip(client), fields(column = %column))]
pub async fn get_unique_values(
    client: &HttpClient,
    column: &str,
) -> Result<reqwest::Response, Error> {
    let column = column.trim();
    if column.is_empty() {
        warn!("get_unique_values called with empty column");
        return Err(Error::Api(400, "Column name cannot be empty".into()));
    }

    let mut params = HashMap::with_capacity(1);
    params.insert("column".to_string(), column.to_string());

    client
        .request(HttpMethod::GET, PATH_UNIQUE_VALUES, Some(params))
        .await
}
