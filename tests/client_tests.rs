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
async fn test_dashboard_index_ok() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET).path("/dashboard");
        then.status(200).body("dashboard html");
    });
    let resp = client.dashboard_index().await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_dashboard_index_with_params() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/dashboard")
            .query_param("page", "1")
            .query_param("search", "test");
        then.status(200);
    });
    let mut params = HashMap::new();
    params.insert("page".into(), "1".into());
    params.insert("search".into(), "test".into());
    let resp = client.dashboard_index_with_params(params).await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_cekunit_index() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET).path("/cekunit");
        then.status(200).body("list");
    });
    let resp = client.cekunit_index().await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_cekunit_index_with_params() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/cekunit")
            .query_param("sort", "no")
            .query_param("direction", "asc");
        then.status(200);
    });
    let mut params = HashMap::new();
    params.insert("sort".into(), "no".into());
    params.insert("direction".into(), "asc".into());
    let resp = client.cekunit_index_with_params(params).await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_cekunit_create() {
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
async fn test_cekunit_store_sends_form_data() {
    let (server, client) = setup().await;
    client.http().set_csrf_token("test_csrf".to_string()).await;

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/cekunit")
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
            .path("/cekunit")
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
async fn test_cekunit_show() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET).path("/cekunit/42");
        then.status(200).body("detail");
    });
    let resp = client.cekunit_show(42).await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_cekunit_edit() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET).path("/cekunit/7/edit");
        then.status(200).body("<form>edit</form>");
    });
    let resp = client.cekunit_edit(7).await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_cekunit_update() {
    let (server, client) = setup().await;
    client
        .http()
        .set_csrf_token("update_csrf".to_string())
        .await;

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/cekunit/10")
            .body_includes("_token=update_csrf")
            .body_includes("_method=PUT")
            .body_includes("nama_nasabah=John");
        then.status(302);
    });

    let mut data = HashMap::new();
    data.insert("nama_nasabah".to_string(), "John".to_string());
    let resp = client.cekunit_update(10, data).await.unwrap();
    assert_eq!(resp.status(), 302);
    mock.assert();
}

#[tokio::test]
async fn test_cekunit_update_adds_method_if_missing() {
    let (server, client) = setup().await;
    client.http().set_csrf_token("csrf".to_string()).await;

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/cekunit/5")
            .body_includes("_method=PUT");
        then.status(200);
    });

    let data = HashMap::new();
    let resp = client.cekunit_update(5, data).await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_cekunit_destroy() {
    let (server, client) = setup().await;
    client.http().set_csrf_token("del_csrf".to_string()).await;

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/cekunit/99")
            .body_includes("_token=del_csrf")
            .body_includes("_method=DELETE");
        then.status(302);
    });

    let resp = client.cekunit_destroy(99).await.unwrap();
    assert_eq!(resp.status(), 302);
    mock.assert();
}

#[tokio::test]
async fn test_dashboard_delete_all() {
    let (server, client) = setup().await;
    client.http().set_csrf_token("csrf".to_string()).await;

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
async fn test_cekunit_delete_by_category() {
    let (server, client) = setup().await;
    client.http().set_csrf_token("cat_csrf".to_string()).await;

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/dashboard/cekunit/delete-by-category")
            .body_includes("_token=cat_csrf")
            .body_includes("column=kategori")
            .body_includes("value=A");
        then.status(200).body("deleted");
    });

    let resp = client
        .cekunit_delete_by_category("kategori", "A")
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_cekunit_delete_by_category_null_value() {
    let (server, client) = setup().await;
    client.http().set_csrf_token("csrf".to_string()).await;

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/dashboard/cekunit/delete-by-category")
            .body_includes("value=null");
        then.status(200);
    });

    let resp = client
        .cekunit_delete_by_category("status", "null")
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_cekunit_export() {
    let (server, client) = setup().await;

    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/dashboard/cekunit/export")
            .query_param("format", "csv")
            .query_param("sort", "no")
            .query_param("direction", "asc");
        then.status(200)
            .header("content-type", "text/csv")
            .body("col1,col2\nval1,val2");
    });

    let resp = client.cekunit_export("csv", "no", "asc").await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_cekunit_get_unique_values() {
    let (server, client) = setup().await;

    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/dashboard/cekunit/get-unique-values")
            .query_param("column", "kategori");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"["A","B","C","D"]"#);
    });

    let resp = client.cekunit_get_unique_values("kategori").await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_users_index_ok() {
    let (server, client) = setup().await;

    let mock = server.mock(|when, then| {
        when.method(GET).path("/users");
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
