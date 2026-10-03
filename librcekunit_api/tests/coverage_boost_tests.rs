#![allow(unused_crate_dependencies)]

use core::error::Error;
use std::path::PathBuf;

use librcekunit_api::Client;
use librcekunit_handler::{
    Auth, Config, CookieStore, Crud, Error as HandlerError, Form, HttpMethod, Transport,
};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const CSRF_HTML: &str = r#"<input name="_token" value="tok-boost">"#;

fn tmp_path(tag: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("librcekunit_boost_{tag}.csv"));
    let _ = std::fs::remove_file(&p);
    p
}

async fn server_with_csrf() -> Result<MockServer, Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dashboard"))
        .respond_with(ResponseTemplate::new(200).set_body_string(CSRF_HTML))
        .mount(&server)
        .await;
    Ok(server)
}

async fn client_for(server: &MockServer) -> Result<Client, Box<dyn Error>> {
    let config = Config::new(&server.uri()).with_cookie_store(CookieStore::Memory);
    Ok(Client::new(config).await?)
}

#[tokio::test]
async fn login_500_returns_api_error() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/login"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"<input name="_token" value="login-tok">"#),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/login"))
        .respond_with(ResponseTemplate::new(500).set_body_string("server down"))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let result = client.login("u@e.com", "p").await;

    match result {
        Err(HandlerError::Api(500, body)) => {
            assert_eq!(body, "server down");
        }
        other => return Err(format!("expected Api(500), got {other:?}").into()),
    }
    Ok(())
}

#[tokio::test]
async fn login_401_returns_api_error() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/login"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"<input name="_token" value="login-tok">"#),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/login"))
        .respond_with(ResponseTemplate::new(401).set_body_string("unauthorized"))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let result = client.login("u@e.com", "bad").await;
    assert!(matches!(result, Err(HandlerError::Api(401, _))));
    Ok(())
}

#[tokio::test]
async fn login_302_redirect_is_ok() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/login"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"<input name="_token" value="login-tok">"#),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/dashboard"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/login"))
        .respond_with(ResponseTemplate::new(302).insert_header("location", "/dashboard"))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    client.login("u@e.com", "p").await?;
    assert!(client.http().get_csrf().await.is_none());
    Ok(())
}

#[tokio::test]
async fn login_303_redirect_is_ok() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/login"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"<input name="_token" value="login-tok">"#),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/dashboard"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/login"))
        .respond_with(ResponseTemplate::new(303).insert_header("location", "/dashboard"))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    client.login("u@e.com", "p").await?;
    Ok(())
}

#[tokio::test]
async fn logout_500_returns_api_error() -> Result<(), Box<dyn Error>> {
    let server = server_with_csrf().await?;
    Mock::given(method("POST"))
        .and(path("/logout"))
        .respond_with(ResponseTemplate::new(500).set_body_string("boom"))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let result = client.logout().await;
    assert!(matches!(result, Err(HandlerError::Api(500, _))));
    Ok(())
}

#[tokio::test]
async fn logout_403_returns_api_error() -> Result<(), Box<dyn Error>> {
    let server = server_with_csrf().await?;
    Mock::given(method("POST"))
        .and(path("/logout"))
        .respond_with(ResponseTemplate::new(403).set_body_string("forbidden"))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let result = client.logout().await;
    assert!(matches!(result, Err(HandlerError::Api(403, _))));
    Ok(())
}

#[tokio::test]
async fn logout_302_is_ok_and_clears_csrf() -> Result<(), Box<dyn Error>> {
    let server = server_with_csrf().await?;
    Mock::given(method("GET"))
        .and(path("/login"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/logout"))
        .respond_with(ResponseTemplate::new(302).insert_header("location", "/login"))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let _ = client.store(Form::new()).await?;
    assert!(client.http().get_csrf().await.is_some());

    client.logout().await?;
    assert!(client.http().get_csrf().await.is_none());
    Ok(())
}

#[tokio::test]
async fn import_file_nonexistent_returns_io_error() -> Result<(), Box<dyn Error>> {
    let server = server_with_csrf().await?;
    let client = client_for(&server).await?;
    let path = std::path::Path::new("/definitely/not/here.csv");
    let r = librcekunit_api::endpoints::input_user::import_file(client.http(), path).await;
    assert!(matches!(r, Err(HandlerError::Io(_))));
    Ok(())
}

#[tokio::test]
async fn import_file_with_valid_file_but_server_500() -> Result<(), Box<dyn Error>> {
    let server = server_with_csrf().await?;
    Mock::given(method("POST"))
        .and(path("/dashboard/input_user/insert"))
        .respond_with(ResponseTemplate::new(500).set_body_string("upload failed"))
        .mount(&server)
        .await;

    let path = tmp_path("valid_500");
    std::fs::write(&path, "no,foo\n1,bar\n")?;

    let client = client_for(&server).await?;
    let resp = librcekunit_api::endpoints::input_user::import_file(client.http(), &path).await?;
    assert_eq!(resp.status(), 500);

    let _ = std::fs::remove_file(&path);
    Ok(())
}

#[tokio::test]
async fn import_file_when_csrf_endpoint_500() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dashboard"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let path = tmp_path("csrf_fail");
    std::fs::write(&path, "no,foo\n1,bar\n")?;

    let client = client_for(&server).await?;
    let r = librcekunit_api::endpoints::input_user::import_file(client.http(), &path).await;
    assert!(r.is_err());

    let _ = std::fs::remove_file(&path);
    Ok(())
}

#[tokio::test]
async fn clear_session_when_no_cookie_file_is_ok() -> Result<(), Box<dyn Error>> {
    let config = Config::new("http://127.0.0.1:1").with_cookie_store(CookieStore::Memory);
    let http = librcekunit_api::HttpClient::new(&config).await?;
    http.set_csrf(Some(String::from("tok"))).await;
    http.clear_session().await;
    assert!(http.get_csrf().await.is_none());
    Ok(())
}

#[tokio::test]
async fn clear_session_when_cookie_file_missing() -> Result<(), Box<dyn Error>> {
    let path = tmp_path("clear_missing");
    let config =
        Config::new("http://127.0.0.1:1").with_cookie_store(CookieStore::Persistent(path.clone()));
    let http = librcekunit_api::HttpClient::new(&config).await?;
    http.set_csrf(Some(String::from("tok"))).await;
    http.clear_session().await;
    assert!(http.get_csrf().await.is_none());
    assert!(!path.exists());
    Ok(())
}

#[tokio::test]
async fn clear_session_when_cookie_file_exists() -> Result<(), Box<dyn Error>> {
    let path = tmp_path("clear_exists");
    std::fs::write(&path, "[]")?;
    assert!(path.exists());

    let config =
        Config::new("http://127.0.0.1:1").with_cookie_store(CookieStore::Persistent(path.clone()));
    let http = librcekunit_api::HttpClient::new(&config).await?;
    http.clear_session().await;
    assert!(!path.exists());
    Ok(())
}

#[tokio::test]
async fn transport_request_with_options_method() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("OPTIONS"))
        .and(path("/cekunit"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let _ = client
        .http()
        .request(HttpMethod::OPTIONS, "/cekunit", None)
        .await?;
    Ok(())
}

#[tokio::test]
async fn transport_request_with_head_method() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("HEAD"))
        .and(path("/cekunit"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let _ = client
        .http()
        .request(HttpMethod::HEAD, "/cekunit", None)
        .await?;
    Ok(())
}

#[tokio::test]
async fn transport_request_with_put_method() -> Result<(), Box<dyn Error>> {
    let server = server_with_csrf().await?;
    Mock::given(method("PUT"))
        .and(path("/cekunit/1"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let _ = client
        .http()
        .request(HttpMethod::PUT, "/cekunit/1", Some(Form::new()))
        .await?;
    Ok(())
}

#[tokio::test]
async fn transport_request_with_patch_method() -> Result<(), Box<dyn Error>> {
    let server = server_with_csrf().await?;
    Mock::given(method("PATCH"))
        .and(path("/cekunit/1"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let _ = client
        .http()
        .request(HttpMethod::PATCH, "/cekunit/1", Some(Form::new()))
        .await?;
    Ok(())
}

#[tokio::test]
async fn transport_request_with_delete_method() -> Result<(), Box<dyn Error>> {
    let server = server_with_csrf().await?;
    Mock::given(method("DELETE"))
        .and(path("/cekunit/1"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let _ = client
        .http()
        .request(HttpMethod::DELETE, "/cekunit/1", Some(Form::new()))
        .await?;
    Ok(())
}

#[tokio::test]
async fn fetch_csrf_no_token_returns_error() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dashboard"))
        .respond_with(ResponseTemplate::new(200).set_body_string("<html></html>"))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let r = client.http().fetch_csrf_token("/dashboard").await;
    assert!(matches!(r, Err(HandlerError::CsrfNotFound)));
    Ok(())
}

#[tokio::test]
async fn fetch_csrf_from_login_path() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/login"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"<input name="_token" value="from-login">"#),
        )
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let token = client.http().fetch_csrf_token("/login").await?;
    assert_eq!(token, "from-login");
    Ok(())
}
