use httpmock::prelude::*;
use librcekunit::{Client, Config, CookieStore, Error};

async fn setup() -> (MockServer, Client) {
    let server = MockServer::start();
    let config = Config::new(&server.base_url())
        .with_user_agent("librcekunit-test/2.0")
        .with_timeout(5)
        .with_cookie_store(CookieStore::None);
    let client = Client::new(config).await.expect("Failed to create client");
    (server, client)
}

async fn perform_dummy_login(server: &MockServer, client: &Client, csrf_token: &str) {
    server.mock(|when, then| {
        when.method(GET).path("/login");
        then.status(200)
            .header("content-type", "text/html")
            .body(format!(
                r#"<html><body><form><input type="hidden" name="_token" value="{}" /></form></body></html>"#,
                csrf_token
            ));
    });

    server.mock(|when, then| {
        when.method(POST)
            .path("/login")
            .body_includes(format!("_token={}", csrf_token));
        then.status(302);
    });

    client.login("user@test.com", "pass").await.unwrap();
}

#[tokio::test]
async fn test_logout_success() {
    let (server, client) = setup().await;
    perform_dummy_login(&server, &client, "login_xsrf").await;

    let logout_mock = server.mock(|when, then| {
        when.method(POST).path("/logout").body("_token=");
        then.status(302);
    });

    let result = client.logout().await;
    assert!(result.is_ok());

    let csrf = client.http().get_csrf_token().await;
    assert!(csrf.is_none() || csrf == Some(String::new()));

    logout_mock.assert();
}

#[tokio::test]
async fn test_logout_failure_server_error() {
    let (server, client) = setup().await;
    perform_dummy_login(&server, &client, "tok").await;

    let logout_mock = server.mock(|when, then| {
        when.method(POST).path("/logout");
        then.status(500).body("Internal Server Error");
    });

    let result = client.logout().await;
    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Api(status, msg) => {
            assert_eq!(status, 500);
            assert!(msg.contains("Logout failed"));
        }
        other => panic!("Expected Error::Api, got {:?}", other),
    }
    logout_mock.assert();
}

#[tokio::test]
async fn test_logout_without_login_fetches_csrf_from_root() {
    let (server, client) = setup().await;

    let root_mock = server.mock(|when, then| {
        when.method(GET).path("/");
        then.status(200)
            .body(r#"<input name="_token" value="root_token">"#);
    });

    let logout_mock = server.mock(|when, then| {
        when.method(POST).path("/logout").body("_token=root_token");
        then.status(200);
    });

    let result = client.logout().await;
    assert!(result.is_ok());
    root_mock.assert();
    logout_mock.assert();
}

#[tokio::test]
async fn test_logout_with_prefilled_csrf_token() {
    let (server, client) = setup().await;

    client.http().set_csrf_token("my_token".to_string()).await;

    let logout_mock = server.mock(|when, then| {
        when.method(POST).path("/logout").body("_token=my_token");
        then.status(200);
    });

    let result = client.logout().await;
    assert!(result.is_ok());
    logout_mock.assert();
}

#[tokio::test]
async fn test_logout_network_timeout() {
    let (server, client) = setup().await;
    perform_dummy_login(&server, &client, "tok").await;

    let _logout_mock = server.mock(|when, then| {
        when.method(POST).path("/logout");
        then.status(302).delay(std::time::Duration::from_secs(10));
    });

    let result = client.logout().await;
    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Reqwest(e) => {
            assert!(e.is_timeout(), "Harusnya timeout");
        }
        other => panic!("Expected Reqwest timeout, got {:?}", other),
    }
}

#[tokio::test]
async fn test_logout_clears_session() {
    let (server, client) = setup().await;
    perform_dummy_login(&server, &client, "tok").await;

    let logout_mock = server.mock(|when, then| {
        when.method(POST).path("/logout");
        then.status(302);
    });

    client.logout().await.unwrap();
    assert!(
        client.http().get_csrf_token().await.is_none()
            || client.http().get_csrf_token().await == Some(String::new())
    );
    logout_mock.assert();
}
