use crate::api::auth;
use crate::handler::{Config, Error, HttpClient, HttpMethod};
use std::collections::HashMap;

pub struct Client {
    http: HttpClient,
    token_cache: auth::TokenCache,
}

impl Client {
    pub async fn new(config: Config) -> Result<Self, Error> {
        let http = HttpClient::new(&config)?;
        Ok(Self {
            http,
            token_cache: auth::TokenCache::new(),
        })
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<(), Error> {
        auth::login::login(&self.http, email, password).await
    }

    pub async fn logout(&self) -> Result<(), Error> {
        auth::logout::logout(&self.http).await
    }

    pub fn http(&self) -> &HttpClient {
        &self.http
    }

    pub fn cache(&self) -> &auth::TokenCache {
        &self.token_cache
    }

    pub async fn request(
        &self,
        method: HttpMethod,
        path: &str,
        form_data: Option<HashMap<&str, &str>>,
    ) -> Result<reqwest::Response, Error> {
        self.http.request(method, path, form_data).await
    }

    pub async fn cekunit_index(&self) -> Result<reqwest::Response, Error> {
        self.request(HttpMethod::GET, "/dashboard/cekunit", None)
            .await
    }

    pub async fn cekunit_store(
        &self,
        data: HashMap<&str, &str>,
    ) -> Result<reqwest::Response, Error> {
        self.request(HttpMethod::POST, "/dashboard/cekunit", Some(data))
            .await
    }

    pub async fn users_index(&self) -> Result<reqwest::Response, Error> {
        self.request(HttpMethod::GET, "/dashboard/users", None)
            .await
    }
}
