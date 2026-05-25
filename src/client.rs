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

    pub async fn dashboard_index(&self) -> Result<reqwest::Response, Error> {
        crate::api::dashboard::dashboard_index(&self.http).await
    }

    pub async fn dashboard_index_with_params(
        &self,
        params: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::dashboard::dashboard_index_with_params(&self.http, params).await
    }

    pub async fn cekunit_index(&self) -> Result<reqwest::Response, Error> {
        crate::api::crud::index(&self.http).await
    }

    pub async fn cekunit_index_with_params(
        &self,
        params: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::crud::index_with_params(&self.http, params).await
    }

    pub async fn cekunit_create(&self) -> Result<reqwest::Response, Error> {
        crate::api::crud::create(&self.http).await
    }

    pub async fn cekunit_store(
        &self,
        data: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::crud::store(&self.http, data).await
    }

    pub async fn cekunit_show(&self, no: u64) -> Result<reqwest::Response, Error> {
        crate::api::crud::show(&self.http, no).await
    }

    pub async fn cekunit_edit(&self, no: u64) -> Result<reqwest::Response, Error> {
        crate::api::crud::edit(&self.http, no).await
    }

    pub async fn cekunit_update(
        &self,
        no: u64,
        data: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::crud::update(&self.http, no, data).await
    }

    pub async fn cekunit_destroy(&self, no: u64) -> Result<reqwest::Response, Error> {
        crate::api::crud::destroy(&self.http, no).await
    }

    pub async fn dashboard_delete_all(&self) -> Result<reqwest::Response, Error> {
        crate::api::dashboard::delete_all(&self.http).await
    }

    pub async fn cekunit_delete_by_category(
        &self,
        column: &str,
        value: &str,
    ) -> Result<reqwest::Response, Error> {
        crate::api::dashboard::delete_by_category(&self.http, column, value).await
    }

    pub async fn cekunit_export(
        &self,
        format: &str,
        sort: &str,
        direction: &str,
    ) -> Result<reqwest::Response, Error> {
        crate::api::dashboard::export(&self.http, format, sort, direction).await
    }

    pub async fn cekunit_get_unique_values(
        &self,
        column: &str,
    ) -> Result<reqwest::Response, Error> {
        crate::api::dashboard::get_unique_values(&self.http, column).await
    }

    pub async fn users_index(&self) -> Result<reqwest::Response, Error> {
        self.request(HttpMethod::GET, "/dashboard/users", None)
            .await
    }
}
