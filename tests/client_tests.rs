use httpmock::prelude::*;
use librcekunit::{Client, Config, CookieStore, HttpMethod};
use std::collections::HashMap;

async fn setup() -> (MockServer, Client) {
    let server = MockServer::start();
    let config = Config::new(&server.base_url())
        .with_user_agent("librcekunit-test/2.0")
        .with_timeout(5)
        .with_cookie_store(CookieStore::None);
    let client = Client::new(config).await.expect("Failed to create client");
    (server, client)
}

#[tokio::test]
async fn test_cekunit_index_ok() {
    let (server, client) = setup().await;

    let mock = server.mock(|when, then| {
        when.method(GET).path("/dashboard/cekunit");
        then.status(200).body("list of units");
    });

    let resp = client.cekunit_index().await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_cekunit_index_server_error() {
    let (server, client) = setup().await;

    server.mock(|when, then| {
        when.method(GET).path("/dashboard/cekunit");
        then.status(500);
    });

    let resp = client.cekunit_index().await.unwrap();
    assert_eq!(resp.status(), 500);
}

#[tokio::test]
async fn test_cekunit_store_sends_form_data() {
    let (server, client) = setup().await;

    client.http().set_csrf_token("test_csrf".to_string()).await;

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/dashboard/cekunit")
            .body_includes("_token=test_csrf")
            .body_includes("name=unit1")
            .body_includes("type=alpha");
        then.status(201).body("created");
    });

    let mut data = HashMap::new();
    data.insert("name".to_string(), "unit1".to_string());
    data.insert("type".to_string(), "alpha".to_string());

    let resp = client.cekunit_store(data).await.unwrap();
    assert_eq!(resp.status(), 201);
    mock.assert();
}

#[tokio::test]
async fn test_cekunit_store_without_csrf_fetches_from_root() {
    let (server, client) = setup().await;

    let root_mock = server.mock(|when, then| {
        when.method(GET).path("/");
        then.status(200)
            .body(r#"<input name="_token" value="root_csrf">"#);
    });

    let post_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/dashboard/cekunit")
            .body_includes("_token=root_csrf")
            .body_includes("key=value");
        then.status(201);
    });

    let mut data = HashMap::new();
    data.insert("key".to_string(), "value".to_string());
    let resp = client.cekunit_store(data).await.unwrap();
    assert_eq!(resp.status(), 201);

    root_mock.assert();
    post_mock.assert();
}

#[tokio::test]
async fn test_users_index_ok() {
    let (server, client) = setup().await;

    let mock = server.mock(|when, then| {
        when.method(GET).path("/dashboard/users");
        then.status(200).body("users list");
    });

    let resp = client.users_index().await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_request_get_with_query_params() {
    let (server, client) = setup().await;

    let mock = server.mock(|when, then| {
        when.method(GET).path("/search").query_param("q", "rust");
        then.status(200);
    });

    let mut params = HashMap::new();
    params.insert("q".to_string(), "rust".to_string());

    let resp = client
        .request(HttpMethod::GET, "/search", Some(params))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_request_post_with_csrf_injection() {
    let (server, client) = setup().await;

    client.http().set_csrf_token("inj_token".to_string()).await;

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/submit")
            .body_includes("_token=inj_token")
            .body_includes("field=value");
        then.status(200);
    });

    let mut body = HashMap::new();
    body.insert("field".to_string(), "value".to_string());
    let resp = client
        .request(HttpMethod::POST, "/submit", Some(body))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_request_put() {
    let (server, client) = setup().await;

    client.http().set_csrf_token("put_csrf".to_string()).await;

    let mock = server.mock(|when, then| {
        when.method(PUT)
            .path("/item/1")
            .body_includes("_token=put_csrf");
        then.status(200);
    });

    let resp = client
        .request(HttpMethod::PUT, "/item/1", Some(HashMap::new()))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}
