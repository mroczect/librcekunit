use librcekunit_handler::{Error, Form, HttpMethod, Result};

use crate::http_client::HttpClient;

pub async fn index(http: &HttpClient) -> Result<reqwest::Response> {
    http.request_impl(HttpMethod::GET, "/dashboard", None).await
}

pub async fn index_with_params(http: &HttpClient, params: Form) -> Result<reqwest::Response> {
    http.request_impl(HttpMethod::GET, "/dashboard", Some(params))
        .await
}

pub async fn delete_all(http: &HttpClient) -> Result<reqwest::Response> {
    let mut data = Form::new();
    let _ = data.insert(String::from("_method"), String::from("DELETE"));
    http.request_impl(HttpMethod::POST, "/dashboard/delete-all", Some(data))
        .await
}

pub async fn delete_by_category(
    http: &HttpClient,
    column: &str,
    value: &str,
) -> Result<reqwest::Response> {
    if column.is_empty() || value.is_empty() {
        return Err(Error::Api(400, String::from("column/value empty")));
    }
    let mut data = Form::new();
    let _ = data.insert(String::from("column"), column.to_string());
    let _ = data.insert(String::from("value"), value.to_string());
    let _ = data
        .entry(String::from("_method"))
        .or_insert_with(|| String::from("DELETE"));
    http.request_impl(
        HttpMethod::POST,
        "/dashboard/cekunit/delete-by-category",
        Some(data),
    )
    .await
}

pub async fn export(
    http: &HttpClient,
    format: &str,
    sort: &str,
    direction: &str,
) -> Result<reqwest::Response> {
    let mut params = Form::new();
    let _ = params.insert(String::from("format"), format.to_string());
    let _ = params.insert(String::from("sort"), sort.to_string());
    let _ = params.insert(String::from("direction"), direction.to_string());
    http.request_impl(HttpMethod::GET, "/dashboard/cekunit/export", Some(params))
        .await
}

pub async fn get_unique_values(http: &HttpClient, column: &str) -> Result<reqwest::Response> {
    let mut params = Form::new();
    let _ = params.insert(String::from("column"), column.to_string());
    http.request_impl(
        HttpMethod::GET,
        "/dashboard/cekunit/get-unique-values",
        Some(params),
    )
    .await
}
