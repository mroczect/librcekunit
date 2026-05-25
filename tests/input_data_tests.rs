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
async fn test_input_data_create_returns_form() {
    let (server, client) = setup().await;
    let mock = server.mock(|when, then| {
        when.method(GET).path("/dashboard/input-data");
        then.status(200).body("<form>input data form</form>");
    });
    let resp = client.input_data_create().await.unwrap();
    assert_eq!(resp.status(), 200);
    mock.assert();
}

#[tokio::test]
async fn test_input_data_store_with_valid_data() {
    let (server, client) = setup().await;
    client.http().set_csrf_token("tok_input".to_string()).await;

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/dashboard/input-data")
            .body_includes("_token=tok_input")
            .body_includes("no_perjanjian=OD-25-F0000123")
            .body_includes("nama_nasabah=John+Doe")
            .body_includes("nopol=BP1234CD");
        then.status(302);
    });

    let mut data = HashMap::new();
    data.insert("no_perjanjian".into(), "OD-25-F0000123".into());
    data.insert("nama_nasabah".into(), "John Doe".into());
    data.insert("nopol".into(), "BP1234CD".into());

    let resp = client.input_data_store(data).await.unwrap();
    assert_eq!(resp.status(), 302);
    mock.assert();
}

#[tokio::test]
async fn test_input_data_store_without_csrf_auto_fetches() {
    let (server, client) = setup().await;

    let root_mock = server.mock(|when, then| {
        when.method(GET).path("/");
        then.status(200)
            .body(r#"<input name="_token" value="auto_csrf">"#);
    });

    let post_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/dashboard/input-data")
            .body_includes("_token=auto_csrf")
            .body_includes("no_perjanjian=OD-25-F0000001");
        then.status(302);
    });

    let mut data = HashMap::new();
    data.insert("no_perjanjian".into(), "OD-25-F0000001".into());

    let resp = client.input_data_store(data).await.unwrap();
    assert_eq!(resp.status(), 302);
    root_mock.assert();
    post_mock.assert();
}

#[tokio::test]
async fn test_input_data_store_server_error() {
    let (server, client) = setup().await;
    client.http().set_csrf_token("tok".to_string()).await;

    server.mock(|when, then| {
        when.method(POST).path("/dashboard/input-data");
        then.status(422).body("validation errors");
    });

    let mut data = HashMap::new();
    data.insert("no_perjanjian".into(), "".into());

    let resp = client.input_data_store(data).await.unwrap();
    assert_eq!(resp.status(), 422);
}
