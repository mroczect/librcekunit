use librcekunit_handler::{Form, HttpMethod, Result};

use crate::http_client::HttpClient;

pub async fn index(http: &HttpClient) -> Result<reqwest::Response> {
    http.request_impl(HttpMethod::GET, "/pic", None).await
}

pub async fn index_with_params(http: &HttpClient, params: Form) -> Result<reqwest::Response> {
    http.request_impl(HttpMethod::GET, "/pic", Some(params))
        .await
}

pub async fn create(http: &HttpClient) -> Result<reqwest::Response> {
    http.request_impl(HttpMethod::GET, "/pic/create", None)
        .await
}

pub async fn store(http: &HttpClient, data: Form) -> Result<reqwest::Response> {
    http.request_impl(HttpMethod::POST, "/pic", Some(data))
        .await
}

pub async fn show(http: &HttpClient, nomor: u64) -> Result<reqwest::Response> {
    http.request_impl(HttpMethod::GET, &format!("/pic/{nomor}"), None)
        .await
}

pub async fn edit(http: &HttpClient, nomor: u64) -> Result<reqwest::Response> {
    http.request_impl(HttpMethod::GET, &format!("/pic/{nomor}/edit"), None)
        .await
}

pub async fn update(http: &HttpClient, nomor: u64, mut data: Form) -> Result<reqwest::Response> {
    let _ = data
        .entry(String::from("_method"))
        .or_insert_with(|| String::from("PUT"));
    http.request_impl(HttpMethod::POST, &format!("/pic/{nomor}"), Some(data))
        .await
}

pub async fn destroy(http: &HttpClient, nomor: u64) -> Result<reqwest::Response> {
    let mut data = Form::new();
    let _ = data.insert(String::from("_method"), String::from("DELETE"));
    http.request_impl(HttpMethod::POST, &format!("/pic/{nomor}"), Some(data))
        .await
}

pub async fn dashboard_index(http: &HttpClient) -> Result<reqwest::Response> {
    http.request_impl(HttpMethod::GET, "/dashboard/pic", None)
        .await
}

pub async fn dashboard_index_with_params(
    http: &HttpClient,
    params: Form,
) -> Result<reqwest::Response> {
    http.request_impl(HttpMethod::GET, "/dashboard/pic", Some(params))
        .await
}

pub async fn input_create(http: &HttpClient) -> Result<reqwest::Response> {
    http.request_impl(HttpMethod::GET, "/dashboard/input-PIC", None)
        .await
}

pub async fn input_store(http: &HttpClient, data: Form) -> Result<reqwest::Response> {
    http.request_impl(HttpMethod::POST, "/dashboard/input-PIC", Some(data))
        .await
}
