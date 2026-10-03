#![allow(unused_crate_dependencies)]

use core::error::Error;

use librcekunit_api::Client;
use librcekunit_handler::{Config, CookieStore, Form};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const CSRF_HTML: &str = r#"<input name="_token" value="tok-csrf">"#;

async fn client_for(server: &MockServer) -> Result<Client, Box<dyn Error>> {
    let config = Config::new(&server.uri()).with_cookie_store(CookieStore::Memory);
    let client = Client::new(config).await?;
    Ok(client)
}

async fn mount_csrf(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/dashboard"))
        .respond_with(ResponseTemplate::new(200).set_body_string(CSRF_HTML))
        .mount(server)
        .await;
}

fn body_of(req: &wiremock::Request) -> String {
    String::from_utf8_lossy(&req.body).to_string()
}

fn find_method<'a>(reqs: &'a [wiremock::Request], m: &str) -> Option<&'a wiremock::Request> {
    reqs.iter().find(|r| r.method.as_str() == m)
}

#[tokio::test]
async fn cekunit_index_sends_get() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/cekunit"))
        .respond_with(ResponseTemplate::new(200).set_body_string("index-html"))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let resp = librcekunit_api::endpoints::cekunit::index(client.http()).await?;
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.text().await?, "index-html");

    let reqs = server.received_requests().await.unwrap_or_default();
    assert_eq!(reqs.len(), 1);
    let first = reqs.first().ok_or("no requests")?;
    assert_eq!(first.method.as_str(), "GET");
    assert_eq!(first.url.path(), "/cekunit");
    Ok(())
}

#[tokio::test]
async fn cekunit_index_with_params_adds_query() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/cekunit"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let mut params = Form::new();
    let _ = params.insert(String::from("page"), String::from("2"));
    let resp =
        librcekunit_api::endpoints::cekunit::index_with_params(client.http(), params).await?;
    assert_eq!(resp.status(), 200);

    let reqs = server.received_requests().await.unwrap_or_default();
    let first = reqs.first().ok_or("no requests")?;
    assert_eq!(first.url.query(), Some("page=2"));
    Ok(())
}

#[tokio::test]
async fn cekunit_create_sends_get_to_create_path() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/cekunit/create"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let resp = librcekunit_api::endpoints::cekunit::create(client.http()).await?;
    assert_eq!(resp.status(), 200);

    let reqs = server.received_requests().await.unwrap_or_default();
    let first = reqs.first().ok_or("no requests")?;
    assert_eq!(first.url.path(), "/cekunit/create");
    Ok(())
}

#[tokio::test]
async fn cekunit_store_injects_csrf() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    mount_csrf(&server).await;
    Mock::given(method("POST"))
        .and(path("/cekunit"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let mut form = Form::new();
    let _ = form.insert(String::from("nama_nasabah"), String::from("BUDI"));
    let _ = form.insert(String::from("nopol"), String::from("BP1234XY"));

    let resp = librcekunit_api::endpoints::cekunit::store(client.http(), form).await?;
    assert_eq!(resp.status(), 200);

    let reqs = server.received_requests().await.unwrap_or_default();
    let post = find_method(&reqs, "POST").ok_or("no POST")?;
    let body = body_of(post);
    assert!(body.contains("_token=tok-csrf"));
    assert!(body.contains("nama_nasabah=BUDI"));
    assert!(body.contains("nopol=BP1234XY"));
    Ok(())
}

#[tokio::test]
async fn cekunit_store_triggers_csrf_fetch_when_missing() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    mount_csrf(&server).await;
    Mock::given(method("POST"))
        .and(path("/cekunit"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let _ = librcekunit_api::endpoints::cekunit::store(client.http(), Form::new()).await?;

    let reqs = server.received_requests().await.unwrap_or_default();
    let get_count = reqs.iter().filter(|r| r.method.as_str() == "GET").count();
    assert_eq!(get_count, 1);
    Ok(())
}

#[tokio::test]
async fn cekunit_show_uses_id_in_path() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/cekunit/42"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let resp = librcekunit_api::endpoints::cekunit::show(client.http(), 42).await?;
    assert_eq!(resp.status(), 200);

    let reqs = server.received_requests().await.unwrap_or_default();
    let first = reqs.first().ok_or("no requests")?;
    assert_eq!(first.url.path(), "/cekunit/42");
    Ok(())
}

#[tokio::test]
async fn cekunit_update_spoofs_put_via_method_field() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    mount_csrf(&server).await;
    Mock::given(method("POST"))
        .and(path("/cekunit/7"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let mut data = Form::new();
    let _ = data.insert(String::from("status"), String::from("LUNAS"));
    let resp = librcekunit_api::endpoints::cekunit::update(client.http(), 7, data).await?;
    assert_eq!(resp.status(), 200);

    let reqs = server.received_requests().await.unwrap_or_default();
    let post = find_method(&reqs, "POST").ok_or("no POST")?;
    assert_eq!(post.url.path(), "/cekunit/7");
    let body = body_of(post);
    assert!(body.contains("_method=PUT"));
    assert!(body.contains("status=LUNAS"));
    assert!(body.contains("_token=tok-csrf"));
    Ok(())
}

#[tokio::test]
async fn cekunit_destroy_spoofs_delete_via_method_field() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    mount_csrf(&server).await;
    Mock::given(method("POST"))
        .and(path("/cekunit/9"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let resp = librcekunit_api::endpoints::cekunit::destroy(client.http(), 9).await?;
    assert_eq!(resp.status(), 200);

    let reqs = server.received_requests().await.unwrap_or_default();
    let post = find_method(&reqs, "POST").ok_or("no POST")?;
    let body = body_of(post);
    assert!(body.contains("_method=DELETE"));
    Ok(())
}

#[tokio::test]
async fn dashboard_index_sends_get() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dashboard"))
        .respond_with(ResponseTemplate::new(200).set_body_string("dash"))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let resp = librcekunit_api::endpoints::dashboard::index(client.http()).await?;
    assert_eq!(resp.text().await?, "dash");
    Ok(())
}

#[tokio::test]
async fn dashboard_delete_all_spoofs_delete() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    mount_csrf(&server).await;
    Mock::given(method("POST"))
        .and(path("/dashboard/delete-all"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let resp = librcekunit_api::endpoints::dashboard::delete_all(client.http()).await?;
    assert_eq!(resp.status(), 200);

    let reqs = server.received_requests().await.unwrap_or_default();
    let post = find_method(&reqs, "POST").ok_or("no POST")?;
    let body = body_of(post);
    assert!(body.contains("_method=DELETE"));
    Ok(())
}

#[tokio::test]
async fn dashboard_delete_by_category_sends_column_and_value() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    mount_csrf(&server).await;
    Mock::given(method("POST"))
        .and(path("/dashboard/cekunit/delete-by-category"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let resp =
        librcekunit_api::endpoints::dashboard::delete_by_category(client.http(), "kategori", "A")
            .await?;
    assert_eq!(resp.status(), 200);

    let reqs = server.received_requests().await.unwrap_or_default();
    let post = find_method(&reqs, "POST").ok_or("no POST")?;
    let body = body_of(post);
    assert!(body.contains("column=kategori"));
    assert!(body.contains("value=A"));
    assert!(body.contains("_method=DELETE"));
    Ok(())
}

#[tokio::test]
async fn dashboard_delete_by_category_rejects_empty_column() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    let client = client_for(&server).await?;
    let result =
        librcekunit_api::endpoints::dashboard::delete_by_category(client.http(), "", "value").await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn dashboard_delete_by_category_rejects_empty_value() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    let client = client_for(&server).await?;
    let result =
        librcekunit_api::endpoints::dashboard::delete_by_category(client.http(), "column", "")
            .await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn dashboard_export_sends_query_params() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dashboard/cekunit/export"))
        .respond_with(ResponseTemplate::new(200).set_body_string("csv"))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let resp =
        librcekunit_api::endpoints::dashboard::export(client.http(), "csv", "nomor", "asc").await?;
    assert_eq!(resp.text().await?, "csv");

    let reqs = server.received_requests().await.unwrap_or_default();
    let first = reqs.first().ok_or("no requests")?;
    let query = first.url.query().ok_or("no query")?;
    assert!(query.contains("format=csv"));
    assert!(query.contains("sort=nomor"));
    assert!(query.contains("direction=asc"));
    Ok(())
}

#[tokio::test]
async fn dashboard_get_unique_values_sends_column() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dashboard/cekunit/get-unique-values"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let resp =
        librcekunit_api::endpoints::dashboard::get_unique_values(client.http(), "kategori").await?;
    assert_eq!(resp.status(), 200);

    let reqs = server.received_requests().await.unwrap_or_default();
    let first = reqs.first().ok_or("no requests")?;
    assert_eq!(first.url.query(), Some("column=kategori"));
    Ok(())
}

#[tokio::test]
async fn input_data_create_sends_get() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dashboard/input-data"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let resp = librcekunit_api::endpoints::input_data::create(client.http()).await?;
    assert_eq!(resp.status(), 200);
    Ok(())
}

#[tokio::test]
async fn input_data_store_sends_post_with_csrf() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    mount_csrf(&server).await;
    Mock::given(method("POST"))
        .and(path("/dashboard/input-data"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let mut form = Form::new();
    let _ = form.insert(String::from("no_perjanjian"), String::from("SP-25-001"));
    let resp = librcekunit_api::endpoints::input_data::store(client.http(), form).await?;
    assert_eq!(resp.status(), 200);

    let reqs = server.received_requests().await.unwrap_or_default();
    let post = find_method(&reqs, "POST").ok_or("no POST")?;
    let body = body_of(post);
    assert!(body.contains("no_perjanjian=SP-25-001"));
    assert!(body.contains("_token=tok-csrf"));
    Ok(())
}

#[tokio::test]
async fn input_user_index_sends_get() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dashboard/input-user"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let resp = librcekunit_api::endpoints::input_user::index(client.http()).await?;
    assert_eq!(resp.status(), 200);

    let reqs = server.received_requests().await.unwrap_or_default();
    let first = reqs.first().ok_or("no requests")?;
    assert_eq!(first.url.path(), "/dashboard/input-user");
    Ok(())
}

#[tokio::test]
async fn input_user_store_sends_post() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    mount_csrf(&server).await;
    Mock::given(method("POST"))
        .and(path("/dashboard/input_user"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let resp = librcekunit_api::endpoints::input_user::store(client.http(), Form::new()).await?;
    assert_eq!(resp.status(), 200);

    let reqs = server.received_requests().await.unwrap_or_default();
    let post = find_method(&reqs, "POST").ok_or("no POST")?;
    assert_eq!(post.url.path(), "/dashboard/input_user");
    Ok(())
}

#[tokio::test]
async fn input_user_update_spoofs_put() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    mount_csrf(&server).await;
    Mock::given(method("POST"))
        .and(path("/dashboard/input_user/3"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let resp =
        librcekunit_api::endpoints::input_user::update(client.http(), 3, Form::new()).await?;
    assert_eq!(resp.status(), 200);

    let reqs = server.received_requests().await.unwrap_or_default();
    let post = find_method(&reqs, "POST").ok_or("no POST")?;
    let body = body_of(post);
    assert!(body.contains("_method=PUT"));
    Ok(())
}

#[tokio::test]
async fn input_user_destroy_spoofs_delete() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    mount_csrf(&server).await;
    Mock::given(method("POST"))
        .and(path("/dashboard/input_user/5"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let resp = librcekunit_api::endpoints::input_user::destroy(client.http(), 5).await?;
    assert_eq!(resp.status(), 200);

    let reqs = server.received_requests().await.unwrap_or_default();
    let post = find_method(&reqs, "POST").ok_or("no POST")?;
    let body = body_of(post);
    assert!(body.contains("_method=DELETE"));
    Ok(())
}

#[tokio::test]
async fn input_user_export_sends_query_params() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dashboard/input_user/export"))
        .respond_with(ResponseTemplate::new(200).set_body_string("csv"))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let mut params = Form::new();
    let _ = params.insert(String::from("start_date"), String::from("2026-01-01"));
    let resp = librcekunit_api::endpoints::input_user::export(client.http(), params).await?;
    assert_eq!(resp.text().await?, "csv");

    let reqs = server.received_requests().await.unwrap_or_default();
    let first = reqs.first().ok_or("no requests")?;
    assert_eq!(first.url.query(), Some("start_date=2026-01-01"));
    Ok(())
}

#[tokio::test]
async fn input_user_import_file_missing_yields_io_error() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    let client = client_for(&server).await?;
    let path = std::path::Path::new("/nonexistent/no.csv");
    let result = librcekunit_api::endpoints::input_user::import_file(client.http(), path).await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn input_user_import_file_sends_multipart() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    mount_csrf(&server).await;
    Mock::given(method("POST"))
        .and(path("/dashboard/input_user/insert"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let mut path = std::env::temp_dir();
    path.push("librcekunit_import_test.csv");
    std::fs::write(&path, "no,foo\n1,bar\n")?;

    let client = client_for(&server).await?;
    let resp = librcekunit_api::endpoints::input_user::import_file(client.http(), &path).await?;
    assert_eq!(resp.status(), 200);

    let reqs = server.received_requests().await.unwrap_or_default();
    let post = find_method(&reqs, "POST").ok_or("no POST")?;
    let content_type = post
        .headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(content_type.starts_with("multipart/form-data"));
    let body = body_of(post);
    assert!(body.contains("no,foo"));

    let _ = std::fs::remove_file(&path);
    Ok(())
}

#[tokio::test]
async fn pic_index_sends_get_to_pic() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pic"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let resp = librcekunit_api::endpoints::pic::index(client.http()).await?;
    assert_eq!(resp.status(), 200);
    Ok(())
}

#[tokio::test]
async fn pic_create_sends_get_to_pic_create() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pic/create"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let resp = librcekunit_api::endpoints::pic::create(client.http()).await?;
    assert_eq!(resp.status(), 200);
    Ok(())
}

#[tokio::test]
async fn pic_store_sends_post_with_csrf() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    mount_csrf(&server).await;
    Mock::given(method("POST"))
        .and(path("/pic"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let mut form = Form::new();
    let _ = form.insert(String::from("id_coll"), String::from("C001"));
    let resp = librcekunit_api::endpoints::pic::store(client.http(), form).await?;
    assert_eq!(resp.status(), 200);

    let reqs = server.received_requests().await.unwrap_or_default();
    let post = find_method(&reqs, "POST").ok_or("no POST")?;
    let body = body_of(post);
    assert!(body.contains("id_coll=C001"));
    assert!(body.contains("_token=tok-csrf"));
    Ok(())
}

#[tokio::test]
async fn pic_update_spoofs_put() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    mount_csrf(&server).await;
    Mock::given(method("POST"))
        .and(path("/pic/2"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let resp = librcekunit_api::endpoints::pic::update(client.http(), 2, Form::new()).await?;
    assert_eq!(resp.status(), 200);

    let reqs = server.received_requests().await.unwrap_or_default();
    let post = find_method(&reqs, "POST").ok_or("no POST")?;
    let body = body_of(post);
    assert!(body.contains("_method=PUT"));
    Ok(())
}

#[tokio::test]
async fn pic_destroy_spoofs_delete() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    mount_csrf(&server).await;
    Mock::given(method("POST"))
        .and(path("/pic/4"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let resp = librcekunit_api::endpoints::pic::destroy(client.http(), 4).await?;
    assert_eq!(resp.status(), 200);

    let reqs = server.received_requests().await.unwrap_or_default();
    let post = find_method(&reqs, "POST").ok_or("no POST")?;
    let body = body_of(post);
    assert!(body.contains("_method=DELETE"));
    Ok(())
}

#[tokio::test]
async fn pic_dashboard_index_sends_get() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dashboard/pic"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let resp = librcekunit_api::endpoints::pic::dashboard_index(client.http()).await?;
    assert_eq!(resp.status(), 200);
    Ok(())
}

#[tokio::test]
async fn pic_input_create_sends_get() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dashboard/input-PIC"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let resp = librcekunit_api::endpoints::pic::input_create(client.http()).await?;
    assert_eq!(resp.status(), 200);
    Ok(())
}

#[tokio::test]
async fn pic_input_store_sends_post_with_csrf() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    mount_csrf(&server).await;
    Mock::given(method("POST"))
        .and(path("/dashboard/input-PIC"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let resp = librcekunit_api::endpoints::pic::input_store(client.http(), Form::new()).await?;
    assert_eq!(resp.status(), 200);

    let reqs = server.received_requests().await.unwrap_or_default();
    let post = find_method(&reqs, "POST").ok_or("no POST")?;
    let body = body_of(post);
    assert!(body.contains("_token=tok-csrf"));
    Ok(())
}

#[tokio::test]
async fn users_index_sends_get_to_dashboard_users() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dashboard/users"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let resp = librcekunit_api::endpoints::users::index(client.http()).await?;
    assert_eq!(resp.status(), 200);

    let reqs = server.received_requests().await.unwrap_or_default();
    let first = reqs.first().ok_or("no requests")?;
    assert_eq!(first.url.path(), "/dashboard/users");
    Ok(())
}

#[tokio::test]
async fn users_create_sends_get_to_users_create() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/users/create"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let resp = librcekunit_api::endpoints::users::create(client.http()).await?;
    assert_eq!(resp.status(), 200);
    Ok(())
}

#[tokio::test]
async fn users_store_sends_post_to_dashboard_users() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    mount_csrf(&server).await;
    Mock::given(method("POST"))
        .and(path("/dashboard/users"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let resp = librcekunit_api::endpoints::users::store(client.http(), Form::new()).await?;
    assert_eq!(resp.status(), 200);

    let reqs = server.received_requests().await.unwrap_or_default();
    let post = find_method(&reqs, "POST").ok_or("no POST")?;
    assert_eq!(post.url.path(), "/dashboard/users");
    Ok(())
}

#[tokio::test]
async fn users_update_spoofs_put() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    mount_csrf(&server).await;
    Mock::given(method("POST"))
        .and(path("/dashboard/users"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let resp = librcekunit_api::endpoints::users::update(client.http(), 8, Form::new()).await?;
    assert_eq!(resp.status(), 200);

    let reqs = server.received_requests().await.unwrap_or_default();
    let post = find_method(&reqs, "POST").ok_or("no POST")?;
    let body = body_of(post);
    assert!(body.contains("_method=PUT"));
    Ok(())
}

#[tokio::test]
async fn users_destroy_spoofs_delete() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    mount_csrf(&server).await;
    Mock::given(method("POST"))
        .and(path("/users/8"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let resp = librcekunit_api::endpoints::users::destroy(client.http(), 8).await?;
    assert_eq!(resp.status(), 200);

    let reqs = server.received_requests().await.unwrap_or_default();
    let post = find_method(&reqs, "POST").ok_or("no POST")?;
    let body = body_of(post);
    assert!(body.contains("_method=DELETE"));
    Ok(())
}

#[tokio::test]
async fn auth_login_sends_credentials_with_csrf() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/login"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"<input name="_token" value="login-csrf">"#),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/login"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    librcekunit_api::endpoints::auth::login(client.http(), "user@example.com", "secret").await?;

    let reqs = server.received_requests().await.unwrap_or_default();
    let post = find_method(&reqs, "POST").ok_or("no POST")?;
    let body = body_of(post);
    assert!(body.contains("_token=login-csrf"));
    assert!(body.contains("email=user%40example.com"));
    assert!(body.contains("password=secret"));
    Ok(())
}

#[tokio::test]
async fn auth_logout_sends_post_with_csrf() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    mount_csrf(&server).await;
    Mock::given(method("POST"))
        .and(path("/logout"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    librcekunit_api::endpoints::auth::logout(client.http()).await?;

    let reqs = server.received_requests().await.unwrap_or_default();
    let post = find_method(&reqs, "POST").ok_or("no POST")?;
    assert_eq!(post.url.path(), "/logout");
    Ok(())
}

#[tokio::test]
async fn http_500_response_is_returned_not_error() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/cekunit/1"))
        .respond_with(ResponseTemplate::new(500).set_body_string("server error"))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let resp = librcekunit_api::endpoints::cekunit::show(client.http(), 1).await?;
    assert_eq!(resp.status(), 500);
    Ok(())
}

#[tokio::test]
async fn http_404_response_is_returned_not_error() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/cekunit/999"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let client = client_for(&server).await?;
    let resp = librcekunit_api::endpoints::cekunit::show(client.http(), 999).await?;
    assert_eq!(resp.status(), 404);
    Ok(())
}
