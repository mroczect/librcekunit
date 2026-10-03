#![allow(unused_crate_dependencies)]

use core::error::Error;

use librcekunit_api::Client;
use librcekunit_handler::{
    Auth, Config, CookieStore, Crud, Dashboard, Form, HttpMethod, InputData, InputUser,
    InputUserId, Pic, PicId, Transport, UserId, Users,
};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const CSRF_HTML: &str = r#"<input name="_token" value="tok-dispatch">"#;

#[allow(clippy::too_many_lines)]
async fn setup() -> Result<(MockServer, Client), Box<dyn Error>> {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/dashboard"))
        .respond_with(ResponseTemplate::new(200).set_body_string(CSRF_HTML))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/cekunit"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/cekunit/create"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/cekunit"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/cekunit/7"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/cekunit/7/edit"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/cekunit/7"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/dashboard/cekunit/export"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/dashboard/cekunit/get-unique-values"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/dashboard/delete-all"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/dashboard/cekunit/delete-by-category"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/dashboard/input-data"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/dashboard/input-data"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/dashboard/input-user"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/dashboard/input_user/create"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/dashboard/input_user"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/dashboard/input_user/3"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/dashboard/input_user/3/edit"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/dashboard/input_user/3"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/dashboard/input_user/export"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/dashboard/input_user/insert"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/pic"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/pic/create"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/pic"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/pic/2"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/pic/2/edit"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/pic/2"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/dashboard/pic"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/dashboard/input-PIC"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/dashboard/input-PIC"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/dashboard/users"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/users/create"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/users/9"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/users/9/edit"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/users/9"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let config = Config::new(&server.uri()).with_cookie_store(CookieStore::Memory);
    let client = Client::new(config).await?;

    Ok((server, client))
}

fn cekunit_id() -> Result<librcekunit_handler::CekUnitId, Box<dyn Error>> {
    Ok(librcekunit_handler::CekUnitId::try_from(7_u64)?)
}

fn pic_id() -> Result<PicId, Box<dyn Error>> {
    Ok(PicId::try_from(2_u64)?)
}

fn user_id() -> Result<UserId, Box<dyn Error>> {
    Ok(UserId::try_from(9_u64)?)
}

fn input_user_id() -> Result<InputUserId, Box<dyn Error>> {
    Ok(InputUserId::try_from(3_u64)?)
}

#[tokio::test]
async fn dispatch_auth_login() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/login"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(r#"<input name="_token" value="login">"#),
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
    client.login("u@e.com", "p").await?;
    Ok(())
}

#[tokio::test]
async fn dispatch_auth_logout() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dashboard"))
        .respond_with(ResponseTemplate::new(200).set_body_string(CSRF_HTML))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/logout"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let config = Config::new(&server.uri()).with_cookie_store(CookieStore::Memory);
    let client = Client::new(config).await?;
    client.logout().await?;
    Ok(())
}

#[tokio::test]
async fn dispatch_crud_all_methods() -> Result<(), Box<dyn Error>> {
    let (_, client) = setup().await?;

    let _ = client.index().await?;
    let _ = client.index_with_params(Form::new()).await?;
    let _ = client.create().await?;
    let _ = client.store(Form::new()).await?;
    let _ = client.show(7).await?;
    let _ = client.edit(7).await?;
    let _ = client.update(7, Form::new()).await?;
    let _ = client.destroy(7).await?;

    let _ = cekunit_id()?;
    Ok(())
}

#[tokio::test]
async fn dispatch_dashboard_all_methods() -> Result<(), Box<dyn Error>> {
    let (_, client) = setup().await?;

    let _ = client.dashboard_index().await?;
    let _ = client.dashboard_index_with_params(Form::new()).await?;
    let _ = client.delete_all().await?;
    let _ = client.delete_by_category("k", "v").await?;
    let _ = client.export("csv", "nomor", "asc").await?;
    let _ = client.get_unique_values("kategori").await?;
    Ok(())
}

#[tokio::test]
async fn dispatch_input_data_all_methods() -> Result<(), Box<dyn Error>> {
    let (_, client) = setup().await?;

    let _ = client.input_data_create().await?;
    let _ = client.input_data_store(Form::new()).await?;
    Ok(())
}

#[tokio::test]
async fn dispatch_input_user_all_methods() -> Result<(), Box<dyn Error>> {
    let (_, client) = setup().await?;
    let id = input_user_id()?;

    let _ = client.input_user_index().await?;
    let _ = client.input_user_index_with_params(Form::new()).await?;
    let _ = client.input_user_create().await?;
    let _ = client.input_user_store(Form::new()).await?;
    let _ = client.input_user_show(id).await?;
    let _ = client.input_user_edit(id).await?;
    let _ = client.input_user_update(id, Form::new()).await?;
    let _ = client.input_user_destroy(id).await?;
    let _ = client.input_user_export(Form::new()).await?;
    let _ = client.input_user_import(Form::new()).await?;
    Ok(())
}

#[tokio::test]
async fn dispatch_pic_all_methods() -> Result<(), Box<dyn Error>> {
    let (_, client) = setup().await?;
    let id = pic_id()?;

    let _ = client.pic_index().await?;
    let _ = client.pic_index_with_params(Form::new()).await?;
    let _ = client.pic_create().await?;
    let _ = client.pic_store(Form::new()).await?;
    let _ = client.pic_show(id).await?;
    let _ = client.pic_edit(id).await?;
    let _ = client.pic_update(id, Form::new()).await?;
    let _ = client.pic_destroy(id).await?;
    let _ = client.dashboard_pic_index().await?;
    let _ = client.dashboard_pic_index_with_params(Form::new()).await?;
    let _ = client.input_pic_create().await?;
    let _ = client.input_pic_store(Form::new()).await?;
    Ok(())
}

#[tokio::test]
async fn dispatch_users_all_methods() -> Result<(), Box<dyn Error>> {
    let (_, client) = setup().await?;
    let id = user_id()?;

    let _ = client.users_index().await?;
    let _ = client.users_index_with_params(Form::new()).await?;
    let _ = client.users_create().await?;
    let _ = client.users_store(Form::new()).await?;
    let _ = client.users_show(id).await?;
    let _ = client.users_edit(id).await?;
    let _ = client.users_update(id, Form::new()).await?;
    let _ = client.users_destroy(id).await?;
    let _ = client.dashboard_users_index().await?;
    let _ = client
        .dashboard_users_index_with_params(Form::new())
        .await?;
    Ok(())
}

#[tokio::test]
async fn dispatch_transport_request() -> Result<(), Box<dyn Error>> {
    let (_, client) = setup().await?;
    let http = client.http();
    let _ = http.request(HttpMethod::GET, "/cekunit", None).await?;
    let _ = http
        .request(HttpMethod::POST, "/cekunit", Some(Form::new()))
        .await?;
    Ok(())
}

#[tokio::test]
async fn dispatch_transport_fetch_csrf() -> Result<(), Box<dyn Error>> {
    let (_, client) = setup().await?;
    let http = client.http();
    let token = http.fetch_csrf_token("/dashboard").await?;
    assert_eq!(token, "tok-dispatch");
    Ok(())
}

#[tokio::test]
async fn dispatch_transport_reset_csrf() -> Result<(), Box<dyn Error>> {
    let (_, client) = setup().await?;
    let http = client.http();
    let _ = http.fetch_csrf_token("/dashboard").await?;
    assert!(http.get_csrf().await.is_some());
    http.reset_csrf_token().await;
    assert!(http.get_csrf().await.is_none());
    Ok(())
}

#[tokio::test]
async fn dispatch_transport_clear_session() -> Result<(), Box<dyn Error>> {
    let (_, client) = setup().await?;
    let http = client.http();
    let _ = http.fetch_csrf_token("/dashboard").await?;
    http.clear_session().await;
    assert!(http.get_csrf().await.is_none());
    Ok(())
}

#[tokio::test]
async fn dispatch_client_import_csv() -> Result<(), Box<dyn Error>> {
    let (_, client) = setup().await?;

    let mut path = std::env::temp_dir();
    path.push("librcekunit_dispatch.csv");
    std::fs::write(&path, "no,foo\n1,bar\n")?;

    let _ = client.import_csv(&path).await?;

    let _ = std::fs::remove_file(&path);
    Ok(())
}
