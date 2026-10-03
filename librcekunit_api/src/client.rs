use core::fmt;

use librcekunit_handler::{Config, Result};

use crate::http_client::HttpClient;

pub struct Client {
    http: HttpClient,
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client").finish_non_exhaustive()
    }
}

impl Client {
    pub async fn new(config: Config) -> Result<Self> {
        let http = HttpClient::new(&config).await?;
        Ok(Self { http })
    }
    pub const fn http(&self) -> &HttpClient {
        &self.http
    }

    pub async fn import_csv(&self, path: &std::path::Path) -> Result<reqwest::Response> {
        crate::endpoints::input_user::import_file(&self.http, path).await
    }
}
