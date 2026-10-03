extern crate alloc;
use alloc::sync::Arc;
use core::fmt;
use core::future::Future;
use std::path::PathBuf;

use librcekunit_handler::{Config, CookieStore, Form, HttpMethod, Result, Transport};
use reqwest::Client as ReqwestClient;
use reqwest::cookie::Jar;
use tokio::sync::RwLock;

use crate::cookies;
use crate::csrf;

pub struct HttpClient {
    client: ReqwestClient,
    base_url: String,
    cookie_jar: Arc<Jar>,
    csrf_token: RwLock<Option<String>>,
    cookie_file: Option<PathBuf>,
}

impl fmt::Debug for HttpClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HttpClient")
            .field("base_url", &self.base_url)
            .field("cookie_file", &self.cookie_file)
            .finish_non_exhaustive()
    }
}

impl HttpClient {
    pub async fn new(config: &Config) -> Result<Self> {
        let jar = Arc::new(Jar::default());
        let client = ReqwestClient::builder()
            .user_agent(config.user_agent.clone())
            .timeout(config.timeout)
            .cookie_provider(Arc::clone(&jar))
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .map_err(|e| librcekunit_handler::Error::Network(e.to_string()))?;

        let cookie_file = match &config.cookie_store {
            CookieStore::Persistent(p) => Some(p.clone()),
            CookieStore::None | CookieStore::Memory | _ => None,
        };

        let http = Self {
            client,
            base_url: config.base_url.trim_end_matches('/').to_string(),
            cookie_jar: jar,
            csrf_token: RwLock::new(None),
            cookie_file,
        };

        if let Some(path) = http.cookie_file.as_ref() {
            let _ = cookies::load(&http.cookie_jar, &http.base_url, path).await;
        }

        Ok(http)
    }

    pub async fn fetch_csrf(&self, path: &str) -> Result<String> {
        let url = self.join_url(path);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| librcekunit_handler::Error::Network(e.to_string()))?;
        let html = resp
            .text()
            .await
            .map_err(|e| librcekunit_handler::Error::Network(e.to_string()))?;
        let token = csrf::extract(&html)?;
        self.set_csrf(Some(token.clone())).await;
        Ok(token)
    }

    pub async fn ensure_csrf(&self) -> Result<()> {
        if self.get_csrf().await.is_some() {
            return Ok(());
        }
        let _ = self.fetch_csrf("/dashboard").await?;
        Ok(())
    }

    pub async fn set_csrf(&self, token: Option<String>) {
        let mut guard = self.csrf_token.write().await;
        *guard = token;
    }

    pub async fn get_csrf(&self) -> Option<String> {
        let guard = self.csrf_token.read().await;
        guard.clone()
    }

    pub async fn request_impl(
        &self,
        method: HttpMethod,
        path: &str,
        body: Option<Form>,
    ) -> Result<reqwest::Response> {
        if method.requires_csrf() {
            self.ensure_csrf().await?;
        }

        let url = self.join_url(path);
        let mut builder = match method {
            HttpMethod::GET => self.client.get(&url),
            HttpMethod::POST => self.client.post(&url),
            HttpMethod::PUT => self.client.put(&url),
            HttpMethod::PATCH => self.client.patch(&url),
            HttpMethod::DELETE => self.client.delete(&url),
            HttpMethod::HEAD => self.client.head(&url),
            HttpMethod::OPTIONS => self.client.request(reqwest::Method::OPTIONS, &url),
            _ => self.client.request(
                reqwest::Method::from_bytes(method.as_str().as_bytes())
                    .unwrap_or(reqwest::Method::GET),
                &url,
            ),
        };

        if method.requires_csrf() {
            let mut form = body.unwrap_or_default();
            if let Some(token) = self.get_csrf().await {
                let _ = form.entry(String::from("_token")).or_insert(token);
            }
            builder = builder.form(&form);
        } else if let Some(params) = body {
            builder = builder.query(&params);
        }

        let resp = builder
            .send()
            .await
            .map_err(|e| librcekunit_handler::Error::Network(e.to_string()))?;

        if let Some(path) = self.cookie_file.as_ref() {
            let _ = cookies::save(&self.cookie_jar, &self.base_url, path).await;
        }

        Ok(resp)
    }

    pub async fn request_multipart(
        &self,
        path: &str,
        field: &str,
        filename: String,
        bytes: Vec<u8>,
    ) -> Result<reqwest::Response> {
        self.ensure_csrf().await?;
        let url = self.join_url(path);
        let part = reqwest::multipart::Part::bytes(bytes).file_name(filename);
        let base_form = reqwest::multipart::Form::new().part(field.to_string(), part);
        let form = if let Some(token) = self.get_csrf().await {
            base_form.text("_token", token)
        } else {
            base_form
        };

        let resp = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| librcekunit_handler::Error::Network(e.to_string()))?;

        if let Some(path) = self.cookie_file.as_ref() {
            let _ = cookies::save(&self.cookie_jar, &self.base_url, path).await;
        }

        Ok(resp)
    }

    pub async fn clear_session(&self) {
        self.set_csrf(None).await;
        if let Some(path) = self.cookie_file.as_ref()
            && path.exists()
        {
            let _ = tokio::fs::remove_file(path).await;
        }
    }

    fn join_url(&self, path: &str) -> String {
        if path.starts_with("http://") || path.starts_with("https://") {
            return path.to_string();
        }
        let base = self.base_url.trim_end_matches('/');
        let p = path.trim_start_matches('/');
        format!("{base}/{p}")
    }
}

impl Transport for HttpClient {
    type Response = reqwest::Response;

    fn request(
        &self,
        method: HttpMethod,
        path: &str,
        body: Option<Form>,
    ) -> impl Future<Output = Result<Self::Response>> + Send {
        self.request_impl(method, path, body)
    }

    fn fetch_csrf_token(&self, url: &str) -> impl Future<Output = Result<String>> + Send {
        self.fetch_csrf(url)
    }

    fn reset_csrf_token(&self) -> impl Future<Output = ()> + Send {
        self.set_csrf(None)
    }
    fn clear_session(&self) -> impl Future<Output = ()> + Send {
        Self::clear_session(self)
    }
}
