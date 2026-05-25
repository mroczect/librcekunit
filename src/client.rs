use crate::config::Config;
use crate::error::Error;
use crate::http_client::HttpClient;
use crate::types::HttpMethod;
use std::collections::HashMap;
use tracing::instrument;

pub struct Client {
    http: HttpClient,
}

impl Client {
    #[instrument]
    pub async fn new(config: Config) -> Result<Self, Error> {
        let http = HttpClient::new(&config).await?;
        Ok(Self { http })
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<(), Error> {
        crate::api::auth::login::login(&self.http, email, password).await
    }

    pub async fn logout(&self) -> Result<(), Error> {
        crate::api::auth::logout::logout(&self.http).await
    }

    pub fn http(&self) -> &HttpClient {
        &self.http
    }

    pub async fn request(
        &self,
        method: HttpMethod,
        path: &str,
        body: Option<HashMap<String, String>>,
    ) -> Result<reqwest::Response, Error> {
        self.http.request(method, path, body).await
    }

    pub async fn cekunit_index(&self) -> Result<reqwest::Response, Error> {
        self.request(HttpMethod::GET, "/dashboard/", None)
            .await
    }

    pub async fn cekunit_store(
        &self,
        data: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        self.request(HttpMethod::POST, "/dashboard/", Some(data))
            .await
    }

    pub async fn users_index(&self) -> Result<reqwest::Response, Error> {
        self.request(HttpMethod::GET, "/dashboard/users", None)
            .await
    }
}
