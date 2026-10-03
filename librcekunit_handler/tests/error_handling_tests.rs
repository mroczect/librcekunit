use core::error::Error as CoreError;
use librcekunit_handler::{Error, Result};
use serde as _;
use thiserror as _;

#[test]
fn display_config_variant() {
    let err = Error::Config(String::from("bad"));
    assert_eq!(err.to_string(), String::from("Configuration error: bad"));
}

#[test]
fn display_network_variant() {
    let err = Error::Network(String::from("timeout"));
    assert_eq!(err.to_string(), String::from("Network error: timeout"));
}

#[test]
fn display_io_variant() {
    let err = Error::Io(String::from("read failed"));
    assert_eq!(err.to_string(), String::from("IO error: read failed"));
}

#[test]
fn display_json_variant() {
    let err = Error::Json(String::from("parse failed"));
    assert_eq!(err.to_string(), String::from("JSON error: parse failed"));
}

#[test]
fn display_auth_variant() {
    let err = Error::Auth(String::from("invalid credentials"));
    assert_eq!(
        err.to_string(),
        String::from("Authentication failed: invalid credentials")
    );
}

#[test]
fn display_api_variant_400() {
    let err = Error::Api(400, String::from("bad request"));
    assert_eq!(
        err.to_string(),
        String::from("API error (400): bad request")
    );
}

#[test]
fn display_api_variant_404() {
    let err = Error::Api(404, String::from("not found"));
    assert_eq!(err.to_string(), String::from("API error (404): not found"));
}

#[test]
fn display_api_variant_500() {
    let err = Error::Api(500, String::from("internal"));
    assert_eq!(err.to_string(), String::from("API error (500): internal"));
}

#[test]
fn display_api_variant_503() {
    let err = Error::Api(503, String::from("unavailable"));
    assert_eq!(
        err.to_string(),
        String::from("API error (503): unavailable")
    );
}

#[test]
fn display_api_variant_empty_message() {
    let err = Error::Api(200, String::new());
    assert_eq!(err.to_string(), String::from("API error (200): "));
}

#[test]
fn display_csrf_variant() {
    let err = Error::CsrfNotFound;
    assert_eq!(
        err.to_string(),
        String::from("CSRF token not found in HTML")
    );
}

#[test]
fn display_not_logged_in_variant() {
    let err = Error::NotLoggedIn;
    assert_eq!(err.to_string(), String::from("Not logged in"));
}

#[test]
fn display_cookie_store_variant() {
    let err = Error::CookieStore(String::from("disk full"));
    assert_eq!(
        err.to_string(),
        String::from("Cookie store error: disk full")
    );
}

#[test]
fn debug_config_reveals_variant_and_payload() {
    let err = Error::Config(String::from("abc"));
    let debug = format!("{err:?}");
    assert!(debug.contains("Config"));
    assert!(debug.contains("abc"));
}

#[test]
fn debug_api_reveals_status_and_message() {
    let err = Error::Api(418, String::from("teapot"));
    let debug = format!("{err:?}");
    assert!(debug.contains("Api"));
    assert!(debug.contains("418"));
    assert!(debug.contains("teapot"));
}

#[test]
fn debug_csrf_reveals_variant() {
    let err = Error::CsrfNotFound;
    let debug = format!("{err:?}");
    assert!(debug.contains("CsrfNotFound"));
}

#[test]
fn debug_not_logged_in_reveals_variant() {
    let err = Error::NotLoggedIn;
    let debug = format!("{err:?}");
    assert!(debug.contains("NotLoggedIn"));
}

#[test]
fn error_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<Error>();
}

#[test]
fn error_is_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<Error>();
}

#[test]
fn error_is_static() {
    fn assert_static<T: 'static>() {}
    assert_static::<Error>();
}

#[test]
fn error_is_core_error() {
    fn assert_error<T: CoreError>() {}
    assert_error::<Error>();
}

#[test]
fn error_boxes_as_dyn_core_error() {
    let err: Box<dyn CoreError + Send + Sync> = Box::new(Error::NotLoggedIn);
    assert_eq!(err.to_string(), String::from("Not logged in"));
}
#[test]
fn error_can_be_returned_from_fn() {
    fn run(fail: bool) -> Result<()> {
        if fail {
            Err(Error::NotLoggedIn)
        } else {
            Ok(())
        }
    }
    let err_result = run(true);
    assert!(err_result.is_err());
    assert!(matches!(err_result, Err(Error::NotLoggedIn)));

    let ok_result = run(false);
    assert!(ok_result.is_ok());
}

#[test]
fn value_can_be_returned_from_fn() {
    fn run(fail: bool) -> Result<u32> {
        if fail { Err(Error::NotLoggedIn) } else { Ok(7) }
    }
    assert_eq!(run(false).ok(), Some(7));
    assert!(run(true).is_err());
}

#[test]
fn result_alias_default_error_ok() {
    let value: Result<u32> = Ok(7);
    assert_eq!(value.ok(), Some(7));
}

#[test]
fn result_alias_default_error_err() {
    let value: Result<u32> = Err(Error::NotLoggedIn);
    assert!(value.is_err());
}

#[test]
fn result_alias_custom_error() {
    let value: Result<u32, &'static str> = Err("custom");
    assert!(value.is_err());
}

#[test]
fn result_alias_unit_ok() {
    let value: Result<()> = Ok(());
    assert!(value.is_ok());
}

#[test]
fn matches_config_variant() {
    let err = Error::Config(String::from("x"));
    assert!(matches!(err, Error::Config(_)));
}

#[test]
fn matches_network_variant() {
    let err = Error::Network(String::from("x"));
    assert!(matches!(err, Error::Network(_)));
}

#[test]
fn matches_io_variant() {
    let err = Error::Io(String::from("x"));
    assert!(matches!(err, Error::Io(_)));
}

#[test]
fn matches_json_variant() {
    let err = Error::Json(String::from("x"));
    assert!(matches!(err, Error::Json(_)));
}

#[test]
fn matches_auth_variant() {
    let err = Error::Auth(String::from("x"));
    assert!(matches!(err, Error::Auth(_)));
}

#[test]
fn matches_api_variant() {
    let err = Error::Api(1, String::from("x"));
    assert!(matches!(err, Error::Api(1, _)));
}

#[test]
fn matches_csrf_variant() {
    let err = Error::CsrfNotFound;
    assert!(matches!(err, Error::CsrfNotFound));
}

#[test]
fn matches_not_logged_in_variant() {
    let err = Error::NotLoggedIn;
    assert!(matches!(err, Error::NotLoggedIn));
}

#[test]
fn matches_cookie_store_variant() {
    let err = Error::CookieStore(String::from("x"));
    assert!(matches!(err, Error::CookieStore(_)));
}

#[test]
fn extract_api_status_and_message() {
    let err = Error::Api(404, String::from("gone"));
    if let Error::Api(status, msg) = err {
        assert_eq!(status, 404);
        assert_eq!(msg, String::from("gone"));
    }
}

#[test]
fn extract_config_message() {
    let err = Error::Config(String::from("details"));
    if let Error::Config(msg) = err {
        assert_eq!(msg, String::from("details"));
    }
}

#[test]
fn vec_of_results_all_ok() {
    let items: [Result<u32>; 3] = [Ok(1), Ok(2), Ok(3)];
    let all_ok = items.iter().all(Result::is_ok);
    assert!(all_ok);
}

#[test]
fn vec_of_results_some_err() {
    let items: [Result<u32>; 3] = [Ok(1), Err(Error::NotLoggedIn), Ok(3)];
    let err_count = items.iter().filter(|r| r.is_err()).count();
    assert_eq!(err_count, 1);
}
