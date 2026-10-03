use librcekunit_handler::{Form, HttpMethod, Result};

use crate::http_client::HttpClient;

pub async fn index(http: &HttpClient) -> Result<reqwest::Response> {
    http.request_impl(HttpMethod::GET, "/cekunit", None).await
}

pub async fn index_with_params(http: &HttpClient, params: Form) -> Result<reqwest::Response> {
    http.request_impl(HttpMethod::GET, "/cekunit", Some(params))
        .await
}

pub async fn create(http: &HttpClient) -> Result<reqwest::Response> {
    http.request_impl(HttpMethod::GET, "/cekunit/create", None)
        .await
}

pub async fn store(http: &HttpClient, data: Form) -> Result<reqwest::Response> {
    http.request_impl(HttpMethod::POST, "/cekunit", Some(data))
        .await
}

pub async fn show(http: &HttpClient, no: u64) -> Result<reqwest::Response> {
    http.request_impl(HttpMethod::GET, &format!("/cekunit/{no}"), None)
        .await
}

pub async fn edit(http: &HttpClient, no: u64) -> Result<reqwest::Response> {
    http.request_impl(HttpMethod::GET, &format!("/cekunit/{no}/edit"), None)
        .await
}

pub async fn update(http: &HttpClient, no: u64, mut data: Form) -> Result<reqwest::Response> {
    let _ = data
        .entry(String::from("_method"))
        .or_insert_with(|| String::from("PUT"));
    http.request_impl(HttpMethod::POST, &format!("/cekunit/{no}"), Some(data))
        .await
}

pub async fn destroy(http: &HttpClient, no: u64) -> Result<reqwest::Response> {
    let mut data = Form::new();
    let _ = data.insert(String::from("_method"), String::from("DELETE"));
    http.request_impl(HttpMethod::POST, &format!("/cekunit/{no}"), Some(data))
        .await
}
