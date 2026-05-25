use httpmock::prelude::*;
use librcekunit::{Client, Config, Error};

fn setup() -> (MockServer, Client) {
    let server = MockServer::start();
    let config = Config::new(&server.base_url()).with_persistent_cookies(None);
    let client = tokio_test::block_on(Client::new(config)).unwrap();
    (server, client)
}

#[test]
fn test_login_success() {
    let (server, client) = setup();

    let _login_page_mock = server.mock(|when, then| {
        when.method(GET).path("/login");
        then.status(200)
            .append_header("set-cookie", "XSRF-TOKEN=dummy_xsrf; Path=/")
            .append_header("set-cookie", "session=initial_session; Path=/; HttpOnly")
            .body(r#"<html><body><form><input type="hidden" name="_token" value="abc123" /></form></body></html>"#);
    });

    let login_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/login")
            .body_contains("_token=abc123")
            .body_contains("email=test%40example.com")
            .body_contains("password=secret");
        then.status(302)
            .append_header("set-cookie", "session=logged_in_session; Path=/; HttpOnly")
            .append_header("set-cookie", "XSRF-TOKEN=new_xsrf; Path=/");
    });

    tokio_test::block_on(client.login("test@example.com", "secret")).expect("Login harus berhasil");

    let csrf = tokio_test::block_on(client.http().get_csrf_token());
    assert_eq!(csrf, Some("new_xsrf".to_string()));

    login_mock.assert();
}

#[test]
fn test_login_failure() {
    let (server, client) = setup();

    server.mock(|when, then| {
        when.method(GET).path("/login");
        then.status(200)
            .body(r#"<form><input name="_token" value="fail_token" /></form>"#);
    });

    let login_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/login")
            .body_contains("_token=fail_token");
        then.status(422)
            .body("These credentials do not match our records.");
    });

    let result = tokio_test::block_on(client.login("wrong@email.com", "wrong"));
    assert!(result.is_err());

    match result.unwrap_err() {
        Error::Auth(msg) => assert!(msg.contains("Login gagal")),
        _ => panic!("Harusnya error Auth"),
    }

    login_mock.assert();
}

#[test]
fn test_logout() {
    let (server, client) = setup();

    server.mock(|when, then| {
        when.method(GET).path("/login");
        then.status(200)
            .append_header("set-cookie", "XSRF-TOKEN=initial_xsrf; Path=/")
            .body(r#"<form><input name="_token" value="token_login" /></form>"#);
    });

    server.mock(|when, then| {
        when.method(POST)
            .path("/login")
            .body_contains("_token=token_login");
        then.status(302)
            .append_header("set-cookie", "session=logged_in; Path=/; HttpOnly")
            .append_header("set-cookie", "XSRF-TOKEN=logged_xsrf; Path=/");
    });

    let logout_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/logout")
            .body_contains("_token=logged_xsrf");
        then.status(302).append_header(
            "set-cookie",
            "session=deleted; Expires=Thu, 01 Jan 1970 00:00:00 GMT",
        );
    });

    tokio_test::block_on(client.login("test@example.com", "secret")).unwrap();

    let csrf_before = tokio_test::block_on(client.http().get_csrf_token());
    assert_eq!(csrf_before, Some("logged_xsrf".to_string()));

    tokio_test::block_on(client.logout()).expect("Logout harus berhasil");

    let csrf_after = tokio_test::block_on(client.http().get_csrf_token());
    assert!(
        csrf_after == None || csrf_after == Some(String::new()),
        "Token CSRF seharusnya kosong, tetapi ditemukan {:?}",
        csrf_after
    );

    logout_mock.assert();
}
