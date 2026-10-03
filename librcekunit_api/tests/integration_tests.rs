#![allow(unused_crate_dependencies)]

use core::error::Error;
use std::path::PathBuf;

use librcekunit_api::{Client, HttpClient};
use librcekunit_handler::{
    Auth, Config, CookieStore, Crud, Dashboard, Form, HttpMethod, Transport,
};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const CSRF_HTML: &str = r#"<form><input name="_token" value="csrf-fixed"></form>"#;

fn tmp_cookies_path(tag: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "librcekunit_it_{}_{}.json",
        std::process::id(),
        tag
    ));
    let _ = std::fs::remove_file(&path);
    path
}

fn body_of(req: &wiremock::Request) -> String {
    String::from_utf8_lossy(&req.body).to_string()
}

fn post_bodies(reqs: &[wiremock::Request]) -> Vec<String> {
    reqs.iter()
        .filter(|r| r.method.as_str() == "POST")
        .map(body_of)
        .collect()
}

async fn csrf_endpoint(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/dashboard"))
        .respond_with(ResponseTemplate::new(200).set_body_string(CSRF_HTML))
        .mount(server)
        .await;
}

#[tokio::test]
async fn scenario_full_login_flow() -> Result<(), Box<dyn Error>> {
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
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let config = Config::new(&server.uri()).with_cookie_store(CookieStore::Memory);
    let client = Client::new(config).await?;

    client.login("admin@campus.edu", "hunter2").await?;

    let reqs = server.received_requests().await.unwrap_or_default();
    let get = reqs
        .iter()
        .find(|r| r.method.as_str() == "GET")
        .ok_or("no GET")?;
    assert_eq!(get.url.path(), "/login");
    let post_bodies = post_bodies(&reqs);
    assert_eq!(post_bodies.len(), 1);
    let body = post_bodies.first().ok_or("no POST body")?;
    assert!(body.contains("_token=login-tok"));
    assert!(body.contains("email=admin%40campus.edu"));
    assert!(body.contains("password=hunter2"));
    Ok(())
}

#[tokio::test]
async fn scenario_csrf_fetched_once_then_reused() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    csrf_endpoint(&server).await;
    Mock::given(method("POST"))
        .and(path("/cekunit"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let config = Config::new(&server.uri()).with_cookie_store(CookieStore::Memory);
    let client = Client::new(config).await?;

    let _ = client.store(Form::new()).await?;
    let _ = client.store(Form::new()).await?;
    let _ = client.store(Form::new()).await?;

    let reqs = server.received_requests().await.unwrap_or_default();
    let get_count = reqs.iter().filter(|r| r.method.as_str() == "GET").count();
    let post_count = reqs.iter().filter(|r| r.method.as_str() == "POST").count();
    assert_eq!(get_count, 1);
    assert_eq!(post_count, 3);

    for body in post_bodies(&reqs) {
        assert!(body.contains("_token=csrf-fixed"));
    }
    Ok(())
}

#[tokio::test]
async fn scenario_csrf_reset_triggers_refetch() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    csrf_endpoint(&server).await;
    Mock::given(method("POST"))
        .and(path("/cekunit"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let config = Config::new(&server.uri()).with_cookie_store(CookieStore::Memory);
    let client = Client::new(config).await?;

    let _ = client.store(Form::new()).await?;
    client.http().reset_csrf_token().await;
    let _ = client.store(Form::new()).await?;

    let reqs = server.received_requests().await.unwrap_or_default();
    let get_count = reqs.iter().filter(|r| r.method.as_str() == "GET").count();
    assert_eq!(get_count, 2);
    Ok(())
}

#[tokio::test]
async fn scenario_crud_lifecycle() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    csrf_endpoint(&server).await;
    Mock::given(method("GET"))
        .and(path("/cekunit"))
        .respond_with(ResponseTemplate::new(200).set_body_string("list"))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/cekunit"))
        .respond_with(ResponseTemplate::new(200).set_body_string("stored"))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/cekunit/1"))
        .respond_with(ResponseTemplate::new(200).set_body_string("updated"))
        .mount(&server)
        .await;

    let config = Config::new(&server.uri()).with_cookie_store(CookieStore::Memory);
    let client = Client::new(config).await?;

    let list = client.index().await?;
    assert_eq!(list.text().await?, "list");

    let mut data = Form::new();
    let _ = data.insert(String::from("nama_nasabah"), String::from("ALICE"));
    let stored = client.store(data).await?;
    assert_eq!(stored.status(), 200);

    let mut patch = Form::new();
    let _ = patch.insert(String::from("status"), String::from("LUNAS"));
    let updated = client.update(1, patch).await?;
    assert_eq!(updated.status(), 200);

    let reqs = server.received_requests().await.unwrap_or_default();
    let post_bodies = post_bodies(&reqs);
    assert_eq!(post_bodies.len(), 2);
    let store_body = post_bodies.first().ok_or("no store body")?;
    assert!(store_body.contains("nama_nasabah=ALICE"));
    let update_body = post_bodies.get(1).ok_or("no update body")?;
    assert!(update_body.contains("status=LUNAS"));
    assert!(update_body.contains("_method=PUT"));
    Ok(())
}

#[tokio::test]
async fn scenario_logout_clears_csrf() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    csrf_endpoint(&server).await;
    Mock::given(method("POST"))
        .and(path("/cekunit"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/logout"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let config = Config::new(&server.uri()).with_cookie_store(CookieStore::Memory);
    let client = Client::new(config).await?;

    let _ = client.store(Form::new()).await?;
    assert!(client.http().get_csrf().await.is_some());

    client.logout().await?;

    let after = client.http().get_csrf().await;
    assert!(after.is_none());
    Ok(())
}

#[tokio::test]
async fn scenario_cookie_file_persists_after_request() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dashboard"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("set-cookie", "session=abc123; Path=/")
                .set_body_string(CSRF_HTML),
        )
        .mount(&server)
        .await;

    let path = tmp_cookies_path("persist");
    let config =
        Config::new(&server.uri()).with_cookie_store(CookieStore::Persistent(path.clone()));

    let http = HttpClient::new(&config).await?;
    let _ = http.request(HttpMethod::GET, "/dashboard", None).await?;

    assert!(path.exists());
    let content = std::fs::read_to_string(&path)?;
    assert!(content.contains("session=abc123"));

    let _ = std::fs::remove_file(&path);
    Ok(())
}

#[tokio::test]
async fn scenario_cookie_file_restored_on_new_client() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dashboard"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("set-cookie", "session=persisted; Path=/")
                .set_body_string(CSRF_HTML),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/cekunit"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let path = tmp_cookies_path("restore");
    let config =
        Config::new(&server.uri()).with_cookie_store(CookieStore::Persistent(path.clone()));

    let http1 = HttpClient::new(&config).await?;
    let _ = http1.request(HttpMethod::GET, "/dashboard", None).await?;
    assert!(path.exists());

    let http2 = HttpClient::new(&config).await?;
    let resp = http2.request(HttpMethod::GET, "/cekunit", None).await?;
    assert_eq!(resp.status(), 200);

    let reqs = server.received_requests().await.unwrap_or_default();
    let cekunit_req = reqs
        .iter()
        .find(|r| r.url.path() == "/cekunit")
        .ok_or("no cekunit request")?;
    let cookie_header = cekunit_req
        .headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(cookie_header.contains("session=persisted"));

    let _ = std::fs::remove_file(&path);
    Ok(())
}

#[tokio::test]
async fn scenario_error_response_passes_through() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/cekunit/1"))
        .respond_with(ResponseTemplate::new(500).set_body_string("boom"))
        .mount(&server)
        .await;

    let config = Config::new(&server.uri()).with_cookie_store(CookieStore::Memory);
    let client = Client::new(config).await?;

    let resp = client.show(1).await?;
    assert_eq!(resp.status(), 500);
    assert_eq!(resp.text().await?, "boom");
    Ok(())
}

#[tokio::test]
async fn scenario_network_error_is_reported() -> Result<(), Box<dyn Error>> {
    let config = Config::new("http://127.0.0.1:1")
        .with_timeout(1)
        .with_cookie_store(CookieStore::Memory);
    let client = Client::new(config).await?;

    let result = client.index().await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn scenario_dashboard_delete_all_lifecycle() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    csrf_endpoint(&server).await;
    Mock::given(method("POST"))
        .and(path("/dashboard/delete-all"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let config = Config::new(&server.uri()).with_cookie_store(CookieStore::Memory);
    let client = Client::new(config).await?;

    let _ = client.delete_all().await?;

    let reqs = server.received_requests().await.unwrap_or_default();
    let post_bodies = post_bodies(&reqs);
    let body = post_bodies.first().ok_or("no POST")?;
    assert!(body.contains("_method=DELETE"));
    assert!(body.contains("_token=csrf-fixed"));
    Ok(())
}

#[tokio::test]
async fn scenario_export_streams_csv_body() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    let csv = "no,name\n1,ALICE\n2,BOB\n";
    Mock::given(method("GET"))
        .and(path("/dashboard/cekunit/export"))
        .respond_with(ResponseTemplate::new(200).set_body_string(csv))
        .mount(&server)
        .await;

    let config = Config::new(&server.uri()).with_cookie_store(CookieStore::Memory);
    let client = Client::new(config).await?;

    let resp = client.export("csv", "nomor", "asc").await?;
    assert_eq!(resp.status(), 200);
    let body = resp.text().await?;
    assert!(body.starts_with("no,name"));
    assert!(body.contains("ALICE"));
    Ok(())
}

#[tokio::test]
async fn scenario_full_csrf_lifecycle() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    csrf_endpoint(&server).await;
    Mock::given(method("POST"))
        .and(path("/cekunit"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let config = Config::new(&server.uri()).with_cookie_store(CookieStore::Memory);
    let client = Client::new(config).await?;

    assert!(client.http().get_csrf().await.is_none());
    let _ = client.store(Form::new()).await?;
    assert!(client.http().get_csrf().await.is_some());
    client.http().reset_csrf_token().await;
    assert!(client.http().get_csrf().await.is_none());
    let _ = client.store(Form::new()).await?;
    assert!(client.http().get_csrf().await.is_some());

    let reqs = server.received_requests().await.unwrap_or_default();
    let get_count = reqs.iter().filter(|r| r.method.as_str() == "GET").count();
    assert_eq!(get_count, 2);
    Ok(())
}
