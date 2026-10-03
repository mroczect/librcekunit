extern crate alloc;
use alloc::collections::BTreeSet;
use core::cmp::Ordering;
use core::time::Duration;
use librcekunit_handler::{
    CekUnitId, Config, CookieStore, Error, HttpMethod, InputUserId, JoinUrl, Mode, PicId, State,
    UserId, Validate,
};
use serde as _;
use std::collections::HashSet;
use std::path::PathBuf;
use thiserror as _;

#[test]
fn config_new_defaults_timeout() {
    let config = Config::new("https://a.com");
    assert_eq!(config.timeout, Duration::from_secs(30));
}

#[test]
fn config_new_defaults_user_agent() {
    let config = Config::new("https://a.com");
    assert_eq!(config.user_agent, String::from("librcekunit/3.0"));
}

#[test]
fn config_new_defaults_cookie_store_is_persistent() {
    let config = Config::new("https://a.com");
    assert!(config.cookie_store.path().is_some());
}

#[test]
fn config_new_default_cookie_file_name() {
    let config = Config::new("https://a.com");
    let expected = PathBuf::from("librcekunit_cookies.json");
    assert_eq!(config.cookie_store.path(), Some(&expected));
}

#[test]
fn config_new_trims_single_trailing_slash() {
    let config = Config::new("https://a.com/");
    assert_eq!(config.base_url, String::from("https://a.com"));
}

#[test]
fn config_new_trims_many_trailing_slashes() {
    let config = Config::new("https://a.com////");
    assert_eq!(config.base_url, String::from("https://a.com"));
}

#[test]
fn config_new_empty_string_becomes_root() {
    let config = Config::new("");
    assert_eq!(config.base_url, String::from("/"));
}

#[test]
fn config_with_timeout_overrides() {
    let config = Config::new("https://a.com").with_timeout(120);
    assert_eq!(config.timeout, Duration::from_mins(2));
}

#[test]
fn config_with_timeout_replaces_default() {
    let config = Config::new("https://a.com").with_timeout(45);
    assert_eq!(config.timeout, Duration::from_secs(45));
}

#[test]
fn config_with_user_agent_overrides() {
    let config = Config::new("https://a.com").with_user_agent("custom/9");
    assert_eq!(config.user_agent, String::from("custom/9"));
}

#[test]
fn config_with_cookie_store_overrides_to_memory() {
    let config = Config::new("https://a.com").with_cookie_store(CookieStore::Memory);
    assert!(config.cookie_store.path().is_none());
}

#[test]
fn config_with_cookie_store_overrides_to_none() {
    let config = Config::new("https://a.com").with_cookie_store(CookieStore::None);
    assert!(config.cookie_store.path().is_none());
}

#[test]
fn config_builder_chain_all() {
    let config = Config::new("https://a.com")
        .with_timeout(60)
        .with_user_agent("x/1")
        .with_cookie_store(CookieStore::Memory);
    assert_eq!(config.base_url, String::from("https://a.com"));
    assert_eq!(config.timeout, Duration::from_mins(1));
    assert_eq!(config.user_agent, String::from("x/1"));
    assert!(config.cookie_store.path().is_none());
}

#[test]
fn config_validate_accepts_https() {
    assert!(Config::new("https://a.com").validate().is_ok());
}

#[test]
fn config_validate_accepts_http() {
    assert!(Config::new("http://localhost:8080").validate().is_ok());
}

#[test]
fn config_validate_rejects_whitespace_only() {
    assert!(Config::new("   ").validate().is_err());
}

#[test]
fn config_validate_rejects_relative_host() {
    assert!(Config::new("a.com").validate().is_err());
}

#[test]
fn config_validate_rejects_zero_timeout() {
    let config = Config::new("https://a.com").with_timeout(0);
    assert!(config.validate().is_err());
}

#[test]
fn config_validate_rejects_empty_user_agent() {
    let config = Config::new("https://a.com").with_user_agent("");
    assert!(config.validate().is_err());
}

#[test]
fn config_validate_rejects_ftp_scheme() {
    assert!(Config::new("ftp://a.com").validate().is_err());
}

#[test]
fn config_validate_error_is_config_variant() {
    let result = Config::new("ftp://a.com").validate();
    assert!(matches!(result, Err(Error::Config(_))));
}

#[test]
fn cekunit_id_new_accepts_one() {
    assert!(CekUnitId::new(1).is_some());
}

#[test]
fn cekunit_id_new_accepts_max() {
    assert!(CekUnitId::new(u64::MAX).is_some());
}

#[test]
fn cekunit_id_new_rejects_zero() {
    assert!(CekUnitId::new(0).is_none());
}

#[test]
fn input_user_id_new_accepts_positive() {
    assert!(InputUserId::new(100).is_some());
}

#[test]
fn input_user_id_new_rejects_zero() {
    assert!(InputUserId::new(0).is_none());
}

#[test]
fn pic_id_new_accepts_positive() {
    assert!(PicId::new(999).is_some());
}

#[test]
fn pic_id_new_rejects_zero() {
    assert!(PicId::new(0).is_none());
}

#[test]
fn user_id_new_accepts_positive() {
    assert!(UserId::new(1).is_some());
}

#[test]
fn user_id_new_rejects_zero() {
    assert!(UserId::new(0).is_none());
}

#[test]
fn cekunit_id_try_from_ok() {
    assert!(CekUnitId::try_from(5_u64).is_ok());
}

#[test]
fn cekunit_id_try_from_err() {
    assert!(CekUnitId::try_from(0_u64).is_err());
}

#[test]
fn input_user_id_try_from_ok() {
    assert!(InputUserId::try_from(7_u64).is_ok());
}

#[test]
fn input_user_id_try_from_err() {
    assert!(InputUserId::try_from(0_u64).is_err());
}

#[test]
fn pic_id_try_from_ok() {
    assert!(PicId::try_from(3_u64).is_ok());
}

#[test]
fn user_id_try_from_ok() {
    assert!(UserId::try_from(3_u64).is_ok());
}

#[test]
fn cekunit_id_get_roundtrip() {
    assert_eq!(CekUnitId::new(42).map(CekUnitId::get), Some(42));
}

#[test]
fn input_user_id_get_roundtrip() {
    assert_eq!(InputUserId::new(42).map(InputUserId::get), Some(42));
}

#[test]
fn pic_id_get_roundtrip() {
    assert_eq!(PicId::new(42).map(PicId::get), Some(42));
}

#[test]
fn user_id_get_roundtrip() {
    assert_eq!(UserId::new(42).map(UserId::get), Some(42));
}

#[test]
fn cekunit_id_is_valid_true() {
    assert_eq!(CekUnitId::new(1).map(CekUnitId::is_valid), Some(true));
}

#[test]
fn cekunit_id_display_renders_number() {
    assert_eq!(
        CekUnitId::new(42).map(|i| i.to_string()),
        Some(String::from("42"))
    );
}

#[test]
fn cekunit_id_ord() {
    let a = CekUnitId::new(1);
    let b = CekUnitId::new(2);
    let cmp = match (a, b) {
        (Some(a), Some(b)) => a.cmp(&b),
        _ => Ordering::Equal,
    };
    assert_eq!(cmp, Ordering::Less);
}

#[test]
fn cekunit_id_eq() {
    assert_eq!(CekUnitId::new(5), CekUnitId::new(5));
}

#[test]
fn cekunit_id_ne() {
    assert_ne!(CekUnitId::new(5), CekUnitId::new(6));
}

#[test]
fn cekunit_id_hash_in_hashset_dedups() {
    let mut set: HashSet<CekUnitId> = HashSet::new();
    if let Some(id) = CekUnitId::new(1) {
        let first = set.insert(id);
        let second = set.insert(id);
        assert!(first);
        assert!(!second);
    }
    assert_eq!(set.len(), 1);
}

#[test]
fn cekunit_id_ord_in_btreeset_sorts() {
    let mut set: BTreeSet<CekUnitId> = BTreeSet::new();
    for raw in [3_u64, 1, 2] {
        if let Some(id) = CekUnitId::new(raw) {
            let _ = set.insert(id);
        }
    }
    let sorted: Vec<u64> = set.iter().map(|i| i.get()).collect();
    assert_eq!(sorted, vec![1, 2, 3]);
}

#[test]
fn cekunit_id_copy() {
    let a = CekUnitId::new(1);
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn http_method_as_str_all() {
    assert_eq!(HttpMethod::GET.as_str(), "GET");
    assert_eq!(HttpMethod::POST.as_str(), "POST");
    assert_eq!(HttpMethod::PUT.as_str(), "PUT");
    assert_eq!(HttpMethod::PATCH.as_str(), "PATCH");
    assert_eq!(HttpMethod::DELETE.as_str(), "DELETE");
    assert_eq!(HttpMethod::HEAD.as_str(), "HEAD");
    assert_eq!(HttpMethod::OPTIONS.as_str(), "OPTIONS");
}

#[test]
fn http_method_display_all() {
    assert_eq!(HttpMethod::GET.to_string(), "GET");
    assert_eq!(HttpMethod::POST.to_string(), "POST");
    assert_eq!(HttpMethod::PUT.to_string(), "PUT");
    assert_eq!(HttpMethod::PATCH.to_string(), "PATCH");
    assert_eq!(HttpMethod::DELETE.to_string(), "DELETE");
    assert_eq!(HttpMethod::HEAD.to_string(), "HEAD");
    assert_eq!(HttpMethod::OPTIONS.to_string(), "OPTIONS");
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
fn http_method_post_has_body() {
    assert!(HttpMethod::POST.has_body());
}

#[test]
fn http_method_equality() {
    assert_eq!(HttpMethod::GET, HttpMethod::GET);
    assert_ne!(HttpMethod::GET, HttpMethod::POST);
}

#[test]
fn http_method_hashset() {
    let mut set: HashSet<HttpMethod> = HashSet::new();
    let _ = set.insert(HttpMethod::GET);
    let _ = set.insert(HttpMethod::GET);
    let _ = set.insert(HttpMethod::POST);
    assert_eq!(set.len(), 2);
}

#[test]
fn join_url_basic() {
    assert_eq!(
        JoinUrl::join("https://a.com", "/x"),
        String::from("https://a.com/x")
    );
}

#[test]
fn join_url_base_trailing_slash() {
    assert_eq!(
        JoinUrl::join("https://a.com/", "/x"),
        String::from("https://a.com/x")
    );
}

#[test]
fn join_url_path_no_leading_slash() {
    assert_eq!(
        JoinUrl::join("https://a.com", "x"),
        String::from("https://a.com/x")
    );
}

#[test]
fn join_url_both_slashes() {
    assert_eq!(
        JoinUrl::join("https://a.com/", "x"),
        String::from("https://a.com/x")
    );
}

#[test]
fn join_url_absolute_https() {
    assert_eq!(
        JoinUrl::join("https://a.com", "https://b.com/x"),
        String::from("https://b.com/x")
    );
}

#[test]
fn join_url_absolute_http() {
    assert_eq!(
        JoinUrl::join("https://a.com", "http://b.com/x"),
        String::from("http://b.com/x")
    );
}

#[test]
fn is_absolute_true_https() {
    assert!(JoinUrl::is_absolute("https://a.com"));
}

#[test]
fn is_absolute_true_http() {
    assert!(JoinUrl::is_absolute("http://a.com"));
}

#[test]
fn is_absolute_false_relative_path() {
    assert!(!JoinUrl::is_absolute("/relative"));
}

#[test]
fn is_absolute_false_bare_host() {
    assert!(!JoinUrl::is_absolute("a.com"));
}

#[test]
fn state_default_is_idle() {
    assert_eq!(State::default(), State::Idle);
}

#[test]
fn state_variants_distinct() {
    assert_ne!(State::Idle, State::Running);
    assert_ne!(State::Running, State::Stopped);
    assert_ne!(State::Stopped, State::Failed);
}

#[test]
fn mode_default_is_once() {
    assert_eq!(Mode::default(), Mode::Once);
}

#[test]
fn mode_variants_distinct() {
    assert_ne!(Mode::Once, Mode::Loop);
    assert_ne!(Mode::Loop, Mode::Stream);
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
fn cookie_store_persistent_path_is_some() {
    let store = CookieStore::Persistent(PathBuf::from("cookies.json"));
    assert!(store.path().is_some());
}

#[test]
fn cookie_store_persistent_path_matches() {
    let expected = PathBuf::from("cookies.json");
    let store = CookieStore::Persistent(expected.clone());
    assert_eq!(store.path(), Some(&expected));
}
