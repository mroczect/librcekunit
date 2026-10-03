use librcekunit_handler::{Form, HttpMethod, Result};

use crate::http_client::HttpClient;

pub async fn create(http: &HttpClient) -> Result<reqwest::Response> {
    http.request_impl(HttpMethod::GET, "/dashboard/input-data", None)
        .await
}

pub async fn store(http: &HttpClient, data: Form) -> Result<reqwest::Response> {
    http.request_impl(HttpMethod::POST, "/dashboard/input-data", Some(data))
        .await
}
