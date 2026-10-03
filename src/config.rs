use std::path::PathBuf;
use std::time::Duration;

const DEFAULT_TIMEOUT_SECS: u64 = 30;
const DEFAULT_USER_AGENT: &str = "librcekunit/2.0";
const DEFAULT_COOKIE_FILE: &str = "librcekunit_cookies.json";

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
        let base_url = base_url.trim_end_matches('/');
        let base_url = if base_url.is_empty() { "/" } else { base_url };

        Self {
            base_url: base_url.to_string(),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            user_agent: DEFAULT_USER_AGENT.into(),
            cookie_store: CookieStore::Persistent(PathBuf::from(DEFAULT_COOKIE_FILE)),
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
