use librcekunit_handler::{
    CekUnitId, Config, CookieStore, HttpMethod, InputUserId, JoinUrl, PicId, UserId,
};
use serde as _;
use thiserror as _;

#[test]
fn all_positive_ids_constructible() {
    let cases: [u64; 9] = [1, 2, 3, 10, 100, 1_000, 10_000, 100_000, u64::MAX];
    for raw in cases {
        assert!(CekUnitId::new(raw).is_some());
        assert!(InputUserId::new(raw).is_some());
        assert!(PicId::new(raw).is_some());
        assert!(UserId::new(raw).is_some());
    }
}

#[test]
fn zero_id_always_rejected() {
    assert!(CekUnitId::new(0).is_none());
    assert!(InputUserId::new(0).is_none());
    assert!(PicId::new(0).is_none());
    assert!(UserId::new(0).is_none());
}

#[test]
fn id_get_roundtrip() {
    let cases: [u64; 8] = [1, 2, 42, 999, 1_000, 10_000, 100_000, u64::MAX];
    for raw in cases {
        assert_eq!(CekUnitId::new(raw).map(CekUnitId::get), Some(raw));
        assert_eq!(InputUserId::new(raw).map(InputUserId::get), Some(raw));
        assert_eq!(PicId::new(raw).map(PicId::get), Some(raw));
        assert_eq!(UserId::new(raw).map(UserId::get), Some(raw));
    }
}

#[test]
fn id_try_from_roundtrip() {
    let cases: [u64; 8] = [1, 2, 42, 999, 1_000, 10_000, 100_000, u64::MAX];
    for raw in cases {
        assert_eq!(CekUnitId::try_from(raw).ok().map(CekUnitId::get), Some(raw));
    }
}

#[test]
fn id_display_matches_raw() {
    let cases: [u64; 5] = [1, 42, 999, 100_000, u64::MAX];
    for raw in cases {
        assert_eq!(
            CekUnitId::new(raw).map(|i| i.to_string()),
            Some(raw.to_string())
        );
    }
}

#[test]
fn id_is_valid_true_for_all_positive() {
    let cases: [u64; 5] = [1, 2, 42, 999, u64::MAX];
    for raw in cases {
        assert_eq!(CekUnitId::new(raw).map(CekUnitId::is_valid), Some(true));
    }
}

#[test]
fn join_url_absolute_passthrough() {
    let cases: [&str; 4] = [
        "https://a.com/x",
        "http://b.com/y",
        "https://c.com",
        "http://d.com/",
    ];
    for url in cases {
        assert_eq!(JoinUrl::join("https://base.com", url), url);
    }
}

#[test]
fn join_url_slash_variants() {
    let cases: [(&str, &str, &str); 4] = [
        ("https://a.com", "/x", "https://a.com/x"),
        ("https://a.com/", "/x", "https://a.com/x"),
        ("https://a.com", "x", "https://a.com/x"),
        ("https://a.com/", "x", "https://a.com/x"),
    ];
    for (base, path, expected) in cases {
        assert_eq!(JoinUrl::join(base, path), expected);
    }
}

#[test]
fn join_url_no_double_slash_after_base() {
    let cases: [(&str, &str); 4] = [
        ("https://a.com/", "/x"),
        ("https://a.com//", "//x"),
        ("https://a.com", "/x"),
        ("https://a.com", "x"),
    ];
    for (base, path) in cases {
        let joined = JoinUrl::join(base, path);
        assert!(!joined.contains("//x"));
    }
}

#[test]
fn join_url_result_starts_with_base() {
    let cases: [(&str, &str); 3] = [
        ("https://a.com", "/x"),
        ("https://b.com", "y"),
        ("http://c.com", "/z"),
    ];
    for (base, path) in cases {
        let joined = JoinUrl::join(base, path);
        assert!(joined.starts_with(base));
    }
}

#[test]
fn config_trim_strips_trailing_slashes() {
    let cases: [(&str, &str); 5] = [
        ("https://a.com", "https://a.com"),
        ("https://a.com/", "https://a.com"),
        ("https://a.com//", "https://a.com"),
        ("https://a.com///", "https://a.com"),
        ("http://b.com/", "http://b.com"),
    ];
    for (input, expected) in cases {
        assert_eq!(Config::new(input).base_url, expected);
    }
}

#[test]
fn config_new_never_panics_on_any_input() {
    let cases: [&str; 8] = [
        "",
        " ",
        "   ",
        "x",
        "https://",
        "http://",
        "///",
        "https://a.com////",
    ];
    for input in cases {
        let _ = Config::new(input);
    }
}

#[test]
fn config_default_timeout_is_positive() {
    let config = Config::new("https://a.com");
    assert!(!config.timeout.is_zero());
}

#[test]
fn config_default_user_agent_non_empty() {
    let config = Config::new("https://a.com");
    assert!(!config.user_agent.is_empty());
}

#[test]
fn config_cookie_store_default_is_persistent() {
    let config = Config::new("https://a.com");
    let is_persistent = matches!(config.cookie_store, CookieStore::Persistent(_));
    assert!(is_persistent);
}

#[test]
fn http_method_read_classification_consistent() {
    let read_methods: [HttpMethod; 3] = [HttpMethod::GET, HttpMethod::HEAD, HttpMethod::OPTIONS];
    for method in read_methods {
        assert!(!method.requires_csrf());
        assert!(!method.has_body());
    }
}

#[test]
fn http_method_write_classification_consistent() {
    let write_methods: [HttpMethod; 4] = [
        HttpMethod::POST,
        HttpMethod::PUT,
        HttpMethod::PATCH,
        HttpMethod::DELETE,
    ];
    for method in write_methods {
        assert!(method.requires_csrf());
        assert!(method.has_body());
    }
}

#[test]
fn http_method_as_str_uppercase() {
    let all: [HttpMethod; 7] = [
        HttpMethod::GET,
        HttpMethod::POST,
        HttpMethod::PUT,
        HttpMethod::PATCH,
        HttpMethod::DELETE,
        HttpMethod::HEAD,
        HttpMethod::OPTIONS,
    ];
    for method in all {
        let s = method.as_str();
        assert_eq!(s, s.to_uppercase());
    }
}

#[test]
fn http_method_display_equals_as_str() {
    let all: [HttpMethod; 7] = [
        HttpMethod::GET,
        HttpMethod::POST,
        HttpMethod::PUT,
        HttpMethod::PATCH,
        HttpMethod::DELETE,
        HttpMethod::HEAD,
        HttpMethod::OPTIONS,
    ];
    for method in all {
        assert_eq!(method.to_string(), method.as_str());
    }
}

#[test]
fn http_method_as_str_unique() {
    let all: [HttpMethod; 7] = [
        HttpMethod::GET,
        HttpMethod::POST,
        HttpMethod::PUT,
        HttpMethod::PATCH,
        HttpMethod::DELETE,
        HttpMethod::HEAD,
        HttpMethod::OPTIONS,
    ];
    let mut seen: Vec<&str> = Vec::new();
    for method in all {
        let s = method.as_str();
        assert!(!seen.contains(&s));
        seen.push(s);
    }
    assert_eq!(seen.len(), 7);
}

#[test]
fn join_url_is_absolute_consistent_with_passthrough() {
    let cases: [&str; 4] = ["https://a.com", "http://b.com", "/rel", "bare"];
    for url in cases {
        let joined = JoinUrl::join("https://base.com", url);
        if JoinUrl::is_absolute(url) {
            assert_eq!(joined, url);
        } else {
            assert!(joined.starts_with("https://base.com"));
        }
    }
}
