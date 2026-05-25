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
async fn test_input_user_index_returns_ok() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET).path("/dashboard/input-user");
        then.status(200).body("table page");
    });
    let resp = client.input_user_index().await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_input_user_index_with_params() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/dashboard/input-user")
            .query_param("page", "2")
            .query_param("search", "Honda")
            .query_param("sort", "nopol");
        then.status(200);
    });
    let mut params = HashMap::new();
    params.insert("page".into(), "2".into());
    params.insert("search".into(), "Honda".into());
    params.insert("sort".into(), "nopol".into());
    let resp = client.input_user_index_with_params(params).await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_input_user_index_with_date_range() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/dashboard/input-user")
            .query_param("start_date", "2026-04-01")
            .query_param("end_date", "2026-05-31")
            .query_param("sort", "created_at");
        then.status(200);
    });
    let mut params = HashMap::new();
    params.insert("start_date".into(), "2026-04-01".into());
    params.insert("end_date".into(), "2026-05-31".into());
    params.insert("sort".into(), "created_at".into());
    let resp = client.input_user_index_with_params(params).await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_input_user_create() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET).path("/dashboard/input_user/create");
        then.status(200).body("<form>...</form>");
    });
    let resp = client.input_user_create().await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_input_user_store() {
    let (server, client) = setup().await;
    client.http().set_csrf_token("tok".to_string()).await;
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/dashboard/input_user")
            .body_includes("_token=tok")
            .body_includes("nopol=BP1234XX");
        then.status(201);
    });
    let mut data = HashMap::new();
    data.insert("nopol".to_string(), "BP1234XX".to_string());
    let resp = client.input_user_store(data).await.unwrap();
    assert_eq!(resp.status(), 201);
    mock.assert();
}

#[tokio::test]
async fn test_input_user_show() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET).path("/dashboard/input_user/42");
        then.status(200).body("detail");
    });
    let resp = client.input_user_show(42).await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_input_user_edit() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET).path("/dashboard/input_user/7/edit");
        then.status(200).body("edit form");
    });
    let resp = client.input_user_edit(7).await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_input_user_update() {
    let (server, client) = setup().await;
    client.http().set_csrf_token("tok".to_string()).await;
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/dashboard/input_user/10")
            .body_includes("_method=PUT")
            .body_includes("_token=tok")
            .body_includes("lokasi=New+Location");
        then.status(302);
    });
    let mut data = HashMap::new();
    data.insert("lokasi".to_string(), "New Location".to_string());
    let resp = client.input_user_update(10, data).await.unwrap();
    assert_eq!(resp.status(), 302);
    mock.assert();
}

#[tokio::test]
async fn test_input_user_destroy() {
    let (server, client) = setup().await;
    client.http().set_csrf_token("tok".to_string()).await;
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/dashboard/input_user/99")
            .body_includes("_method=DELETE")
            .body_includes("_token=tok");
        then.status(302);
    });
    let resp = client.input_user_destroy(99).await.unwrap();
    assert_eq!(resp.status(), 302);
    mock.assert();
}

#[tokio::test]
async fn test_input_user_export() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/dashboard/input_user/export")
            .query_param("format", "csv")
            .query_param("sort", "id");
        then.status(200).body("csv data");
    });
    let mut params = HashMap::new();
    params.insert("format".into(), "csv".into());
    params.insert("sort".into(), "id".into());
    let resp = client.input_user_export(params).await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_input_user_export_with_date_range() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/dashboard/input_user/export")
            .query_param("start_date", "2026-04-01")
            .query_param("end_date", "2026-04-30");
        then.status(200);
    });
    let mut params = HashMap::new();
    params.insert("format".into(), "csv".into());
    params.insert("start_date".into(), "2026-04-01".into());
    params.insert("end_date".into(), "2026-04-30".into());
    let resp = client.input_user_export(params).await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_input_user_import() {
    let (server, client) = setup().await;
    client.http().set_csrf_token("tok".to_string()).await;
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/dashboard/input_user/insert")
            .body_includes("_token=tok")
            .body_includes("file=...");
        then.status(200).body("imported");
    });
    let mut data = HashMap::new();
    data.insert("file".to_string(), "...".to_string());
    let resp = client.input_user_import(data).await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}
