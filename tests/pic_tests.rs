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
async fn test_pic_index_ok() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET).path("/pic");
        then.status(200).body("list of pic");
    });
    let resp = client.pic_index().await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_pic_store() {
    let (server, client) = setup().await;
    client.http().set_csrf_token("tok".to_string()).await;
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/pic")
            .body_includes("_token=tok")
            .body_includes("id_coll=COLL01")
            .body_includes("nama_collector=John+Doe");
        then.status(201);
    });
    let mut data = HashMap::new();
    data.insert("id_coll".into(), "COLL01".into());
    data.insert("nama_collector".into(), "John Doe".into());
    let resp = client.pic_store(data).await.unwrap();
    assert_eq!(resp.status(), 201);
    mock.assert();
}

#[tokio::test]
async fn test_pic_show() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET).path("/pic/5");
        then.status(200).body("detail pic");
    });
    let resp = client.pic_show(5).await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_pic_update() {
    let (server, client) = setup().await;
    client.http().set_csrf_token("tok".to_string()).await;
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/pic/3")
            .body_includes("_method=PUT")
            .body_includes("_token=tok")
            .body_includes("status=Aktif");
        then.status(302);
    });
    let mut data = HashMap::new();
    data.insert("status".into(), "Aktif".into());
    let resp = client.pic_update(3, data).await.unwrap();
    assert_eq!(resp.status(), 302);
    mock.assert();
}

#[tokio::test]
async fn test_pic_destroy() {
    let (server, client) = setup().await;
    client.http().set_csrf_token("tok".to_string()).await;
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/pic/7")
            .body_includes("_method=DELETE")
            .body_includes("_token=tok");
        then.status(302);
    });
    let resp = client.pic_destroy(7).await.unwrap();
    assert_eq!(resp.status(), 302);
    mock.assert();
}

#[tokio::test]
async fn test_dashboard_pic_index() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET).path("/dashboard/pic");
        then.status(200).body("dashboard pic page");
    });
    let resp = client.dashboard_pic_index().await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_dashboard_pic_index_with_page() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/dashboard/pic")
            .query_param("page", "2");
        then.status(200);
    });
    let mut params = HashMap::new();
    params.insert("page".into(), "2".into());
    let resp = client
        .dashboard_pic_index_with_params(params)
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_input_pic_create() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET).path("/dashboard/input-PIC");
        then.status(200).body("<form>input pic</form>");
    });
    let resp = client.input_pic_create().await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_input_pic_store() {
    let (server, client) = setup().await;
    client.http().set_csrf_token("tok".to_string()).await;
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/dashboard/input-PIC")
            .body_includes("_token=tok")
            .body_includes("id_coll=NEW01")
            .body_includes("nama_collector=New+Collector");
        then.status(302);
    });
    let mut data = HashMap::new();
    data.insert("id_coll".into(), "NEW01".into());
    data.insert("nama_collector".into(), "New Collector".into());
    let resp = client.input_pic_store(data).await.unwrap();
    assert_eq!(resp.status(), 302);
    mock.assert();
}
