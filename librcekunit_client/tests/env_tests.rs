#![allow(unused_crate_dependencies)]

use core::error::Error;

use librcekunit_client::ClientBuilder;
use librcekunit_client::env::{
    ENV_BASE_URL, ENV_COOKIE_FILE, ENV_EMAIL, ENV_PASSWORD, ENV_TIMEOUT_SECS, ENV_USER_AGENT,
    from_env_with, login_from_env_with,
};
use librcekunit_handler::{Auth, CookieStore};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn make_lookup<'a>(entries: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
    move |key| {
        entries
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| String::from(*v))
    }
}

#[tokio::test]
async fn from_env_with_minimal_entries_succeeds() -> Result<(), Box<dyn Error>> {
    let lookup = make_lookup(&[(ENV_BASE_URL, "http://127.0.0.1:1")]);
    let _client = from_env_with(lookup).await?;
    Ok(())
}

#[tokio::test]
async fn from_env_with_missing_base_url_errors() {
    let lookup = make_lookup(&[]);
    let r = from_env_with(lookup).await;
    assert!(r.is_err());
}

#[tokio::test]
async fn from_env_with_persistent_cookie_store() -> Result<(), Box<dyn Error>> {
    let lookup = make_lookup(&[
        (ENV_BASE_URL, "http://127.0.0.1:1"),
        (ENV_COOKIE_FILE, "/tmp/librcekunit_env_test.json"),
    ]);
    let _client = from_env_with(lookup).await?;
    Ok(())
}

#[tokio::test]
async fn from_env_with_custom_user_agent() -> Result<(), Box<dyn Error>> {
    let lookup = make_lookup(&[
        (ENV_BASE_URL, "http://127.0.0.1:1"),
        (ENV_USER_AGENT, "env-ua/1"),
    ]);
    let _client = from_env_with(lookup).await?;
    Ok(())
}

#[tokio::test]
async fn from_env_with_valid_timeout() -> Result<(), Box<dyn Error>> {
    let lookup = make_lookup(&[
        (ENV_BASE_URL, "http://127.0.0.1:1"),
        (ENV_TIMEOUT_SECS, "45"),
    ]);
    let _client = from_env_with(lookup).await?;
    Ok(())
}

#[tokio::test]
async fn from_env_with_invalid_timeout_errors() {
    let lookup = make_lookup(&[
        (ENV_BASE_URL, "http://127.0.0.1:1"),
        (ENV_TIMEOUT_SECS, "not-a-number"),
    ]);
    let r = from_env_with(lookup).await;
    assert!(r.is_err());
}

#[tokio::test]
async fn from_env_with_zero_timeout_still_builds_client() -> Result<(), Box<dyn Error>> {
    let lookup = make_lookup(&[
        (ENV_BASE_URL, "http://127.0.0.1:1"),
        (ENV_TIMEOUT_SECS, "0"),
    ]);
    let _client = from_env_with(lookup).await?;
    Ok(())
}

#[tokio::test]
async fn login_from_env_with_missing_email_errors() -> Result<(), Box<dyn Error>> {
    let client = ClientBuilder::new()
        .base_url("http://127.0.0.1:1")
        .cookie_store(CookieStore::Memory)
        .build()
        .await?;
    let lookup = make_lookup(&[(ENV_PASSWORD, "secret")]);
    let r = login_from_env_with(&client, lookup).await;
    assert!(r.is_err());
    Ok(())
}

#[tokio::test]
async fn login_from_env_with_missing_password_errors() -> Result<(), Box<dyn Error>> {
    let client = ClientBuilder::new()
        .base_url("http://127.0.0.1:1")
        .cookie_store(CookieStore::Memory)
        .build()
        .await?;
    let lookup = make_lookup(&[(ENV_EMAIL, "a@b.com")]);
    let r = login_from_env_with(&client, lookup).await;
    assert!(r.is_err());
    Ok(())
}

#[tokio::test]
async fn login_from_env_with_all_set_succeeds() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/login"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(r#"<input name="_token" value="tok-env">"#),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/login"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = ClientBuilder::new()
        .base_url(server.uri())
        .cookie_store(CookieStore::Memory)
        .build()
        .await?;

    let lookup = make_lookup(&[(ENV_EMAIL, "user@example.com"), (ENV_PASSWORD, "hunter2")]);
    login_from_env_with(&client, lookup).await?;
    Ok(())
}

#[tokio::test]
async fn from_env_with_all_set_via_mock_server() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    let uri = server.uri();
    let entries: [(&str, &str); 3] = [
        (ENV_BASE_URL, uri.as_str()),
        (ENV_USER_AGENT, "full/1"),
        (ENV_TIMEOUT_SECS, "20"),
    ];
    let lookup = make_lookup(&entries);
    let client = from_env_with(lookup).await?;
    let debug = format!("{:?}", client);
    assert!(debug.contains("Client"));
    Ok(())
}

#[test]
fn env_const_names_are_stable() {
    assert_eq!(ENV_BASE_URL, "LIBRCEKUNIT_BASE_URL");
    assert_eq!(ENV_COOKIE_FILE, "LIBRCEKUNIT_COOKIE_FILE");
    assert_eq!(ENV_USER_AGENT, "LIBRCEKUNIT_USER_AGENT");
    assert_eq!(ENV_TIMEOUT_SECS, "LIBRCEKUNIT_TIMEOUT_SECS");
    assert_eq!(ENV_EMAIL, "LIBRCEKUNIT_EMAIL");
    assert_eq!(ENV_PASSWORD, "LIBRCEKUNIT_PASSWORD");
}

#[test]
fn env_const_names_are_unique() {
    let all = [
        ENV_BASE_URL,
        ENV_COOKIE_FILE,
        ENV_USER_AGENT,
        ENV_TIMEOUT_SECS,
        ENV_EMAIL,
        ENV_PASSWORD,
    ];
    for (i, a) in all.iter().enumerate() {
        for b in all.iter().skip(i.saturating_add(1)) {
            assert_ne!(a, b);
        }
    }
}

#[tokio::test]
async fn client_implements_auth_trait() -> Result<(), Box<dyn Error>> {
    fn assert_auth<T: Auth>() {}
    assert_auth::<librcekunit_client::Client>();
    let _ = ClientBuilder::new().build().await;
    Ok(())
}
