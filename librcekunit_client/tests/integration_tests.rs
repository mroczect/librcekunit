#![allow(unused_crate_dependencies)]

use core::error::Error as StdError;

use librcekunit_client::prelude::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const CSRF_HTML: &str = r#"<input name="_token" value="tok-flow">"#;

#[tokio::test]
async fn scenario_builder_to_login_to_crud() -> Result<(), Box<dyn StdError>> {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/login"))
        .respond_with(ResponseTemplate::new(200).set_body_string(CSRF_HTML))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/login"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/dashboard"))
        .respond_with(ResponseTemplate::new(200).set_body_string(CSRF_HTML))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/cekunit"))
        .respond_with(ResponseTemplate::new(200).set_body_string("list"))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/cekunit"))
        .respond_with(ResponseTemplate::new(200).set_body_string("created"))
        .mount(&server)
        .await;

    let client = ClientBuilder::new()
        .base_url(server.uri())
        .cookie_store(CookieStore::Memory)
        .build()
        .await?;

    client.login("u@e.com", "p").await?;

    let list = client.index().await?;
    assert_eq!(list.text().await?, "list");

    let mut data = Form::new();
    let _ = data.insert(String::from("nama_nasabah"), String::from("ALICE"));
    let created = client.store(data).await?;
    assert_eq!(created.text().await?, "created");

    Ok(())
}

#[tokio::test]
async fn scenario_env_to_client_to_store() -> Result<(), Box<dyn StdError>> {
    let server = MockServer::start().await;
    let uri = server.uri();

    Mock::given(method("GET"))
        .and(path("/dashboard"))
        .respond_with(ResponseTemplate::new(200).set_body_string(CSRF_HTML))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/cekunit"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let uri_for_lookup = uri.clone();
    let lookup = move |key: &str| match key {
        "LIBRCEKUNIT_BASE_URL" => Some(uri_for_lookup.clone()),
        "LIBRCEKUNIT_USER_AGENT" => Some(String::from("env-test/1")),
        _ => None,
    };

    let client = from_env_with(lookup).await?;

    let _ = client.store(Form::new()).await?;
    Ok(())
}

#[tokio::test]
async fn scenario_builder_clone_produces_independent_clients() -> Result<(), Box<dyn StdError>> {
    let server_a = MockServer::start().await;
    let server_b = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/cekunit"))
        .respond_with(ResponseTemplate::new(200).set_body_string("A"))
        .mount(&server_a)
        .await;

    Mock::given(method("GET"))
        .and(path("/cekunit"))
        .respond_with(ResponseTemplate::new(200).set_body_string("B"))
        .mount(&server_b)
        .await;

    let builder_a = ClientBuilder::new()
        .base_url(server_a.uri())
        .cookie_store(CookieStore::Memory);

    let builder_b = ClientBuilder::new()
        .base_url(server_b.uri())
        .cookie_store(CookieStore::Memory);

    let client_a = builder_a.build().await?;
    let client_b = builder_b.build().await?;

    let resp_a = client_a.index().await?;
    let resp_b = client_b.index().await?;

    assert_eq!(resp_a.text().await?, "A");
    assert_eq!(resp_b.text().await?, "B");

    Ok(())
}

#[tokio::test]
async fn scenario_prelude_imports_all_needed_types() -> Result<(), Box<dyn StdError>> {
    fn assert_bounds<T: Auth + Crud + Dashboard + InputData + InputUser + Pic + Users>() {}
    assert_bounds::<Client>();

    let server = MockServer::start().await;
    let _client = ClientBuilder::new()
        .base_url(server.uri())
        .cookie_store(CookieStore::None)
        .build()
        .await?;

    let _form: Form = Form::new();
    let err: Error = Error::NotLoggedIn;
    assert!(matches!(err, Error::NotLoggedIn));
    Ok(())
}

#[tokio::test]
async fn scenario_env_missing_base_url_gives_config_error() -> Result<(), Box<dyn StdError>> {
    let lookup = |_key: &str| Option::<String>::None;
    let r = from_env_with(lookup).await;
    assert!(matches!(r, Err(Error::Config(_))));
    Ok(())
}

#[tokio::test]
async fn scenario_login_from_env_missing_credentials_error() -> Result<(), Box<dyn StdError>> {
    let server = MockServer::start().await;
    let client = ClientBuilder::new()
        .base_url(server.uri())
        .cookie_store(CookieStore::Memory)
        .build()
        .await?;

    let lookup = |_key: &str| Option::<String>::None;
    let r = login_from_env_with(&client, lookup).await;
    assert!(matches!(r, Err(Error::Config(_))));
    Ok(())
}

#[tokio::test]
async fn scenario_full_credentials_from_env_success() -> Result<(), Box<dyn StdError>> {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/login"))
        .respond_with(ResponseTemplate::new(200).set_body_string(CSRF_HTML))
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

    let lookup = |key: &str| match key {
        "LIBRCEKUNIT_EMAIL" => Some(String::from("admin@example.com")),
        "LIBRCEKUNIT_PASSWORD" => Some(String::from("hunter2")),
        _ => None,
    };

    login_from_env_with(&client, lookup).await?;
    Ok(())
}
