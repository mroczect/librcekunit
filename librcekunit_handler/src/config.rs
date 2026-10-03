use crate::enums::CookieStore;
use crate::error::{Error, Result};
use core::time::Duration;

pub const DEFAULT_TIMEOUT_SECS: u64 = 30;
pub const DEFAULT_USER_AGENT: &str = "librcekunit/3.0";
pub const DEFAULT_COOKIE_FILE: &str = "librcekunit_cookies.json";

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Config {
    pub base_url: String,
    pub timeout: Duration,
    pub user_agent: String,
    pub cookie_store: CookieStore,
}

impl Config {
    #[must_use]
    pub fn new(base_url: &str) -> Self {
        let trimmed = base_url.trim_end_matches('/');
        let base_url = if trimmed.is_empty() { "/" } else { trimmed };
        Self {
            base_url: base_url.to_string(),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            user_agent: DEFAULT_USER_AGENT.into(),
            cookie_store: CookieStore::Persistent(DEFAULT_COOKIE_FILE.into()),
        }
    }

    #[must_use]
    pub const fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout = Duration::from_secs(secs);
        self
    }

    #[must_use]
    pub fn with_user_agent(mut self, ua: &str) -> Self {
        self.user_agent = ua.to_string();
        self
    }

    #[must_use]
    pub fn with_cookie_store(mut self, store: CookieStore) -> Self {
        self.cookie_store = store;
        self
    }
}
#[allow(clippy::missing_errors_doc)]
pub trait Validate {
    fn validate(&self) -> Result<()>;
}

impl Validate for Config {
    fn validate(&self) -> Result<()> {
        if self.base_url.trim().is_empty() {
            return Err(Error::Config("base_url is empty".into()));
        }
        if !self.base_url.starts_with("http://") && !self.base_url.starts_with("https://") {
            return Err(Error::Config(
                "base_url must start with http:// or https://".into(),
            ));
        }
        if self.timeout.is_zero() {
            return Err(Error::Config("timeout must be > 0".into()));
        }
        if self.user_agent.trim().is_empty() {
            return Err(Error::Config("user_agent is empty".into()));
        }
        Ok(())
    }
}
