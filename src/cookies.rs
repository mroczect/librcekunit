use crate::error::Error;
use reqwest::cookie::{CookieStore, Jar};
use std::fs;
use std::path::Path;
use tracing::{debug, instrument};

#[instrument(skip(jar))]
pub fn save_cookies_to_file(jar: &Jar, url: &str, path: &Path) -> Result<(), Error> {
    let url_parsed = url.parse().expect("Invalid base URL for cookie saving");
    let cookies: Vec<String> = jar
        .cookies(&url_parsed)
        .into_iter()
        .map(|h| h.to_str().unwrap_or_default().to_string())
        .collect();

    if cookies.is_empty() {
        debug!("No cookies to save, removing file if exists");
        if path.exists() {
            fs::remove_file(path)?;
        }
        return Ok(());
    }

    let json = serde_json::to_string(&cookies)?;
    fs::write(path, &json)?;
    debug!(?path, "Saved {} cookies to file", cookies.len());
    Ok(())
}

#[instrument(skip(jar))]
pub fn load_cookies_from_file(jar: &Jar, url: &str, path: &Path) -> Result<(), Error> {
    if !path.exists() {
        debug!("Cookie file not found, skipping load");
        return Ok(());
    }

    let data = fs::read_to_string(path)?;
    let cookies: Vec<String> = serde_json::from_str(&data)?;
    let url_parsed = url.parse().expect("Invalid base URL for cookie loading");

    for cookie_str in &cookies {
        jar.add_cookie_str(cookie_str, &url_parsed);
    }

    debug!(?path, "Loaded {} cookies from file", cookies.len());
    Ok(())
}
