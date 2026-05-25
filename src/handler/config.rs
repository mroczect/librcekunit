use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Config {
    pub base_url: String,

    pub timeout: Duration,

    pub persistent_cookies: bool,

    pub cookie_file: Option<std::path::PathBuf>,

    pub user_agent: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base_url: "http://103.31.38.200".into(),
            timeout: Duration::from_secs(30),
            persistent_cookies: true,
            cookie_file: Some(std::path::PathBuf::from("librcekunit_cookies.json")),
            user_agent: "librcekunit/2.0".into(),
        }
    }
}

impl Config {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            ..Default::default()
        }
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout = Duration::from_secs(secs);
        self
    }

    pub fn with_persistent_cookies(mut self, path: Option<std::path::PathBuf>) -> Self {
        self.persistent_cookies = true;
        self.cookie_file = path;
        self
    }
}
