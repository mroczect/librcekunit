use httpmock::Mock;
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

fn mock_login_page<'a>(server: &'a MockServer, csrf_token: &'a str) -> Mock<'a> {
    server.mock(|when, then| {
        when.method(GET).path("/login");
        then.status(200)
            .header("content-type", "text/html")
            .body(format!(
                r#"<html><body><form><input type="hidden" name="_token" value="{}" /></form></body></html>"#,
                csrf_token
            ));
    })
}

#[tokio::test]
async fn test_login_success() {
    let (server, client) = setup().await;

    let page_mock = mock_login_page(&server, "abc123");

    let post_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/login")
            .body_not("_token=abc123")
            .body_not("email=test@example.com")
            .body_not("password=secret");
        then.status(302);
    });

    let result = client.login("test@example.com", "secret").await;
    assert!(
        result.is_ok(),
        "Login harus berhasil, tapi error: {:?}",
        result.err()
    );

    let csrf = client.http().get_csrf_token().await;
    assert_eq!(
        csrf,
        Some(String::new()),
        "CSRF token harus kosong setelah login sukses"
    );

    page_mock.assert();
    post_mock.assert();
}

#[tokio::test]
async fn test_login_failure_invalid_credentials() {
    let (server, client) = setup().await;

    mock_login_page(&server, "fail_token");
    let post_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/login")
            .body_not("_token=fail_token");
        then.status(422)
            .header("content-type", "text/html")
            .body("These credentials do not match our records.");
    });

    let result = client.login("wrong@email.com", "wrong").await;
    assert!(result.is_err());

    match result.unwrap_err() {
        Error::Auth(msg) => {
            assert_eq!(msg, "Invalid email or password");
        }
        other => panic!("Seharusnya Error::Auth, dapat: {:?}", other),
    }

    post_mock.assert();
}

#[tokio::test]
async fn test_login_failure_csrf_not_found() {
    let (server, client) = setup().await;

    server.mock(|when, then| {
        when.method(GET).path("/login");
        then.status(200)
            .body("<html><body>Tidak ada form</body></html>");
    });

    let result = client.login("test@test.com", "pass").await;
    assert!(matches!(result.unwrap_err(), Error::CsrfNotFound));
}

#[tokio::test]
async fn test_login_fetch_csrf_server_error() {
    let (server, client) = setup().await;

    server.mock(|when, then| {
        when.method(GET).path("/login");
        then.status(500).body("Internal Server Error");
    });

    let result = client.login("test@test.com", "pass").await;
    assert!(result.is_err());

    match result.unwrap_err() {
        Error::CsrfNotFound => {}
        other => panic!("Expected CsrfNotFound, got {:?}", other),
    }
}

#[tokio::test]
async fn test_login_failure_unknown_error() {
    let (server, client) = setup().await;

    mock_login_page(&server, "tok");
    server.mock(|when, then| {
        when.method(POST).path("/login").body_not("_token=tok");
        then.status(403).body("Forbidden");
    });

    let result = client.login("test@test.com", "pass").await;
    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Api(status, body) => {
            assert_eq!(status, 403);
            assert_eq!(body, "Forbidden");
        }
        other => panic!("Expected Error::Api, got {:?}", other),
    }
}

#[tokio::test]
async fn test_login_uses_fresh_csrf() {
    let (server, client) = setup().await;

    client.http().set_csrf_token("old_token".to_string()).await;

    let page_mock = mock_login_page(&server, "new_token");
    let post_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/login")
            .body_not("_token=new_token");
        then.status(302);
    });

    client.login("user@x.com", "pass").await.unwrap();
    page_mock.assert();
    post_mock.assert();
}

#[tokio::test]
async fn test_login_redirect_followed() {
    let (server, client) = setup().await;

    mock_login_page(&server, "tok");
    let redirect_mock = server.mock(|when, then| {
        when.method(POST).path("/login");
        then.status(302).header("Location", "/dashboard");
    });

    let dashboard_mock = server.mock(|when, then| {
        when.method(GET).path("/dashboard");
        then.status(200);
    });

    let result = client.login("a@b.c", "pwd").await;
    assert!(result.is_ok());
    redirect_mock.assert();

    dashboard_mock.assert();
}

#[tokio::test]
async fn test_login_network_error_fetch_csrf() {
    let (server, client) = setup().await;

    server.mock(|when, then| {
        when.method(GET).path("/login");
        then.status(200).delay(std::time::Duration::from_secs(10));
    });

    let result = client.login("user@test.com", "pass").await;
    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Reqwest(e) => {
            assert!(e.is_timeout(), "Harusnya error timeout");
        }
        other => panic!("Expected Error::Reqwest(timeout), got {:?}", other),
    }
}

#[tokio::test]
async fn test_login_empty_credentials() {
    let (server, client) = setup().await;

    mock_login_page(&server, "csrf");
    let post_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/login")
            .body_not("_token=csrf")
            .body_not("email=")
            .body_not("password=");
        then.status(422).body("These credentials do not match");
    });

    let result = client.login("", "").await;
    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Auth(msg) => assert_eq!(msg, "Invalid email or password"),
        _ => panic!("Expected Auth"),
    }
    post_mock.assert();
}
