use std::path::Path;

use librcekunit_handler::{Error, Result};
use reqwest::cookie::{CookieStore, Jar};
use url::Url;

pub async fn save(jar: &Jar, base_url: &str, path: &Path) -> Result<()> {
    let Ok(parsed) = Url::parse(base_url) else {
        return Err(Error::Config(String::from("invalid base_url")));
    };

    let cookies: Vec<String> = CookieStore::cookies(jar, &parsed).map_or_else(Vec::new, |value| {
        value
            .to_str()
            .map(|s| s.split("; ").map(String::from).collect())
            .unwrap_or_default()
    });

    if cookies.is_empty() {
        if path.exists() {
            tokio::fs::remove_file(path)
                .await
                .map_err(|e| Error::Io(e.to_string()))?;
        }
        return Ok(());
    }

    let json = serde_json::to_string(&cookies).map_err(|e| Error::Json(e.to_string()))?;
    tokio::fs::write(path, json)
        .await
        .map_err(|e| Error::Io(e.to_string()))?;
    Ok(())
}

pub async fn load(jar: &Jar, base_url: &str, path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let data = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| Error::Io(e.to_string()))?;
    let cookies: Vec<String> =
        serde_json::from_str(&data).map_err(|e| Error::Json(e.to_string()))?;
    let Ok(parsed) = Url::parse(base_url) else {
        return Err(Error::Config(String::from("invalid base_url")));
    };
    for cookie in &cookies {
        jar.add_cookie_str(cookie, &parsed);
    }
    Ok(())
}
