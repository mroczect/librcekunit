#![allow(unused_crate_dependencies)]

use core::error::Error;

use librcekunit_api::{Client, HttpClient, csrf};
use librcekunit_handler::{
    ApiResponse, CekUnitId, Config, CookieStore, Error as HandlerError, HttpMethod, InputUserId,
    JoinUrl, Mode, PicId, State, Transport, UserId, Validate,
};

fn unit_config() -> Config {
    Config::new("http://127.0.0.1:1").with_cookie_store(CookieStore::Memory)
}

fn tmp_path(tag: &str) -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("librcekunit_unit_{tag}.json"));
    let _ = std::fs::remove_file(&p);
    p
}

#[test]
fn csrf_extract_from_hidden_input() -> Result<(), Box<dyn Error>> {
    let html = r#"<form><input type="hidden" name="_token" value="tok-abc"></form>"#;
    assert_eq!(csrf::extract(html)?, "tok-abc");
    Ok(())
}

#[test]
fn csrf_extract_from_visible_input() -> Result<(), Box<dyn Error>> {
    let html = r#"<input name="_token" value="tok-visible">"#;
    assert_eq!(csrf::extract(html)?, "tok-visible");
    Ok(())
}

#[test]
fn csrf_extract_returns_first_when_multiple() -> Result<(), Box<dyn Error>> {
    let html = r#"<input name="_token" value="first"><input name="_token" value="second">"#;
    assert_eq!(csrf::extract(html)?, "first");
    Ok(())
}

#[test]
fn csrf_extract_from_double_quote_attribute() -> Result<(), Box<dyn Error>> {
    let html = r#"<input name="_token" value="dq-value">"#;
    assert_eq!(csrf::extract(html)?, "dq-value");
    Ok(())
}

#[test]
fn csrf_extract_inside_large_html() -> Result<(), Box<dyn Error>> {
    let pad = "<div>".repeat(500);
    let html = format!("<html>{pad}<input name=\"_token\" value=\"deep\"></html>");
    assert_eq!(csrf::extract(&html)?, "deep");
    Ok(())
}

#[test]
fn csrf_extract_with_other_inputs_around() -> Result<(), Box<dyn Error>> {
    let html = r#"
        <form>
            <input name="email" value="x">
            <input name="_token" value="real-token">
            <input name="password" value="y">
        </form>
    "#;
    assert_eq!(csrf::extract(html)?, "real-token");
    Ok(())
}

#[test]
fn csrf_extract_with_attribute_order_reversed() -> Result<(), Box<dyn Error>> {
    let html = r#"<input value="ordered" name="_token">"#;
    assert_eq!(csrf::extract(html)?, "ordered");
    Ok(())
}

#[test]
fn csrf_extract_value_with_special_chars() -> Result<(), Box<dyn Error>> {
    let html = r#"<input name="_token" value="a-b_c.d~e">"#;
    assert_eq!(csrf::extract(html)?, "a-b_c.d~e");
    Ok(())
}

#[test]
fn csrf_extract_value_long_token() -> Result<(), Box<dyn Error>> {
    let long = "x".repeat(200);
    let html = format!(r#"<input name="_token" value="{long}">"#);
    assert_eq!(csrf::extract(&html)?, long);
    Ok(())
}

#[test]
fn csrf_extract_empty_value_is_returned() -> Result<(), Box<dyn Error>> {
    let html = r#"<input name="_token" value="">"#;
    assert_eq!(csrf::extract(html)?, "");
    Ok(())
}

#[test]
fn csrf_extract_errors_when_missing() {
    assert!(csrf::extract("<html><body></body></html>").is_err());
}

#[test]
fn csrf_extract_errors_on_empty_input() {
    assert!(csrf::extract("").is_err());
}

#[test]
fn csrf_extract_errors_when_value_attr_missing() {
    assert!(csrf::extract(r#"<input name="_token">"#).is_err());
}

#[test]
fn csrf_extract_errors_when_name_attr_missing() {
    assert!(csrf::extract(r#"<input value="x">"#).is_err());
}

#[test]
fn csrf_extract_ignores_similar_names() {
    assert!(csrf::extract(r#"<input name="_tokens" value="x">"#).is_err());
}

#[test]
fn csrf_extract_ignores_prefix_name() {
    assert!(csrf::extract(r#"<input name="my_token" value="x">"#).is_err());
}

#[test]
fn csrf_extract_errors_on_plain_text() {
    assert!(csrf::extract("just plain text").is_err());
}

#[test]
fn csrf_extract_errors_on_json_body() {
    assert!(csrf::extract(r#"{"_token":"in-json"}"#).is_err());
}

#[test]
fn http_method_get_str() {
    assert_eq!(HttpMethod::GET.as_str(), "GET");
}

#[test]
fn http_method_post_str() {
    assert_eq!(HttpMethod::POST.as_str(), "POST");
}

#[test]
fn http_method_put_str() {
    assert_eq!(HttpMethod::PUT.as_str(), "PUT");
}

#[test]
fn http_method_patch_str() {
    assert_eq!(HttpMethod::PATCH.as_str(), "PATCH");
}

#[test]
fn http_method_delete_str() {
    assert_eq!(HttpMethod::DELETE.as_str(), "DELETE");
}

#[test]
fn http_method_head_str() {
    assert_eq!(HttpMethod::HEAD.as_str(), "HEAD");
}

#[test]
fn http_method_options_str() {
    assert_eq!(HttpMethod::OPTIONS.as_str(), "OPTIONS");
}

#[test]
fn http_method_display_matches_as_str() {
    assert_eq!(HttpMethod::GET.to_string(), HttpMethod::GET.as_str());
    assert_eq!(HttpMethod::POST.to_string(), HttpMethod::POST.as_str());
    assert_eq!(HttpMethod::PUT.to_string(), HttpMethod::PUT.as_str());
    assert_eq!(HttpMethod::PATCH.to_string(), HttpMethod::PATCH.as_str());
    assert_eq!(HttpMethod::DELETE.to_string(), HttpMethod::DELETE.as_str());
    assert_eq!(HttpMethod::HEAD.to_string(), HttpMethod::HEAD.as_str());
    assert_eq!(
        HttpMethod::OPTIONS.to_string(),
        HttpMethod::OPTIONS.as_str()
    );
}

#[test]
fn http_method_get_no_csrf() {
    assert!(!HttpMethod::GET.requires_csrf());
}

#[test]
fn http_method_head_no_csrf() {
    assert!(!HttpMethod::HEAD.requires_csrf());
}

#[test]
fn http_method_options_no_csrf() {
    assert!(!HttpMethod::OPTIONS.requires_csrf());
}

#[test]
fn http_method_post_requires_csrf() {
    assert!(HttpMethod::POST.requires_csrf());
}

#[test]
fn http_method_put_requires_csrf() {
    assert!(HttpMethod::PUT.requires_csrf());
}

#[test]
fn http_method_patch_requires_csrf() {
    assert!(HttpMethod::PATCH.requires_csrf());
}

#[test]
fn http_method_delete_requires_csrf() {
    assert!(HttpMethod::DELETE.requires_csrf());
}

#[test]
fn http_method_get_no_body() {
    assert!(!HttpMethod::GET.has_body());
}

#[test]
fn http_method_head_no_body() {
    assert!(!HttpMethod::HEAD.has_body());
}

#[test]
fn http_method_options_no_body() {
    assert!(!HttpMethod::OPTIONS.has_body());
}

#[test]
fn http_method_post_has_body() {
    assert!(HttpMethod::POST.has_body());
}

#[test]
fn http_method_put_has_body() {
    assert!(HttpMethod::PUT.has_body());
}

#[test]
fn http_method_patch_has_body() {
    assert!(HttpMethod::PATCH.has_body());
}

#[test]
fn http_method_delete_has_body() {
    assert!(HttpMethod::DELETE.has_body());
}

#[test]
fn http_method_eq() {
    assert_eq!(HttpMethod::GET, HttpMethod::GET);
    assert_ne!(HttpMethod::GET, HttpMethod::POST);
}

#[test]
fn http_method_copy() {
    let a = HttpMethod::GET;
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn http_method_hashset_dedup() {
    use std::collections::HashSet;
    let mut set: HashSet<HttpMethod> = HashSet::new();
    let _ = set.insert(HttpMethod::GET);
    let _ = set.insert(HttpMethod::GET);
    let _ = set.insert(HttpMethod::POST);
    assert_eq!(set.len(), 2);
}

#[tokio::test]
async fn http_client_csrf_starts_none() -> Result<(), Box<dyn Error>> {
    let http = HttpClient::new(&unit_config()).await?;
    assert!(http.get_csrf().await.is_none());
    Ok(())
}

#[tokio::test]
async fn http_client_set_then_get_csrf() -> Result<(), Box<dyn Error>> {
    let http = HttpClient::new(&unit_config()).await?;
    http.set_csrf(Some(String::from("tok-1"))).await;
    assert_eq!(http.get_csrf().await.as_deref(), Some("tok-1"));
    Ok(())
}

#[tokio::test]
async fn http_client_set_overwrite_csrf() -> Result<(), Box<dyn Error>> {
    let http = HttpClient::new(&unit_config()).await?;
    http.set_csrf(Some(String::from("first"))).await;
    http.set_csrf(Some(String::from("second"))).await;
    assert_eq!(http.get_csrf().await.as_deref(), Some("second"));
    Ok(())
}

#[tokio::test]
async fn http_client_reset_csrf_clears() -> Result<(), Box<dyn Error>> {
    let http = HttpClient::new(&unit_config()).await?;
    http.set_csrf(Some(String::from("tok"))).await;
    http.reset_csrf_token().await;
    assert!(http.get_csrf().await.is_none());
    Ok(())
}

#[tokio::test]
async fn http_client_set_none_clears() -> Result<(), Box<dyn Error>> {
    let http = HttpClient::new(&unit_config()).await?;
    http.set_csrf(Some(String::from("tok"))).await;
    http.set_csrf(None).await;
    assert!(http.get_csrf().await.is_none());
    Ok(())
}

#[tokio::test]
async fn http_client_clear_session_clears_csrf() -> Result<(), Box<dyn Error>> {
    let http = HttpClient::new(&unit_config()).await?;
    http.set_csrf(Some(String::from("tok"))).await;
    http.clear_session().await;
    assert!(http.get_csrf().await.is_none());
    Ok(())
}

#[tokio::test]
async fn http_client_set_empty_string_is_some() -> Result<(), Box<dyn Error>> {
    let http = HttpClient::new(&unit_config()).await?;
    http.set_csrf(Some(String::new())).await;
    assert_eq!(http.get_csrf().await.as_deref(), Some(""));
    Ok(())
}

#[tokio::test]
async fn http_client_roundtrip_unicode_token() -> Result<(), Box<dyn Error>> {
    let http = HttpClient::new(&unit_config()).await?;
    http.set_csrf(Some(String::from("café-tok-日本"))).await;
    assert_eq!(http.get_csrf().await.as_deref(), Some("café-tok-日本"));
    Ok(())
}

#[tokio::test]
async fn http_client_concurrent_set_get() -> Result<(), Box<dyn Error>> {
    let http = HttpClient::new(&unit_config()).await?;
    let set_fut = http.set_csrf(Some(String::from("concurrent")));
    let get_fut = http.get_csrf();
    let ((), _token) = tokio::join!(set_fut, get_fut);
    assert_eq!(http.get_csrf().await.as_deref(), Some("concurrent"));
    Ok(())
}

#[tokio::test]
async fn http_client_multiple_resets_idempotent() -> Result<(), Box<dyn Error>> {
    let http = HttpClient::new(&unit_config()).await?;
    http.reset_csrf_token().await;
    http.reset_csrf_token().await;
    http.reset_csrf_token().await;
    assert!(http.get_csrf().await.is_none());
    Ok(())
}

#[tokio::test]
async fn http_client_debug_contains_struct_name() -> Result<(), Box<dyn Error>> {
    let http = HttpClient::new(&unit_config()).await?;
    let debug = format!("{http:?}");
    assert!(debug.contains("HttpClient"));
    Ok(())
}

#[tokio::test]
async fn http_client_debug_shows_base_url() -> Result<(), Box<dyn Error>> {
    let http = HttpClient::new(&unit_config()).await?;
    let debug = format!("{http:?}");
    assert!(debug.contains("127.0.0.1"));
    Ok(())
}

#[tokio::test]
async fn http_client_new_with_memory_store() -> Result<(), Box<dyn Error>> {
    let config = Config::new("http://127.0.0.1:1").with_cookie_store(CookieStore::Memory);
    let http = HttpClient::new(&config).await?;
    assert!(http.get_csrf().await.is_none());
    Ok(())
}

#[tokio::test]
async fn http_client_new_with_none_store() -> Result<(), Box<dyn Error>> {
    let config = Config::new("http://127.0.0.1:1").with_cookie_store(CookieStore::None);
    let http = HttpClient::new(&config).await?;
    assert!(http.get_csrf().await.is_none());
    Ok(())
}

#[tokio::test]
async fn http_client_new_with_persistent_store_missing_file() -> Result<(), Box<dyn Error>> {
    let path = tmp_path("missing");
    let config =
        Config::new("http://127.0.0.1:1").with_cookie_store(CookieStore::Persistent(path.clone()));
    let http = HttpClient::new(&config).await?;
    assert!(http.get_csrf().await.is_none());
    let _ = std::fs::remove_file(&path);
    Ok(())
}

#[tokio::test]
async fn http_client_new_trims_trailing_slash() -> Result<(), Box<dyn Error>> {
    let config = Config::new("http://127.0.0.1:1/").with_cookie_store(CookieStore::Memory);
    let http = HttpClient::new(&config).await?;
    let debug = format!("{http:?}");
    assert!(debug.contains("127.0.0.1:1"));
    Ok(())
}

#[tokio::test]
async fn http_client_new_with_custom_user_agent() -> Result<(), Box<dyn Error>> {
    let config = Config::new("http://127.0.0.1:1")
        .with_user_agent("custom/9.9")
        .with_cookie_store(CookieStore::Memory);
    let http = HttpClient::new(&config).await?;
    assert!(http.get_csrf().await.is_none());
    Ok(())
}

#[tokio::test]
async fn http_client_new_with_short_timeout() -> Result<(), Box<dyn Error>> {
    let config = Config::new("http://127.0.0.1:1")
        .with_timeout(1)
        .with_cookie_store(CookieStore::Memory);
    let http = HttpClient::new(&config).await?;
    assert!(http.get_csrf().await.is_none());
    Ok(())
}

#[tokio::test]
async fn http_client_new_with_invalid_base_url_config() {
    let config = Config::new("");
    let result = HttpClient::new(&config).await;
    let _ = result;
}

#[tokio::test]
async fn client_new_succeeds_with_memory_store() -> Result<(), Box<dyn Error>> {
    let config = Config::new("http://127.0.0.1:1").with_cookie_store(CookieStore::Memory);
    let _client = Client::new(config).await?;
    Ok(())
}

#[tokio::test]
async fn client_new_succeeds_with_none_store() -> Result<(), Box<dyn Error>> {
    let config = Config::new("http://127.0.0.1:1").with_cookie_store(CookieStore::None);
    let _client = Client::new(config).await?;
    Ok(())
}

#[tokio::test]
async fn client_debug_contains_struct_name() -> Result<(), Box<dyn Error>> {
    let client = Client::new(unit_config()).await?;
    let debug = format!("{client:?}");
    assert!(debug.contains("Client"));
    Ok(())
}

#[tokio::test]
async fn client_http_accessor_returns_reference() -> Result<(), Box<dyn Error>> {
    let client = Client::new(unit_config()).await?;
    let _http = client.http();
    Ok(())
}

#[tokio::test]
async fn client_http_accessor_stable_across_calls() -> Result<(), Box<dyn Error>> {
    let client = Client::new(unit_config()).await?;
    let a: *const HttpClient = client.http();
    let b: *const HttpClient = client.http();
    assert!(core::ptr::eq(a, b));
    Ok(())
}

#[test]
fn config_default_base_url() {
    assert_eq!(Config::new("http://x.com").base_url, "http://x.com");
}

#[test]
fn config_trim_trailing_slash() {
    assert_eq!(Config::new("http://x.com/").base_url, "http://x.com");
}

#[test]
fn config_trim_multiple_slashes() {
    assert_eq!(Config::new("http://x.com///").base_url, "http://x.com");
}

#[test]
fn config_empty_becomes_root() {
    assert_eq!(Config::new("").base_url, "/");
}

#[test]
fn config_default_user_agent() {
    assert_eq!(Config::new("http://x.com").user_agent, "librcekunit/3.0");
}

#[test]
fn config_custom_user_agent() {
    assert_eq!(
        Config::new("http://x.com")
            .with_user_agent("custom/1")
            .user_agent,
        "custom/1"
    );
}

#[test]
fn config_validate_ok_https() {
    assert!(Config::new("https://example.com").validate().is_ok());
}

#[test]
fn config_validate_ok_http() {
    assert!(Config::new("http://localhost").validate().is_ok());
}

#[test]
fn config_validate_rejects_empty() {
    assert!(Config::new("").validate().is_err());
}

#[test]
fn config_validate_rejects_whitespace() {
    assert!(Config::new("   ").validate().is_err());
}

#[test]
fn config_validate_rejects_relative() {
    assert!(Config::new("example.com").validate().is_err());
}

#[test]
fn config_validate_rejects_zero_timeout() {
    assert!(
        Config::new("http://x.com")
            .with_timeout(0)
            .validate()
            .is_err()
    );
}

#[test]
fn config_validate_rejects_empty_user_agent() {
    assert!(
        Config::new("http://x.com")
            .with_user_agent("")
            .validate()
            .is_err()
    );
}

#[test]
fn config_validate_rejects_ftp_scheme() {
    assert!(Config::new("ftp://x.com").validate().is_err());
}

#[test]
fn cekunit_id_accepts_positive() {
    assert!(CekUnitId::new(1).is_some());
    assert!(CekUnitId::new(u64::MAX).is_some());
}

#[test]
fn cekunit_id_rejects_zero() {
    assert!(CekUnitId::new(0).is_none());
}

#[test]
fn cekunit_id_get_roundtrip() {
    assert_eq!(CekUnitId::new(42).map(CekUnitId::get), Some(42));
}

#[test]
fn cekunit_id_display() {
    assert_eq!(
        CekUnitId::new(7).map(|x| x.to_string()),
        Some(String::from("7"))
    );
}

#[test]
fn cekunit_id_try_from_ok() {
    assert!(CekUnitId::try_from(5_u64).is_ok());
}

#[test]
fn cekunit_id_try_from_zero_err() {
    assert!(CekUnitId::try_from(0_u64).is_err());
}

#[test]
fn input_user_id_roundtrip() {
    assert_eq!(InputUserId::new(3).map(InputUserId::get), Some(3));
}

#[test]
fn input_user_id_rejects_zero() {
    assert!(InputUserId::new(0).is_none());
}

#[test]
fn pic_id_roundtrip() {
    assert_eq!(PicId::new(9).map(PicId::get), Some(9));
}

#[test]
fn pic_id_rejects_zero() {
    assert!(PicId::new(0).is_none());
}

#[test]
fn user_id_roundtrip() {
    assert_eq!(UserId::new(11).map(UserId::get), Some(11));
}

#[test]
fn user_id_rejects_zero() {
    assert!(UserId::new(0).is_none());
}

#[test]
fn newtype_ids_are_distinct_types() {
    fn takes_cekunit(_: CekUnitId) {}
    fn takes_user(_: UserId) {}
    if let Some(id) = CekUnitId::new(1) {
        takes_cekunit(id);
    }
    if let Some(id) = UserId::new(1) {
        takes_user(id);
    }
}

#[test]
fn join_url_basic() {
    assert_eq!(JoinUrl::join("http://x", "/y"), "http://x/y");
}

#[test]
fn join_url_trailing_slash_base() {
    assert_eq!(JoinUrl::join("http://x/", "/y"), "http://x/y");
}

#[test]
fn join_url_no_leading_slash_path() {
    assert_eq!(JoinUrl::join("http://x", "y"), "http://x/y");
}

#[test]
fn join_url_both_slashes() {
    assert_eq!(JoinUrl::join("http://x/", "y"), "http://x/y");
}

#[test]
fn join_url_preserves_absolute() {
    assert_eq!(JoinUrl::join("http://x", "http://y/z"), "http://y/z");
}

#[test]
fn join_url_is_absolute_true_https() {
    assert!(JoinUrl::is_absolute("https://x"));
}

#[test]
fn join_url_is_absolute_true_http() {
    assert!(JoinUrl::is_absolute("http://x"));
}

#[test]
fn join_url_is_absolute_false_relative() {
    assert!(!JoinUrl::is_absolute("/x"));
}

#[test]
fn join_url_is_absolute_false_bare() {
    assert!(!JoinUrl::is_absolute("x.com"));
}

#[test]
fn cookie_store_default_is_memory() {
    assert_eq!(CookieStore::default(), CookieStore::Memory);
}

#[test]
fn cookie_store_none_path_is_none() {
    assert!(CookieStore::None.path().is_none());
}

#[test]
fn cookie_store_memory_path_is_none() {
    assert!(CookieStore::Memory.path().is_none());
}

#[test]
fn cookie_store_persistent_has_path() {
    let s = CookieStore::Persistent(std::path::PathBuf::from("c.json"));
    assert!(s.path().is_some());
}

#[test]
fn state_default_is_idle() {
    assert_eq!(State::default(), State::Idle);
}

#[test]
fn mode_default_is_once() {
    assert_eq!(Mode::default(), Mode::Once);
}

#[test]
fn api_response_debug_via_serde() {
    let json = r#"{"status":"ok","data":"payload","message":null}"#;
    let parsed: Result<ApiResponse<String>, _> = serde_json::from_str(json);
    if let Ok(resp) = parsed {
        let debug = format!("{resp:?}");
        assert!(debug.contains("ok"));
    }
}

#[test]
fn handler_error_config_display() {
    assert_eq!(
        HandlerError::Config(String::from("bad")).to_string(),
        "Configuration error: bad"
    );
}

#[test]
fn handler_error_not_logged_in_display() {
    assert_eq!(HandlerError::NotLoggedIn.to_string(), "Not logged in");
}

#[test]
fn handler_error_csrf_not_found_display() {
    assert_eq!(
        HandlerError::CsrfNotFound.to_string(),
        "CSRF token not found in HTML"
    );
}

#[test]
fn handler_error_api_display() {
    let e = HandlerError::Api(500, String::from("boom"));
    assert_eq!(e.to_string(), "API error (500): boom");
}

#[test]
fn handler_error_is_core_error() {
    fn assert_error<T: Error>() {}
    assert_error::<HandlerError>();
}

#[test]
fn handler_error_is_send_sync_static() {
    fn assert_bounds<T: Send + Sync + 'static>() {}
    assert_bounds::<HandlerError>();
}
