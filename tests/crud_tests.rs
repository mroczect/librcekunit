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
async fn test_cekunit_index_returns_ok() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET).path("/cekunit");
        then.status(200).body("list of cekunit");
    });
    let resp = client.cekunit_index().await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_cekunit_index_with_pagination() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET).path("/cekunit").query_param("page", "2");
        then.status(200);
    });
    let mut params = HashMap::new();
    params.insert("page".to_string(), "2".to_string());
    let resp = client.cekunit_index_with_params(params).await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_cekunit_index_server_error() {
    let (server, client) = setup().await;
    server.mock(|when, then| {
        when.method(GET).path("/cekunit");
        then.status(500).body("Internal Server Error");
    });
    let resp = client.cekunit_index().await.unwrap();
    assert_eq!(resp.status(), 500);
}

#[tokio::test]
async fn test_cekunit_create_returns_form() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET).path("/cekunit/create");
        then.status(200).body("<form>...</form>");
    });
    let resp = client.cekunit_create().await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_cekunit_store_with_valid_data() {
    let (server, client) = setup().await;
    client.http().set_csrf_token("csrf_store".to_string()).await;

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/cekunit")
            .body_includes("_token=csrf_store")
            .body_includes("nama_nasabah=John+Doe")
            .body_includes("nopol=B+1234+ABC");
        then.status(201);
    });

    let mut data = HashMap::new();
    data.insert("nama_nasabah".to_string(), "John Doe".to_string());
    data.insert("nopol".to_string(), "B 1234 ABC".to_string());
    let resp = client.cekunit_store(data).await.unwrap();
    assert_eq!(resp.status(), 201);
    mock.assert();
}

#[tokio::test]
async fn test_cekunit_store_without_csrf_auto_fetches() {
    let (server, client) = setup().await;
    let root_mock = server.mock(|when, then| {
        when.method(GET).path("/");
        then.status(200)
            .body(r#"<input name="_token" value="auto_token">"#);
    });
    let post_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/cekunit")
            .body_includes("_token=auto_token")
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
async fn test_cekunit_show_existing() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET).path("/cekunit/123");
        then.status(200).body("detail record");
    });
    let resp = client.cekunit_show(123).await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_cekunit_show_not_found() {
    let (server, client) = setup().await;
    server.mock(|when, then| {
        when.method(GET).path("/cekunit/9999");
        then.status(404).body("Not Found");
    });
    let resp = client.cekunit_show(9999).await.unwrap();
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_cekunit_edit_returns_form() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET).path("/cekunit/5/edit");
        then.status(200).body("<form>edit form</form>");
    });
    let resp = client.cekunit_edit(5).await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_cekunit_update_updates_record() {
    let (server, client) = setup().await;
    client
        .http()
        .set_csrf_token("csrf_update".to_string())
        .await;

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/cekunit/10")
            .body_includes("_token=csrf_update")
            .body_includes("_method=PUT")
            .body_includes("status=APPROVED");
        then.status(302);
    });

    let mut data = HashMap::new();
    data.insert("status".to_string(), "APPROVED".to_string());
    let resp = client.cekunit_update(10, data).await.unwrap();
    assert_eq!(resp.status(), 302);
    mock.assert();
}

#[tokio::test]
async fn test_cekunit_update_auto_adds_put_method() {
    let (server, client) = setup().await;
    client.http().set_csrf_token("tok".to_string()).await;

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/cekunit/2")
            .body_includes("_method=PUT");
        then.status(200);
    });

    let data = HashMap::new();
    let resp = client.cekunit_update(2, data).await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_cekunit_destroy_deletes_record() {
    let (server, client) = setup().await;
    client.http().set_csrf_token("csrf_del".to_string()).await;

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/cekunit/77")
            .body_includes("_token=csrf_del")
            .body_includes("_method=DELETE");
        then.status(302);
    });

    let resp = client.cekunit_destroy(77).await.unwrap();
    assert_eq!(resp.status(), 302);
    mock.assert();
}
