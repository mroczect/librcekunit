#![allow(unused_crate_dependencies)]

extern crate alloc;

use alloc::sync::Arc;
use core::error::Error;

use librcekunit_api::cookies;
use reqwest::cookie::{CookieStore, Jar};
use url::Url;

fn tmp_path(tag: &str) -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("librcekunit_cookies_{tag}.json"));
    let _ = std::fs::remove_file(&p);
    p
}

#[tokio::test]
async fn save_with_no_cookies_and_no_file_is_ok() -> Result<(), Box<dyn Error>> {
    let jar = Arc::new(Jar::default());
    let path = tmp_path("save_none_nofile");
    cookies::save(&jar, "https://example.com", &path).await?;
    assert!(!path.exists());
    Ok(())
}

#[tokio::test]
async fn save_with_no_cookies_removes_existing_file() -> Result<(), Box<dyn Error>> {
    let jar = Arc::new(Jar::default());
    let path = tmp_path("save_none_hasfile");
    std::fs::write(&path, "[]")?;
    assert!(path.exists());
    cookies::save(&jar, "https://example.com", &path).await?;
    assert!(!path.exists());
    Ok(())
}

#[tokio::test]
async fn save_persists_cookie_to_file() -> Result<(), Box<dyn Error>> {
    let jar = Arc::new(Jar::default());
    let url = Url::parse("https://example.com")?;
    jar.add_cookie_str("session=abc; Path=/", &url);

    let path = tmp_path("save_persist");
    cookies::save(&jar, "https://example.com", &path).await?;
    let content = std::fs::read_to_string(&path)?;
    assert!(content.contains("session=abc"));
    let _ = std::fs::remove_file(&path);
    Ok(())
}

#[tokio::test]
async fn save_with_invalid_url_errors() -> Result<(), Box<dyn Error>> {
    let jar = Arc::new(Jar::default());
    let path = tmp_path("save_invalid_url");
    let r = cookies::save(&jar, "not-a-url", &path).await;
    assert!(r.is_err());
    Ok(())
}

#[tokio::test]
async fn load_missing_file_is_ok() -> Result<(), Box<dyn Error>> {
    let jar = Arc::new(Jar::default());
    let path = tmp_path("load_missing");
    cookies::load(&jar, "https://example.com", &path).await?;
    Ok(())
}

#[tokio::test]
async fn load_invalid_json_errors() -> Result<(), Box<dyn Error>> {
    let jar = Arc::new(Jar::default());
    let path = tmp_path("load_invalid_json");
    std::fs::write(&path, "not json at all")?;
    let r = cookies::load(&jar, "https://example.com", &path).await;
    assert!(r.is_err());
    let _ = std::fs::remove_file(&path);
    Ok(())
}

#[tokio::test]
async fn load_with_invalid_url_errors() -> Result<(), Box<dyn Error>> {
    let jar = Arc::new(Jar::default());
    let path = tmp_path("load_invalid_url");
    std::fs::write(&path, r#"["session=abc"]"#)?;
    let r = cookies::load(&jar, "not-a-url", &path).await;
    assert!(r.is_err());
    let _ = std::fs::remove_file(&path);
    Ok(())
}

#[tokio::test]
async fn save_then_load_roundtrip() -> Result<(), Box<dyn Error>> {
    let jar1 = Arc::new(Jar::default());
    let url = Url::parse("https://example.com")?;
    jar1.add_cookie_str("session=roundtrip; Path=/", &url);

    let path = tmp_path("roundtrip");
    cookies::save(&jar1, "https://example.com", &path).await?;

    let jar2 = Arc::new(Jar::default());
    cookies::load(&jar2, "https://example.com", &path).await?;

    let after = jar2.cookies(&url);
    assert!(after.is_some());

    let _ = std::fs::remove_file(&path);
    Ok(())
}

#[tokio::test]
async fn load_empty_json_array_is_ok() -> Result<(), Box<dyn Error>> {
    let jar = Arc::new(Jar::default());
    let path = tmp_path("load_empty");
    std::fs::write(&path, "[]")?;
    cookies::load(&jar, "https://example.com", &path).await?;
    let _ = std::fs::remove_file(&path);
    Ok(())
}
