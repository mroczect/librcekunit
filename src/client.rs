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

    pub async fn input_user_index(&self) -> Result<reqwest::Response, Error> {
        crate::api::input_user::index(&self.http).await
    }

    pub async fn input_user_index_with_params(
        &self,
        params: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::input_user::index_with_params(&self.http, params).await
    }

    pub async fn input_user_create(&self) -> Result<reqwest::Response, Error> {
        crate::api::input_user::create(&self.http).await
    }

    pub async fn input_user_store(
        &self,
        data: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::input_user::store(&self.http, data).await
    }

    pub async fn input_user_show(&self, id: u64) -> Result<reqwest::Response, Error> {
        crate::api::input_user::show(&self.http, id).await
    }

    pub async fn input_user_edit(&self, id: u64) -> Result<reqwest::Response, Error> {
        crate::api::input_user::edit(&self.http, id).await
    }

    pub async fn input_user_update(
        &self,
        id: u64,
        data: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::input_user::update(&self.http, id, data).await
    }

    pub async fn input_user_destroy(&self, id: u64) -> Result<reqwest::Response, Error> {
        crate::api::input_user::destroy(&self.http, id).await
    }

    pub async fn input_user_export(
        &self,
        params: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::input_user::export(&self.http, params).await
    }

    pub async fn input_user_import(
        &self,
        data: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::input_user::import(&self.http, data).await
    }

    pub async fn input_data_create(&self) -> Result<reqwest::Response, Error> {
        crate::api::input_data::create(&self.http).await
    }

    pub async fn input_data_store(
        &self,
        data: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::input_data::store(&self.http, data).await
    }

    pub async fn pic_index(&self) -> Result<reqwest::Response, Error> {
        crate::api::pic::index(&self.http).await
    }

    pub async fn pic_index_with_params(
        &self,
        params: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::pic::index_with_params(&self.http, params).await
    }

    pub async fn pic_create(&self) -> Result<reqwest::Response, Error> {
        crate::api::pic::create(&self.http).await
    }

    pub async fn pic_store(
        &self,
        data: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::pic::store(&self.http, data).await
    }

    pub async fn pic_show(&self, nomor: u64) -> Result<reqwest::Response, Error> {
        crate::api::pic::show(&self.http, nomor).await
    }

    pub async fn pic_edit(&self, nomor: u64) -> Result<reqwest::Response, Error> {
        crate::api::pic::edit(&self.http, nomor).await
    }

    pub async fn pic_update(
        &self,
        nomor: u64,
        data: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::pic::update(&self.http, nomor, data).await
    }

    pub async fn pic_destroy(&self, nomor: u64) -> Result<reqwest::Response, Error> {
        crate::api::pic::destroy(&self.http, nomor).await
    }

    pub async fn dashboard_pic_index(&self) -> Result<reqwest::Response, Error> {
        crate::api::pic::dashboard_index(&self.http).await
    }

    pub async fn dashboard_pic_index_with_params(
        &self,
        params: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::pic::dashboard_index_with_params(&self.http, params).await
    }

    pub async fn input_pic_create(&self) -> Result<reqwest::Response, Error> {
        crate::api::pic::input_create(&self.http).await
    }

    pub async fn input_pic_store(
        &self,
        data: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::pic::input_store(&self.http, data).await
    }

    pub async fn users_index(&self) -> Result<reqwest::Response, Error> {
        crate::api::users::index(&self.http).await
    }

    pub async fn users_index_with_params(
        &self,
        params: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::users::index_with_params(&self.http, params).await
    }

    pub async fn users_create(&self) -> Result<reqwest::Response, Error> {
        crate::api::users::create(&self.http).await
    }

    pub async fn users_store(
        &self,
        data: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::users::store(&self.http, data).await
    }

    pub async fn users_show(&self, nomor: u64) -> Result<reqwest::Response, Error> {
        crate::api::users::show(&self.http, nomor).await
    }

    pub async fn users_edit(&self, nomor: u64) -> Result<reqwest::Response, Error> {
        crate::api::users::edit(&self.http, nomor).await
    }

    pub async fn users_update(
        &self,
        nomor: u64,
        data: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::users::update(&self.http, nomor, data).await
    }

    pub async fn users_destroy(&self, nomor: u64) -> Result<reqwest::Response, Error> {
        crate::api::users::destroy(&self.http, nomor).await
    }

    pub async fn dashboard_users_index(&self) -> Result<reqwest::Response, Error> {
        crate::api::users::dashboard_index(&self.http).await
    }

    pub async fn dashboard_users_index_with_params(
        &self,
        params: HashMap<String, String>,
    ) -> Result<reqwest::Response, Error> {
        crate::api::users::dashboard_index_with_params(&self.http, params).await
    }
}
