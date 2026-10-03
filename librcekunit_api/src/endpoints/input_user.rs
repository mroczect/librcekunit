use std::path::Path;

use librcekunit_handler::{Error, Form, HttpMethod, Result};

use crate::http_client::HttpClient;

pub async fn index(http: &HttpClient) -> Result<reqwest::Response> {
    http.request_impl(HttpMethod::GET, "/dashboard/input-user", None)
        .await
}

pub async fn index_with_params(http: &HttpClient, params: Form) -> Result<reqwest::Response> {
    http.request_impl(HttpMethod::GET, "/dashboard/input-user", Some(params))
        .await
}

pub async fn create(http: &HttpClient) -> Result<reqwest::Response> {
    http.request_impl(HttpMethod::GET, "/dashboard/input_user/create", None)
        .await
}

pub async fn store(http: &HttpClient, data: Form) -> Result<reqwest::Response> {
    http.request_impl(HttpMethod::POST, "/dashboard/input_user", Some(data))
        .await
}

pub async fn show(http: &HttpClient, id: u64) -> Result<reqwest::Response> {
    http.request_impl(
        HttpMethod::GET,
        &format!("/dashboard/input_user/{id}"),
        None,
    )
    .await
}

pub async fn edit(http: &HttpClient, id: u64) -> Result<reqwest::Response> {
    http.request_impl(
        HttpMethod::GET,
        &format!("/dashboard/input_user/{id}/edit"),
        None,
    )
    .await
}

pub async fn update(http: &HttpClient, id: u64, mut data: Form) -> Result<reqwest::Response> {
    let _ = data
        .entry(String::from("_method"))
        .or_insert_with(|| String::from("PUT"));
    http.request_impl(
        HttpMethod::POST,
        &format!("/dashboard/input_user/{id}"),
        Some(data),
    )
    .await
}

pub async fn destroy(http: &HttpClient, id: u64) -> Result<reqwest::Response> {
    let mut data = Form::new();
    let _ = data.insert(String::from("_method"), String::from("DELETE"));
    http.request_impl(
        HttpMethod::POST,
        &format!("/dashboard/input_user/{id}"),
        Some(data),
    )
    .await
}

pub async fn export(http: &HttpClient, params: Form) -> Result<reqwest::Response> {
    http.request_impl(
        HttpMethod::GET,
        "/dashboard/input_user/export",
        Some(params),
    )
    .await
}

pub async fn import(http: &HttpClient, data: Form) -> Result<reqwest::Response> {
    http.request_impl(HttpMethod::POST, "/dashboard/input_user/insert", Some(data))
        .await
}

pub async fn import_file(http: &HttpClient, path: &Path) -> Result<reqwest::Response> {
    let bytes = tokio::fs::read(path)
        .await
        .map_err(|e| Error::Io(e.to_string()))?;
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .map_or_else(|| String::from("upload.csv"), String::from);
    http.request_multipart("/dashboard/input_user/insert", "csv_file", filename, bytes)
        .await
}
