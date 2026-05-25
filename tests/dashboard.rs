use httpmock::prelude::*;
use librcekunit::{Client, Config, CookieStore};
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
async fn test_dashboard_index_returns_ok() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET).path("/dashboard");
        then.status(200).body("main dashboard page");
    });
    let resp = client.dashboard_index().await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_dashboard_index_with_search_and_sort() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/dashboard")
            .query_param("search", "HONDA")
            .query_param("sort", "no")
            .query_param("direction", "desc");
        then.status(200);
    });
    let mut params = HashMap::new();
    params.insert("search".into(), "HONDA".into());
    params.insert("sort".into(), "no".into());
    params.insert("direction".into(), "desc".into());
    let resp = client.dashboard_index_with_params(params).await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_dashboard_index_server_error() {
    let (server, client) = setup().await;
    server.mock(|when, then| {
        when.method(GET).path("/dashboard");
        then.status(503);
    });
    let resp = client.dashboard_index().await.unwrap();
    assert_eq!(resp.status(), 503);
}

#[tokio::test]
async fn test_dashboard_delete_all_success() {
    let (server, client) = setup().await;
    client.http().set_csrf_token("tok".to_string()).await;
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/dashboard/delete-all")
            .body_includes("_method=DELETE");
        then.status(302);
    });
    let resp = client.dashboard_delete_all().await.unwrap();
    assert_eq!(resp.status(), 302);
    mock.assert();
}

#[tokio::test]
async fn test_dashboard_delete_all_failure() {
    let (server, client) = setup().await;
    client.http().set_csrf_token("tok".to_string()).await;
    server.mock(|when, then| {
        when.method(POST).path("/dashboard/delete-all");
        then.status(403).body("Forbidden");
    });
    let resp = client.dashboard_delete_all().await.unwrap();
    assert_eq!(resp.status(), 403);
}

#[tokio::test]
async fn test_delete_by_category_valid() {
    let (server, client) = setup().await;
    client.http().set_csrf_token("tok".to_string()).await;
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/dashboard/cekunit/delete-by-category")
            .body_includes("column=status")
            .body_includes("value=LUNAS");
        then.status(200).body("5 rows deleted");
    });
    let resp = client
        .cekunit_delete_by_category("status", "LUNAS")
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_delete_by_category_null_value() {
    let (server, client) = setup().await;
    client.http().set_csrf_token("tok".to_string()).await;
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/dashboard/cekunit/delete-by-category")
            .body_includes("value=null");
        then.status(200);
    });
    let resp = client
        .cekunit_delete_by_category("actual_penyelesaian", "null")
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_export_csv() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/dashboard/cekunit/export")
            .query_param("format", "csv")
            .query_param("sort", "no")
            .query_param("direction", "asc");
        then.status(200)
            .header("content-type", "text/csv")
            .body("header1,header2\nval1,val2");
    });
    let resp = client.cekunit_export("csv", "no", "asc").await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_export_invalid_format() {
    let (server, client) = setup().await;
    server.mock(|when, then| {
        when.method(GET).path("/dashboard/cekunit/export");
        then.status(400).body("Unsupported format");
    });
    let resp = client.cekunit_export("xml", "no", "asc").await.unwrap();
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn test_get_unique_values() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/dashboard/cekunit/get-unique-values")
            .query_param("column", "kategori");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"["A","B","C"]"#);
    });
    let resp = client.cekunit_get_unique_values("kategori").await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_get_unique_values_empty() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/dashboard/cekunit/get-unique-values")
            .query_param("column", "nonexistent");
        then.status(200)
            .header("content-type", "application/json")
            .body("[]");
    });
    let resp = client
        .cekunit_get_unique_values("nonexistent")
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}
