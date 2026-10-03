use librcekunit_handler::{Error, Form, HttpMethod, Result};

use crate::http_client::HttpClient;

pub async fn login(http: &HttpClient, email: &str, password: &str) -> Result<()> {
    let token = http.fetch_csrf("/login").await?;

    let mut form = Form::new();
    let _ = form.insert(String::from("_token"), token);
    let _ = form.insert(String::from("email"), email.to_string());
    let _ = form.insert(String::from("password"), password.to_string());

    let resp = http
        .request_impl(HttpMethod::POST, "/login", Some(form))
        .await?;
    let status = resp.status();

    if status.is_success() || status.is_redirection() {
        http.set_csrf(None).await;
        return Ok(());
    }

    let body = resp
        .text()
        .await
        .map_err(|e| Error::Network(e.to_string()))?;
    Err(Error::Api(status.as_u16(), body))
}

pub async fn logout(http: &HttpClient) -> Result<()> {
    http.set_csrf(None).await;
    let resp = http
        .request_impl(HttpMethod::POST, "/logout", Some(Form::new()))
        .await?;
    let status = resp.status();

    if status.is_success() || status.is_redirection() {
        http.clear_session().await;
        return Ok(());
    }

    let body = resp
        .text()
        .await
        .map_err(|e| Error::Network(e.to_string()))?;
    Err(Error::Api(status.as_u16(), body))
}
