#![allow(unused_crate_dependencies)]

use core::error::Error;

use librcekunit_client::{ClientBuilder, CookieStore};
use librcekunit_handler::Config;
use std::path::PathBuf;

#[test]
fn builder_default_timeout_is_30() -> Result<(), Box<dyn Error>> {
    let config = ClientBuilder::new()
        .base_url("http://x.com")
        .build_config()?;
    assert_eq!(config.timeout.as_secs(), 30);
    Ok(())
}

#[test]
fn builder_default_user_agent_matches_handler() -> Result<(), Box<dyn Error>> {
    let config = ClientBuilder::new()
        .base_url("http://x.com")
        .build_config()?;
    assert_eq!(config.user_agent, "librcekunit/3.0");
    Ok(())
}

#[test]
fn builder_default_cookie_store_is_memory() -> Result<(), Box<dyn Error>> {
    let config = ClientBuilder::new()
        .base_url("http://x.com")
        .build_config()?;
    assert!(config.cookie_store.path().is_none());
    Ok(())
}

#[test]
fn builder_missing_base_url_errors() {
    let r = ClientBuilder::new().build_config();
    assert!(matches!(r, Err(librcekunit_handler::Error::Config(_))));
}

#[test]
fn builder_custom_timeout_applied() -> Result<(), Box<dyn Error>> {
    let config = ClientBuilder::new()
        .base_url("http://x.com")
        .timeout_secs(120)
        .build_config()?;
    assert_eq!(config.timeout.as_secs(), 120);
    Ok(())
}

#[test]
fn builder_custom_user_agent_applied() -> Result<(), Box<dyn Error>> {
    let config = ClientBuilder::new()
        .base_url("http://x.com")
        .user_agent("custom/9")
        .build_config()?;
    assert_eq!(config.user_agent, "custom/9");
    Ok(())
}

#[test]
fn builder_persistent_cookie_store_applied() -> Result<(), Box<dyn Error>> {
    let path = PathBuf::from("cookies.json");
    let config = ClientBuilder::new()
        .base_url("http://x.com")
        .cookie_store(CookieStore::Persistent(path.clone()))
        .build_config()?;
    assert_eq!(config.cookie_store.path(), Some(&path));
    Ok(())
}

#[test]
fn builder_none_cookie_store_applied() -> Result<(), Box<dyn Error>> {
    let config = ClientBuilder::new()
        .base_url("http://x.com")
        .cookie_store(CookieStore::None)
        .build_config()?;
    assert!(config.cookie_store.path().is_none());
    Ok(())
}

#[test]
fn builder_chain_all_options() -> Result<(), Box<dyn Error>> {
    let config = ClientBuilder::new()
        .base_url("http://x.com")
        .timeout_secs(60)
        .user_agent("chain/1")
        .cookie_store(CookieStore::Memory)
        .build_config()?;
    assert_eq!(config.base_url, "http://x.com");
    assert_eq!(config.timeout.as_secs(), 60);
    assert_eq!(config.user_agent, "chain/1");
    Ok(())
}

#[test]
fn builder_overwrite_base_url_uses_last() -> Result<(), Box<dyn Error>> {
    let config = ClientBuilder::new()
        .base_url("http://first.com")
        .base_url("http://second.com")
        .build_config()?;
    assert_eq!(config.base_url, "http://second.com");
    Ok(())
}

#[test]
fn builder_overwrite_timeout_uses_last() -> Result<(), Box<dyn Error>> {
    let config = ClientBuilder::new()
        .base_url("http://x.com")
        .timeout_secs(10)
        .timeout_secs(99)
        .build_config()?;
    assert_eq!(config.timeout.as_secs(), 99);
    Ok(())
}

#[test]
fn builder_is_debug() {
    let debug = format!("{:?}", ClientBuilder::new());
    assert!(debug.contains("ClientBuilder"));
}

#[test]
fn builder_is_clone() -> Result<(), Box<dyn Error>> {
    let a = ClientBuilder::new().base_url("http://x.com");
    let b = a.clone();
    let config_a = a.build_config()?;
    let config_b = b.build_config()?;
    assert_eq!(config_a.base_url, config_b.base_url);
    Ok(())
}

#[test]
fn builder_default_impl_matches_new() -> Result<(), Box<dyn Error>> {
    let a = ClientBuilder::default()
        .base_url("http://x.com")
        .build_config()?;
    let b = ClientBuilder::new()
        .base_url("http://x.com")
        .build_config()?;
    assert_eq!(a.base_url, b.base_url);
    Ok(())
}

#[tokio::test]
async fn builder_build_creates_client() -> Result<(), Box<dyn Error>> {
    let _client = ClientBuilder::new()
        .base_url("http://127.0.0.1:1")
        .build()
        .await?;
    Ok(())
}

#[tokio::test]
async fn builder_build_without_base_url_errors() -> Result<(), Box<dyn Error>> {
    let r = ClientBuilder::new().build().await;
    assert!(r.is_err());
    Ok(())
}

#[test]
fn config_reachable_via_reexport() {
    let _c: Config = Config::new("http://x.com");
}

#[test]
fn error_reachable_via_reexport() {
    let e: librcekunit_client::Error = librcekunit_handler::Error::NotLoggedIn;
    assert!(matches!(e, librcekunit_handler::Error::NotLoggedIn));
}
