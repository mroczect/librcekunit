use crate::handler::{Config, Error, HttpMethod, join_url};
use reqwest::cookie::Jar;
use reqwest::{Client, Response};
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct HttpClient {
    pub client: Client,
    base_url: String,
    cookie_jar: Arc<Jar>,
    csrf_token: RwLock<Option<String>>,
    cookie_file: Option<PathBuf>,
}

impl HttpClient {
    pub fn new(config: &Config) -> Result<Self, Error> {
        let cookie_jar = Arc::new(Jar::default());
        let client = Client::builder()
            .user_agent(&config.user_agent)
            .timeout(config.timeout)
            .cookie_provider(Arc::clone(&cookie_jar))
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()?;

        let http = Self {
            client,
            base_url: config.base_url.clone(),
            cookie_jar,
            csrf_token: RwLock::new(None),
            cookie_file: config.cookie_file.clone(),
        };

        if config.persistent_cookies {
            if let Some(ref _path) = http.cookie_file {}
        }

        Ok(http)
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    fn _save_cookies(&self, _path: &PathBuf) -> Result<(), Error> {
        Ok(())
    }

    fn _load_cookies(&self, _path: &PathBuf) -> Result<(), Error> {
        Ok(())
    }

    pub async fn fetch_csrf_token(&self, url: &str) -> Result<String, Error> {
        let resp = self.client.get(url).send().await?;
        let body = resp.text().await?;
        Self::parse_csrf_from_html(&body)
    }

    pub fn parse_csrf_from_html(html: &str) -> Result<String, Error> {
        let document = Html::parse_document(html);
        let selector = Selector::parse("input[name='_token']").unwrap();
        if let Some(elem) = document.select(&selector).next() {
            if let Some(token) = elem.value().attr("value") {
                return Ok(token.to_string());
            }
        }
        Err(Error::CsrfNotFound)
    }

    pub async fn set_csrf_token(&self, token: String) {
        *self.csrf_token.write().await = Some(token);
    }

    pub async fn get_csrf_token(&self) -> Option<String> {
        self.csrf_token.read().await.clone()
    }

    pub async fn request(
        &self,
        method: HttpMethod,
        path: &str,
        body: Option<HashMap<&str, &str>>,
    ) -> Result<Response, Error> {
        let url = join_url(&self.base_url, path);
        let mut req_builder = match method {
            HttpMethod::GET => self.client.get(&url),
            HttpMethod::HEAD => self.client.head(&url),
            HttpMethod::POST => self.client.post(&url),
            HttpMethod::PUT => self.client.put(&url),
            HttpMethod::PATCH => self.client.patch(&url),
            HttpMethod::DELETE => self.client.delete(&url),
            HttpMethod::OPTIONS => self.client.request(reqwest::Method::OPTIONS, &url),
        };

        if method != HttpMethod::GET && method != HttpMethod::HEAD {
            if let Some(token) = self.get_csrf_token().await {
                let mut form_data = body.unwrap_or_default();
                form_data.insert("_token", &token);
                req_builder = req_builder.form(&form_data);
            } else {
                if let Some(data) = body {
                    req_builder = req_builder.form(&data);
                }
            }
        } else {
            if let Some(params) = body {
                req_builder = req_builder.query(&params);
            }
        }

        let resp = req_builder.send().await?;
        Ok(resp)
    }

    pub fn cookie_jar(&self) -> &Arc<Jar> {
        &self.cookie_jar
    }
}
