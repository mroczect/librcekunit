use librcekunit_api::Client;
use librcekunit_handler::{Config, CookieStore, Error, Result};

pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Clone)]
pub struct ClientBuilder {
    base_url: Option<String>,
    timeout_secs: u64,
    user_agent: Option<String>,
    cookie_store: CookieStore,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            base_url: None,
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            user_agent: None,
            cookie_store: CookieStore::Memory,
        }
    }

    #[must_use]
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    #[must_use]
    pub const fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    #[must_use]
    pub fn user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    #[must_use]
    pub fn cookie_store(mut self, store: CookieStore) -> Self {
        self.cookie_store = store;
        self
    }

    pub fn build_config(self) -> Result<Config> {
        let base_url = self
            .base_url
            .ok_or_else(|| Error::Config(String::from("base_url is required")))?;

        let mut config = Config::new(&base_url).with_timeout(self.timeout_secs);
        if let Some(ua) = self.user_agent {
            config = config.with_user_agent(&ua);
        }
        config = config.with_cookie_store(self.cookie_store);
        Ok(config)
    }

    pub async fn build(self) -> Result<Client> {
        let config = self.build_config()?;
        Client::new(config).await
    }
}
