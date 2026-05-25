use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    GET,
    HEAD,
    POST,
    PUT,
    PATCH,
    DELETE,
    OPTIONS,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::OPTIONS => "OPTIONS",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteInfo {
    pub domain: Option<String>,
    pub method: String,
    pub uri: String,
    pub name: Option<String>,
    pub action: String,
    pub middleware: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub status: String,
    pub data: Option<T>,
    pub message: Option<String>,
}
