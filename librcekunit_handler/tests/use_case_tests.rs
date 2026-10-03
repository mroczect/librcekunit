use librcekunit_handler::{
    CekUnitId, Config, CookieStore, Error, HttpMethod, InputUserId, JoinUrl, PicId, UserId,
    Validate,
};
use serde as _;
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror as _;

#[test]
fn scenario_setup_client_with_full_config() {
    let config = Config::new("https://campus.example.edu")
        .with_timeout(45)
        .with_user_agent("librcekunit-test/1.0")
        .with_cookie_store(CookieStore::Persistent(PathBuf::from("test_cookies.json")));

    assert!(config.validate().is_ok());
    assert_eq!(config.base_url, String::from("https://campus.example.edu"));
    assert_eq!(config.user_agent, String::from("librcekunit-test/1.0"));
    assert!(config.cookie_store.path().is_some());
}

#[test]
fn scenario_validate_then_build_cekunit_id() {
    let config = Config::new("https://campus.example.edu");
    assert!(config.validate().is_ok());

    let id = CekUnitId::try_from(7);
    assert!(id.is_ok());
    if let Ok(id) = id {
        assert_eq!(id.get(), 7);
    }
}

#[test]
fn scenario_validate_then_build_input_user_id() {
    let config = Config::new("https://campus.example.edu");
    assert!(config.validate().is_ok());

    let id = InputUserId::try_from(11);
    assert!(id.is_ok());
}

#[test]
fn scenario_validate_then_build_pic_id() {
    let config = Config::new("https://campus.example.edu");
    assert!(config.validate().is_ok());

    let id = PicId::try_from(13);
    assert!(id.is_ok());
}

#[test]
fn scenario_validate_then_build_user_id() {
    let config = Config::new("https://campus.example.edu");
    assert!(config.validate().is_ok());

    let id = UserId::try_from(17);
    assert!(id.is_ok());
}

#[test]
fn scenario_join_cekunit_export_url() {
    let config = Config::new("https://campus.example.edu");
    let url = JoinUrl::join(&config.base_url, "/dashboard/cekunit/export");
    assert_eq!(
        url,
        String::from("https://campus.example.edu/dashboard/cekunit/export")
    );
}

#[test]
fn scenario_join_cekunit_delete_by_category_url() {
    let config = Config::new("https://campus.example.edu");
    let url = JoinUrl::join(&config.base_url, "/dashboard/cekunit/delete-by-category");
    assert_eq!(
        url,
        String::from("https://campus.example.edu/dashboard/cekunit/delete-by-category")
    );
}

#[test]
fn scenario_join_input_data_url() {
    let config = Config::new("https://campus.example.edu");
    let url = JoinUrl::join(&config.base_url, "/dashboard/input-data");
    assert_eq!(
        url,
        String::from("https://campus.example.edu/dashboard/input-data")
    );
}

#[test]
fn scenario_join_pic_input_url() {
    let config = Config::new("https://campus.example.edu");
    let url = JoinUrl::join(&config.base_url, "/dashboard/input-PIC");
    assert_eq!(
        url,
        String::from("https://campus.example.edu/dashboard/input-PIC")
    );
}

#[test]
fn scenario_http_method_classification_complete() {
    let reads: [HttpMethod; 3] = [HttpMethod::GET, HttpMethod::HEAD, HttpMethod::OPTIONS];
    let writes: [HttpMethod; 4] = [
        HttpMethod::POST,
        HttpMethod::PUT,
        HttpMethod::PATCH,
        HttpMethod::DELETE,
    ];

    for method in reads {
        assert!(!method.requires_csrf());
        assert!(!method.has_body());
    }
    for method in writes {
        assert!(method.requires_csrf());
        assert!(method.has_body());
    }
}

#[test]
fn scenario_csrf_required_count() {
    let methods: [HttpMethod; 7] = [
        HttpMethod::GET,
        HttpMethod::POST,
        HttpMethod::PUT,
        HttpMethod::PATCH,
        HttpMethod::DELETE,
        HttpMethod::HEAD,
        HttpMethod::OPTIONS,
    ];
    let csrf_required = methods.iter().filter(|m| m.requires_csrf()).count();
    assert_eq!(csrf_required, 4);
}

#[test]
fn scenario_reject_invalid_scheme() {
    let config = Config::new("ftp://bad.example.com");
    let result = config.validate();
    assert!(result.is_err());
    if let Err(Error::Config(msg)) = result {
        assert!(msg.contains("http://"));
    }
}

#[test]
fn scenario_reject_empty_base_url_message() {
    let config = Config::new("   ");
    let result = config.validate();
    assert!(result.is_err());
    if let Err(Error::Config(msg)) = result {
        assert!(msg.contains("empty"));
    }
}

#[test]
fn scenario_reject_empty_user_agent_message() {
    let config = Config::new("https://a.com").with_user_agent("");
    let result = config.validate();
    assert!(result.is_err());
    if let Err(Error::Config(msg)) = result {
        assert!(msg.contains("user_agent"));
    }
}

#[test]
fn scenario_reject_zero_timeout_message() {
    let config = Config::new("https://a.com").with_timeout(0);
    let result = config.validate();
    assert!(result.is_err());
    if let Err(Error::Config(msg)) = result {
        assert!(msg.contains("timeout"));
    }
}

#[test]
fn scenario_reject_zero_cekunit_id() {
    assert!(CekUnitId::new(0).is_none());
}

#[test]
fn scenario_reject_zero_input_user_id() {
    assert!(InputUserId::new(0).is_none());
}

#[test]
fn scenario_reject_zero_pic_id() {
    assert!(PicId::new(0).is_none());
}

#[test]
fn scenario_reject_zero_user_id() {
    assert!(UserId::new(0).is_none());
}

#[test]
fn scenario_cookie_store_all_variants_paths() {
    let none = CookieStore::None;
    let memory = CookieStore::Memory;
    let persistent = CookieStore::Persistent(PathBuf::from("/tmp/cookies.json"));

    assert!(none.path().is_none());
    assert!(memory.path().is_none());
    assert!(persistent.path().is_some());
}

#[test]
fn scenario_build_export_query_params() {
    let mut params: HashMap<String, String> = HashMap::new();
    let first = params.insert(String::from("format"), String::from("csv"));
    let second = params.insert(String::from("sort"), String::from("nomor"));
    let third = params.insert(String::from("direction"), String::from("asc"));

    assert!(first.is_none());
    assert!(second.is_none());
    assert!(third.is_none());
    assert_eq!(params.len(), 3);
    assert_eq!(params.get("format").map(String::as_str), Some("csv"));
    assert_eq!(params.get("sort").map(String::as_str), Some("nomor"));
    assert_eq!(params.get("direction").map(String::as_str), Some("asc"));
}

#[test]
fn scenario_build_delete_by_category_params() {
    let mut params: HashMap<String, String> = HashMap::new();
    let first = params.insert(String::from("column"), String::from("category"));
    let second = params.insert(String::from("value"), String::from("A"));
    let third = params.insert(String::from("_method"), String::from("DELETE"));

    assert!(first.is_none());
    assert!(second.is_none());
    assert!(third.is_none());
    assert_eq!(params.get("column").map(String::as_str), Some("category"));
    assert_eq!(params.get("value").map(String::as_str), Some("A"));
    assert_eq!(params.get("_method").map(String::as_str), Some("DELETE"));
}

#[test]
fn scenario_login_credentials_shape() {
    let mut form: HashMap<String, String> = HashMap::new();
    let token = form.insert(String::from("_token"), String::from("csrf-abc"));
    let email = form.insert(String::from("email"), String::from("user@example.com"));
    let password = form.insert(String::from("password"), String::from("secret"));

    assert!(token.is_none());
    assert!(email.is_none());
    assert!(password.is_none());
    assert_eq!(form.len(), 3);
}

#[test]
fn scenario_full_request_url_end_to_end() {
    let config = Config::new("https://campus.example.edu").with_cookie_store(CookieStore::Memory);

    assert!(config.validate().is_ok());

    let id = CekUnitId::try_from(42);
    assert!(id.is_ok());

    let path = id.map_or_else(
        |_| String::from("/cekunit"),
        |id| format!("/cekunit/{id}/edit"),
    );

    let url = JoinUrl::join(&config.base_url, &path);
    assert_eq!(
        url,
        String::from("https://campus.example.edu/cekunit/42/edit")
    );
}
