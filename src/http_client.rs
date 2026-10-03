use crate::config::CookieStore;
use crate::cookies;
use crate::error::Error;
use crate::types::{HttpMethod, join_url};
use reqwest::Client as ReqwestClient;
use reqwest::cookie::Jar;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, instrument, warn};

const DEFAULT_CSRF_PATH: &str = "/";

pub struct HttpClient {
    client: ReqwestClient,
    base_url: String,
    cookie_jar: Arc<Jar>,
    csrf_token: RwLock<Option<String>>,
    cookie_file: Option<PathBuf>,
}

impl HttpClient {
                                                                                                                                    #[instrument(skip(config))]
    pub async fn new(config: &crate::Config) -> Result<Self, Error> {
        let cookie_jar = Arc::new(Jar::default());
        let client = ReqwestClient::builder()
            .user_agent(&config.user_agent)
            .timeout(config.timeout)
            .cookie_provider(Arc::clone(&cookie_jar))
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()?;

        let base_url = config.base_url.clone();
        let cookie_file = match &config.cookie_store {
            CookieStore::Persistent(path) => Some(path.clone()),
            _ => None,
        };

        let http = Self {
            client,
            base_url,
            cookie_jar,
            csrf_token: RwLock::new(None),
            cookie_file,
        };

        if let Some(ref path) = http.cookie_file {
            if let Err(e) = cookies::load_cookies_from_file(&http.cookie_jar, &http.base_url, path)
            {
                warn!("Failed to load cookies: {}", e);
            }
        }

        Ok(http)
    }

                                                #[inline]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

                                            #[inline]
    pub fn cookie_jar(&self) -> &Arc<Jar> {
        &self.cookie_jar
    }

                                                                                                                                #[instrument(skip(self))]
    pub async fn fetch_csrf_token(&self, url: &str) -> Result<String, Error> {
        if url.trim().is_empty() {
            warn!("fetch_csrf_token called with empty URL");
            return Err(Error::Api(400, "CSRF token URL cannot be empty".into()));
        }

        let full_url = join_url(&self.base_url, url);
        debug!("Fetching CSRF token from {}", full_url);
        let resp = self.client.get(&full_url).send().await?;
        let body = resp.text().await?;
        let token = Self::parse_csrf_from_html(&body)?;
        self.set_csrf_token(token.clone()).await;
        debug!("CSRF token obtained and stored");
        Ok(token)
    }

                                                                                            pub fn parse_csrf_from_html(html: &str) -> Result<String, Error> {
        let document = Html::parse_document(html);
        let selector = Selector::parse("input[name='_token']").unwrap();
        document
            .select(&selector)
            .next()
            .and_then(|e| e.value().attr("value").map(String::from))
            .ok_or(Error::CsrfNotFound)
    }

                        #[inline]
    pub async fn set_csrf_token(&self, token: String) {
        *self.csrf_token.write().await = Some(token);
    }

        #[inline]
    pub async fn get_csrf_token(&self) -> Option<String> {
        self.csrf_token.read().await.clone()
    }

                            async fn ensure_csrf_token(&self) -> Result<(), Error> {
        if self.get_csrf_token().await.is_none() {
            debug!(
                "CSRF token missing, auto-fetching from {}",
                DEFAULT_CSRF_PATH
            );
            self.fetch_csrf_token(DEFAULT_CSRF_PATH).await?;
        }
        Ok(())
    }

                                                                                                                                                                                            #[instrument(skip(self, body))]
    pub async fn request(
        &self,
        method: HttpMethod,
        path: &str,
        body: Option<HashMap<String, String>>,
    ) -> Result<reqwest::Response, Error> {
        if path.trim().is_empty() {
            warn!("request called with empty path");
            return Err(Error::Api(400, "Request path cannot be empty".into()));
        }

        let url = join_url(&self.base_url, path);
        debug!(%method, %url, "Sending request");

        let mut req_builder = match method {
            HttpMethod::GET => self.client.get(&url),
            HttpMethod::POST => self.client.post(&url),
            HttpMethod::PUT => self.client.put(&url),
            HttpMethod::PATCH => self.client.patch(&url),
            HttpMethod::DELETE => self.client.delete(&url),
            HttpMethod::HEAD => self.client.head(&url),
            HttpMethod::OPTIONS => self.client.request(reqwest::Method::OPTIONS, &url),
        };

        if method != HttpMethod::GET && method != HttpMethod::HEAD {
            self.ensure_csrf_token().await?;
            if let Some(token) = self.get_csrf_token().await {
                let mut form = body.unwrap_or_default();
                form.entry("_token".to_string()).or_insert(token);
                req_builder = req_builder.form(&form);
            } else {
                if let Some(form) = body {
                    req_builder = req_builder.form(&form);
                }
            }
        } else if let Some(params) = body {
            req_builder = req_builder.query(&params);
        }

        let resp = req_builder.send().await?;
        debug!(status = %resp.status(), "Response received");

        if let Some(ref path) = self.cookie_file {
            if let Err(e) = cookies::save_cookies_to_file(&self.cookie_jar, &self.base_url, path) {
                warn!("Failed to save cookies: {}", e);
            }
        }

        Ok(resp)
    }

                                                                        pub async fn clear_session(&self) {
        self.set_csrf_token(String::new()).await;
        if let Some(ref path) = self.cookie_file {
            if path.exists() {
                if let Err(e) = std::fs::remove_file(path) {
                    warn!("Failed to remove cookie file during session clear: {}", e);
                }
            }
        }
    }

                                                        pub async fn reset_csrf_token(&self) {
        *self.csrf_token.write().await = None;
    }
}
