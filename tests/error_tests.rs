use librcekunit::Error;
use std::io;

#[test]
fn test_error_config_display() {
    let err = Error::Config("invalid url".into());
    assert_eq!(format!("{}", err), "Configuration error: invalid url");
}

#[tokio::test]
async fn test_error_reqwest_display() {
    let req_err = reqwest::get("http://[:::1]").await.unwrap_err();
    let err: Error = req_err.into();
    let msg = format!("{}", err);
    assert!(msg.contains("Network error:"));
}

#[test]
fn test_error_io_display() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file missing");
    let err: Error = io_err.into();
    assert_eq!(format!("{}", err), "IO error: file missing");
}

#[test]
fn test_error_json_display() {
    let json_err = serde_json::from_str::<i32>("not a number").unwrap_err();
    let err: Error = json_err.into();
    let msg = format!("{}", err);
    assert!(msg.starts_with("JSON error:"));
}

#[test]
fn test_error_auth_display() {
    let err = Error::Auth("bad credentials".into());
    assert_eq!(format!("{}", err), "Authentication failed: bad credentials");
}

#[test]
fn test_error_api_display() {
    let err = Error::Api(404, "not found".into());
    assert_eq!(format!("{}", err), "API error (404): not found");
}

#[test]
fn test_error_csrf_not_found_display() {
    assert_eq!(
        format!("{}", Error::CsrfNotFound),
        "CSRF token not found in HTML"
    );
}

#[test]
fn test_error_not_logged_in_display() {
    assert_eq!(format!("{}", Error::NotLoggedIn), "Not logged in");
}

#[test]
fn test_error_cookie_store_display() {
    let err = Error::CookieStore("cannot write".into());
    assert_eq!(format!("{}", err), "Cookie store error: cannot write");
}

#[tokio::test]
async fn test_from_reqwest_error() {
    let req_err = reqwest::get("http://[:::1]").await.unwrap_err();
    let error: Error = req_err.into();
    match error {
        Error::Reqwest(_) => {}
        other => panic!("Expected Error::Reqwest, got {:?}", other),
    }
}

#[test]
fn test_from_io_error() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
    let error: Error = io_err.into();
    match error {
        Error::Io(_) => {}
        other => panic!("Expected Error::Io, got {:?}", other),
    }
}

#[test]
fn test_from_serde_json_error() {
    let json_err = serde_json::from_str::<serde_json::Value>("{bad}").unwrap_err();
    let error: Error = json_err.into();
    match error {
        Error::Json(_) => {}
        other => panic!("Expected Error::Json, got {:?}", other),
    }
}

#[tokio::test]
async fn test_reqwest_error_source() {
    let req_err = reqwest::get("http://[:::1]").await.unwrap_err();
    let error: Error = req_err.into();
    let source = std::error::Error::source(&error);
    assert!(source.is_some(), "Reqwest error should have a source");
}

#[test]
fn test_io_error_source() {
    let io_err = io::Error::new(io::ErrorKind::BrokenPipe, "pipe");
    let error: Error = io_err.into();
    assert!(std::error::Error::source(&error).is_some());
}

#[test]
fn test_json_error_source() {
    let json_err = serde_json::from_str::<i32>("abc").unwrap_err();
    let error: Error = json_err.into();
    assert!(std::error::Error::source(&error).is_some());
}

#[test]
fn test_non_wrapped_errors_have_no_source() {
    let errors = vec![
        Error::Config("x".into()),
        Error::Auth("x".into()),
        Error::Api(500, "x".into()),
        Error::CsrfNotFound,
        Error::NotLoggedIn,
        Error::CookieStore("x".into()),
    ];
    for err in errors {
        assert!(
            std::error::Error::source(&err).is_none(),
            "{:?} should not have a source",
            err
        );
    }
}

#[test]
fn test_error_debug_contains_message() {
    let err = Error::Auth("pwd wrong".into());
    let debug = format!("{:?}", err);
    assert!(debug.contains("Auth"), "Debug must include variant name");
    assert!(
        debug.contains("pwd wrong"),
        "Debug must include inner message"
    );
}

#[test]
fn test_api_error_debug() {
    let err = Error::Api(500, "internal".into());
    let debug = format!("{:?}", err);
    assert!(debug.contains("Api") && debug.contains("500") && debug.contains("internal"));
}

#[test]
fn test_match_auth_error() {
    let err = Error::Auth("test".to_string());
    match err {
        Error::Auth(msg) => assert_eq!(msg, "test"),
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn test_match_api_error() {
    let err = Error::Api(403, "forbidden".into());
    match err {
        Error::Api(code, msg) => {
            assert_eq!(code, 403);
            assert_eq!(msg, "forbidden");
        }
        _ => panic!("Wrong variant"),
    }
}
