use librcekunit::{Config, CookieStore};
use serial_test::serial;
use std::env;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;

#[test]
fn test_config_new_basic() {
    let cfg = Config::new("http://example.com");
    assert_eq!(cfg.base_url, "http://example.com");
    assert_eq!(cfg.timeout, Duration::from_secs(30));
    assert_eq!(cfg.user_agent, "librcekunit/2.0");
    match cfg.cookie_store {
        CookieStore::Persistent(ref path) => {
            assert_eq!(path, &PathBuf::from("librcekunit_cookies.json"));
        }
        _ => panic!("Default cookie store should be Persistent"),
    }
}

#[test]
fn test_config_new_trim_trailing_slash() {
    let cfg = Config::new("http://example.com/");
    assert_eq!(cfg.base_url, "http://example.com");
}

#[test]
fn test_config_new_multiple_slashes() {
    let cfg = Config::new("http://example.com///");
    assert_eq!(cfg.base_url, "http://example.com");
}

#[test]
fn test_config_new_no_slash() {
    let cfg = Config::new("http://example.com/path");
    assert_eq!(cfg.base_url, "http://example.com/path");
}

#[test]
fn test_config_new_empty_base() {
    let cfg = Config::new("");
    assert_eq!(cfg.base_url, "");
}

#[test]
fn test_with_timeout() {
    let cfg = Config::new("http://x.com").with_timeout(10);
    assert_eq!(cfg.timeout, Duration::from_secs(10));
}

#[test]
fn test_with_timeout_zero() {
    let cfg = Config::new("http://x.com").with_timeout(0);
    assert_eq!(cfg.timeout, Duration::from_secs(0));
}

#[test]
fn test_with_timeout_chained() {
    let cfg = Config::new("http://x.com").with_timeout(5).with_timeout(15);
    assert_eq!(cfg.timeout, Duration::from_secs(15));
}

#[test]
fn test_with_user_agent() {
    let cfg = Config::new("http://x.com").with_user_agent("my-agent/1.0");
    assert_eq!(cfg.user_agent, "my-agent/1.0");
}

#[test]
fn test_with_user_agent_empty() {
    let cfg = Config::new("http://x.com").with_user_agent("");
    assert_eq!(cfg.user_agent, "");
}

#[test]
fn test_with_cookie_store_none() {
    let cfg = Config::new("http://x.com").with_cookie_store(CookieStore::None);
    match cfg.cookie_store {
        CookieStore::None => {}
        _ => panic!("Expected None"),
    }
}

#[test]
fn test_with_cookie_store_memory() {
    let cfg = Config::new("http://x.com").with_cookie_store(CookieStore::Memory);
    match cfg.cookie_store {
        CookieStore::Memory => {}
        _ => panic!("Expected Memory"),
    }
}

#[test]
fn test_with_cookie_store_persistent() {
    let path = PathBuf::from("/tmp/my_cookies.json");
    let cfg = Config::new("http://x.com").with_cookie_store(CookieStore::Persistent(path.clone()));
    match cfg.cookie_store {
        CookieStore::Persistent(ref p) => assert_eq!(p, &path),
        _ => panic!("Expected Persistent"),
    }
}

#[test]
#[serial]
fn test_from_env_with_base_url_set() {
    let original = env::var("BASE_URL").ok();
    unsafe {
        env::set_var("BASE_URL", "https://example.com/api");
    }
    let result = Config::from_env();

    match original {
        Some(val) => unsafe {
            env::set_var("BASE_URL", val);
        },
        None => unsafe {
            env::remove_var("BASE_URL");
        },
    }
    assert!(result.is_ok());
    let cfg = result.unwrap();
    assert_eq!(cfg.base_url, "https://example.com/api");
}

#[test]
#[serial]
fn test_from_env_missing_base_url() {
    let original_dir = env::current_dir().unwrap();
    let original_base_url = env::var("BASE_URL").ok();
    let original_dotenv_path = env::var("DOTENV_PATH").ok();

    let tmp_dir = TempDir::new().expect("Failed to create temp dir");
    env::set_current_dir(tmp_dir.path()).unwrap();

    unsafe {
        env::remove_var("BASE_URL");
        env::set_var("DOTENV_PATH", "/nonexistent_dotenv_file");
    }

    let result = librcekunit::Config::from_env();

    env::set_current_dir(&original_dir).unwrap();
    if let Some(val) = original_base_url {
        unsafe {
            env::set_var("BASE_URL", val);
        }
    } else {
        unsafe {
            env::remove_var("BASE_URL");
        }
    }
    if let Some(val) = original_dotenv_path {
        unsafe {
            env::set_var("DOTENV_PATH", val);
        }
    } else {
        unsafe {
            env::remove_var("DOTENV_PATH");
        }
    }

    assert!(
        result.is_err(),
        "from_env harus error jika BASE_URL tidak ditemukan"
    );
    match result.unwrap_err() {
        librcekunit::Error::Config(msg) => {
            assert!(
                msg.contains("BASE_URL"),
                "Pesan error harus menyebut BASE_URL"
            );
        }
        other => panic!("Diharapkan Error::Config, ditemukan {:?}", other),
    }
}

#[test]
#[serial]
fn test_from_env_trim_trailing_slash() {
    let original = env::var("BASE_URL").ok();
    unsafe {
        env::set_var("BASE_URL", "http://localhost:3000/");
    }
    let cfg = Config::from_env().unwrap();
    match original {
        Some(val) => unsafe {
            env::set_var("BASE_URL", val);
        },
        None => unsafe {
            env::remove_var("BASE_URL");
        },
    }
    assert_eq!(cfg.base_url, "http://localhost:3000");
}

#[test]
fn test_cookie_store_debug() {
    let none = CookieStore::None;
    let memory = CookieStore::Memory;
    let persistent = CookieStore::Persistent(PathBuf::from("a.json"));
    assert!(format!("{:?}", none).contains("None"));
    assert!(format!("{:?}", memory).contains("Memory"));
    assert!(format!("{:?}", persistent).contains("Persistent"));
}

#[test]
fn test_cookie_store_clone() {
    let store = CookieStore::Persistent(PathBuf::from("cookies.txt"));
    let cloned = store.clone();
    match (store, cloned) {
        (CookieStore::Persistent(a), CookieStore::Persistent(b)) => assert_eq!(a, b),
        _ => panic!("Clone failed"),
    }
}
