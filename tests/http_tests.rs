use librcekunit::http_client::HttpClient;
use librcekunit::{Config, CookieStore};
use librcekunit::HttpMethod;
use std::collections::HashMap;

#[tokio::test]
async fn test_parse_csrf_from_html_found() {
    let html = r#"<form><input name="_token" value="abc123" /></form>"#;
    let token = HttpClient::parse_csrf_from_html(html).unwrap();
    assert_eq!(token, "abc123");
}

#[tokio::test]
async fn test_parse_csrf_from_html_not_found() {
    let html = "<div>no token here</div>";
    let result = HttpClient::parse_csrf_from_html(html);
    assert!(result.is_err());
    match result.unwrap_err() {
        librcekunit::Error::CsrfNotFound => {}
        other => panic!("Expected CsrfNotFound, got {:?}", other),
    }
}

#[tokio::test]
async fn test_parse_csrf_from_html_multiple_inputs_takes_first() {
    let html = r#"<input name="_token" value="first" /><input name="_token" value="second" />"#;
    let token = HttpClient::parse_csrf_from_html(html).unwrap();
    assert_eq!(token, "first");
}

async fn create_http_client() -> HttpClient {
    let config = Config::new("http://localhost").with_cookie_store(CookieStore::None);
    HttpClient::new(&config)
        .await
        .expect("Failed to create HttpClient")
}

#[tokio::test]
async fn test_set_and_get_csrf_token() {
    let http = create_http_client().await;
    http.set_csrf_token("my_token".to_string()).await;
    assert_eq!(http.get_csrf_token().await, Some("my_token".to_string()));

    http.set_csrf_token(String::new()).await;
    assert_eq!(http.get_csrf_token().await, Some(String::new()));
}

#[tokio::test]
async fn test_clear_session_resets_token() {
    let http = create_http_client().await;
    http.set_csrf_token("important_token".to_string()).await;
    http.clear_session().await;

    assert_eq!(http.get_csrf_token().await, Some(String::new()));
}

#[tokio::test]
async fn test_csrf_token_initially_none() {
    let http = create_http_client().await;
    assert_eq!(http.get_csrf_token().await, None);
}

#[tokio::test]
async fn test_base_url_getter() {
    let config = Config::new("http://test.io/api");
    let http = HttpClient::new(&config).await.unwrap();
    assert_eq!(http.base_url(), "http://test.io/api");
}

#[tokio::test]
async fn test_cookie_jar_is_accessible() {
    let http = create_http_client().await;
    let jar = http.cookie_jar();

    assert!(std::sync::Arc::strong_count(jar) >= 1);
}

#[tokio::test]
async fn test_reset_csrf_token() {
    let http = create_http_client().await;
    http.set_csrf_token("some_token".to_string()).await;
    assert!(http.get_csrf_token().await.is_some());

    http.reset_csrf_token().await;
    assert_eq!(http.get_csrf_token().await, None);
}

#[tokio::test]
async fn test_ensure_csrf_fetches_when_none() {
    use httpmock::prelude::*;
    let server = MockServer::start();
    let config = Config::new(&server.base_url()).with_cookie_store(CookieStore::None);
    let http = HttpClient::new(&config).await.unwrap();

    http.reset_csrf_token().await;

    let root_mock = server.mock(|when, then| {
        when.method(GET).path("/");
        then.status(200)
            .body(r#"<input name="_token" value="fresh_token">"#);
    });

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/any")
            .body_includes("_token=fresh_token");
        then.status(200);
    });

    let resp = http
        .request(HttpMethod::POST, "/any", Some(HashMap::new()))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    root_mock.assert();
    mock.assert();
}
