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
async fn test_users_index_ok() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET).path("/users");
        then.status(200).body("list of users");
    });
    let resp = client.users_index().await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_users_store() {
    let (server, client) = setup().await;
    client.http().set_csrf_token("tok".to_string()).await;
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/users")
            .body_includes("_token=tok")
            .body_includes("nama=John+Doe")
            .body_includes("no_wa=08123456789");
        then.status(201);
    });
    let mut data = HashMap::new();
    data.insert("nama".into(), "John Doe".into());
    data.insert("no_wa".into(), "08123456789".into());
    let resp = client.users_store(data).await.unwrap();
    assert_eq!(resp.status(), 201);
    mock.assert();
}

#[tokio::test]
async fn test_users_show() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET).path("/users/5");
        then.status(200).body("detail user");
    });
    let resp = client.users_show(5).await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_users_update() {
    let (server, client) = setup().await;
    client.http().set_csrf_token("tok".to_string()).await;
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/users/3")
            .body_includes("_method=PUT")
            .body_includes("_token=tok")
            .body_includes("status=Karyawan");
        then.status(302);
    });
    let mut data = HashMap::new();
    data.insert("status".into(), "Karyawan".into());
    let resp = client.users_update(3, data).await.unwrap();
    assert_eq!(resp.status(), 302);
    mock.assert();
}

#[tokio::test]
async fn test_users_destroy() {
    let (server, client) = setup().await;
    client.http().set_csrf_token("tok".to_string()).await;
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/users/7")
            .body_includes("_method=DELETE")
            .body_includes("_token=tok");
        then.status(302);
    });
    let resp = client.users_destroy(7).await.unwrap();
    assert_eq!(resp.status(), 302);
    mock.assert();
}

#[tokio::test]
async fn test_dashboard_users_index() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET).path("/dashboard/users");
        then.status(200).body("dashboard users page");
    });
    let resp = client.dashboard_users_index().await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_dashboard_users_index_with_page() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/dashboard/users")
            .query_param("page", "2");
        then.status(200);
    });
    let mut params = HashMap::new();
    params.insert("page".into(), "2".into());
    let resp = client
        .dashboard_users_index_with_params(params)
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}
