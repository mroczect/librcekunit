use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Config {
    pub base_url: String,
    pub timeout: Duration,
    pub user_agent: String,
    pub cookie_store: CookieStore,
}

#[derive(Debug, Clone)]
pub enum CookieStore {
    None,
    Memory,
    Persistent(PathBuf),
}

impl Config {
    pub fn from_env() -> Result<Self, crate::Error> {
        dotenvy::dotenv().ok();

        let base_url = std::env::var("BASE_URL")
            .map_err(|_| crate::Error::Config("BASE_URL environment variable is not set".into()))?;

        Ok(Self::new(&base_url))
    }

    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            timeout: Duration::from_secs(30),
            user_agent: "librcekunit/2.0".into(),
            cookie_store: CookieStore::Persistent(PathBuf::from("librcekunit_cookies.json")),
        }
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout = Duration::from_secs(secs);
        self
    }

    pub fn with_user_agent(mut self, ua: &str) -> Self {
        self.user_agent = ua.to_string();
        self
    }

    pub fn with_cookie_store(mut self, store: CookieStore) -> Self {
        self.cookie_store = store;
        self
    }
}
