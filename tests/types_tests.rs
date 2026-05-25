use librcekunit::types::{ApiResponse, HttpMethod, join_url};
use serde_json;

#[test]
fn test_http_method_as_str_all_variants() {
    assert_eq!(HttpMethod::GET.as_str(), "GET");
    assert_eq!(HttpMethod::POST.as_str(), "POST");
    assert_eq!(HttpMethod::PUT.as_str(), "PUT");
    assert_eq!(HttpMethod::PATCH.as_str(), "PATCH");
    assert_eq!(HttpMethod::DELETE.as_str(), "DELETE");
    assert_eq!(HttpMethod::HEAD.as_str(), "HEAD");
    assert_eq!(HttpMethod::OPTIONS.as_str(), "OPTIONS");
}

#[test]
fn test_http_method_display() {
    assert_eq!(format!("{}", HttpMethod::GET), "GET");
    assert_eq!(format!("{}", HttpMethod::POST), "POST");
    assert_eq!(format!("{}", HttpMethod::DELETE), "DELETE");
}

#[test]
fn test_http_method_equality() {
    assert_eq!(HttpMethod::GET, HttpMethod::GET);
    assert_ne!(HttpMethod::GET, HttpMethod::POST);
}

#[test]
fn test_http_method_clone() {
    let method = HttpMethod::PATCH;
    let cloned = method;
    assert_eq!(method, cloned);
}

#[test]
fn test_http_method_copy_semantics() {
    let m1 = HttpMethod::PUT;
    let m2 = m1;
    assert_eq!(m1, m2);
}

#[test]
fn test_join_url_base_no_trailing_slash() {
    assert_eq!(
        join_url("http://example.com", "/path"),
        "http://example.com/path"
    );
}

#[test]
fn test_join_url_base_with_trailing_slash() {
    assert_eq!(
        join_url("http://example.com/", "/path"),
        "http://example.com/path"
    );
}

#[test]
fn test_join_url_path_without_leading_slash() {
    assert_eq!(
        join_url("http://example.com", "path"),
        "http://example.com/path"
    );
}

#[test]
fn test_join_url_both_slashes() {
    assert_eq!(
        join_url("http://example.com/", "path"),
        "http://example.com/path"
    );
}

#[test]
fn test_join_url_empty_path() {
    assert_eq!(join_url("http://example.com", ""), "http://example.com/");
    assert_eq!(join_url("http://example.com/", ""), "http://example.com/");
}

#[test]
fn test_join_url_root_path() {
    assert_eq!(join_url("http://example.com", "/"), "http://example.com/");
}

#[test]
fn test_join_url_path_is_full_url() {
    let full = "https://other.com/api";
    assert_eq!(join_url("http://example.com", full), full);
}

#[test]
fn test_join_url_base_with_multiple_trailing_slashes() {
    assert_eq!(
        join_url("http://example.com///", "/path"),
        "http://example.com/path"
    );
}

#[test]
fn test_join_url_path_with_multiple_leading_slashes() {
    assert_eq!(
        join_url("http://example.com", "//path"),
        "http://example.com/path"
    );
    assert_eq!(
        join_url("http://example.com", "///path"),
        "http://example.com/path"
    );
}

#[test]
fn test_api_response_serialize_with_data() {
    let response = ApiResponse {
        status: "success".into(),
        data: Some(vec!["item1", "item2"]),
        message: None,
    };
    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"status\":\"success\""));
    assert!(json.contains("\"data\":[\"item1\",\"item2\"]"));
    assert!(json.contains("\"message\":null"));
}

#[test]
fn test_api_response_deserialize_with_data() {
    let json = r#"{"status":"ok","data":[1,2,3],"message":"done"}"#;
    let resp: ApiResponse<Vec<i32>> = serde_json::from_str(json).unwrap();
    assert_eq!(resp.status, "ok");
    assert_eq!(resp.data, Some(vec![1, 2, 3]));
    assert_eq!(resp.message, Some("done".into()));
}

#[test]
fn test_api_response_deserialize_null_data() {
    let json = r#"{"status":"error","data":null,"message":"not found"}"#;
    let resp: ApiResponse<String> = serde_json::from_str(json).unwrap();
    assert_eq!(resp.status, "error");
    assert_eq!(resp.data, None);
    assert_eq!(resp.message, Some("not found".into()));
}

#[test]
fn test_api_response_missing_optional_fields() {
    let json = r#"{"status":"partial"}"#;
    let resp: ApiResponse<()> = serde_json::from_str(json).unwrap();
    assert_eq!(resp.status, "partial");
    assert_eq!(resp.data, None);
    assert_eq!(resp.message, None);
}

#[test]
fn test_api_response_roundtrip() {
    let original = ApiResponse {
        status: "ok".into(),
        data: Some(42),
        message: Some("all good".into()),
    };
    let json = serde_json::to_string(&original).unwrap();
    let parsed: ApiResponse<i32> = serde_json::from_str(&json).unwrap();
    assert_eq!(original.status, parsed.status);
    assert_eq!(original.data, parsed.data);
    assert_eq!(original.message, parsed.message);
}
