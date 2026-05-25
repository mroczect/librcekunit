use librcekunit::{Client, Config};
use std::collections::HashMap;
use std::env;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    librcekunit::init_tracing();

    dotenvy::dotenv().ok();

    let email = env::var("DEMO_EMAIL").unwrap_or_else(|_| "admin@example.com".into());
    let password = env::var("DEMO_PASSWORD").unwrap_or_else(|_| "rahasia123".into());

    let config = Config::from_env()
        .unwrap_or_else(|_| {
            warn!("BASE_URL not set; falling back to http://localhost:8080");
            Config::new("http://localhost:8080")
        })
        .with_user_agent("librcekunit-export-demo/2.0")
        .with_timeout(30);

    info!(base_url = %config.base_url, "Creating client");
    let client = Client::new(config).await?;

    info!(%email, "Logging in");
    if let Err(e) = client.login(&email, &password).await {
        eprintln!("Login failed: {}", e);
        std::process::exit(1);
    }
    info!("Login successful");

    let export_format = "csv";
    let sort_column = "created_at";
    let sort_direction = "asc";
    let search_query = "";
    let start_date = "2026-05-26";
    let end_date = "2026-05-26";

    let mut params = HashMap::new();
    params.insert("format".into(), export_format.into());
    params.insert("sort".into(), sort_column.into());
    params.insert("direction".into(), sort_direction.into());
    if !search_query.is_empty() {
        params.insert("search".into(), search_query.into());
    }
    if !start_date.is_empty() {
        params.insert("start_date".into(), start_date.into());
    }
    if !end_date.is_empty() {
        params.insert("end_date".into(), end_date.into());
    }

    info!("Requesting export with parameters: {:?}", params);
    match client.input_user_export(params).await {
        Ok(response) => {
            if response.status().is_success() {
                let body = response.bytes().await?;
                let filename = "export_input_user.csv";
                std::fs::write(filename, &body)?;
                info!(
                    "Export successful, saved {} bytes to `{}`",
                    body.len(),
                    filename
                );
                println!("File saved as `{}`", filename);
            } else {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                eprintln!("Export returned HTTP {}: {}", status, body);
                std::process::exit(2);
            }
        }
        Err(e) => {
            eprintln!("Export request failed: {}", e);
            std::process::exit(3);
        }
    }

    info!("Logging out");
    if let Err(e) = client.logout().await {
        warn!("Logout failed (ignored): {}", e);
    }

    Ok(())
}
