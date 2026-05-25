use crate::handler::{Error, HttpClient, HttpMethod};
use std::collections::HashMap;

pub async fn login(client: &HttpClient, email: &str, password: &str) -> Result<(), Error> {
    let resp = client.request(HttpMethod::GET, "/login", None).await?;
    let body = resp.text().await?;
    let csrf = HttpClient::parse_csrf_from_html(&body)?;

    let mut form = HashMap::new();
    form.insert("_token", csrf.as_str());
    form.insert("email", email);
    form.insert("password", password);

    let resp = client
        .request(HttpMethod::POST, "/login", Some(form))
        .await?;

    if resp.status().is_redirection() || resp.status().as_u16() == 302 {
        
        let mut xsrf_token = None;
        for header_value in resp.headers().get_all("set-cookie") {
            if let Ok(cookie_str) = header_value.to_str() {
                if let Some(token) = extract_xsrf_from_cookie(cookie_str) {
                    xsrf_token = Some(token);
                    break; 
                }
            }
        }
        if let Some(token) = xsrf_token {
            client.set_csrf_token(token).await;
        }
        Ok(())
    } else {
        let status = resp.status();
        let error_body = resp.text().await.unwrap_or_default();
        Err(Error::Auth(format!(
            "Login gagal ({}): {}",
            status, error_body
        )))
    }
}

fn extract_xsrf_from_cookie(cookie_str: &str) -> Option<String> {
    for part in cookie_str.split(';') {
        let trimmed = part.trim();
        if trimmed.starts_with("XSRF-TOKEN=") {
            return Some(trimmed["XSRF-TOKEN=".len()..].to_string());
        }
    }
    None
}
