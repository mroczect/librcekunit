//! export DEMO_EMAIL="user@example.com"
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
            warn!("BASE_URL not set; using http://localhost:8080");
            Config::new("http://localhost:8080")
        })
        .with_user_agent("librcekunit-pic-demo/2.0")
        .with_timeout(10);

    info!(base_url = %config.base_url, "Creating client");
    let client = Client::new(config).await?;

    info!(%email, "Logging in");
    if let Err(e) = client.login(&email, &password).await {
        eprintln!("Login failed: {}", e);
        std::process::exit(1);
    }
    info!("Login successful");

    info!("Fetching PIC list");
    match client.pic_index().await {
        Ok(resp) if resp.status().is_success() => {
            let body = resp.text().await.unwrap_or_default();
            info!("PIC list retrieved ({} bytes)", body.len());
        }
        Ok(resp) => {
            warn!("PIC index returned status {}", resp.status());
        }
        Err(e) => {
            warn!("Failed to fetch PIC list: {}", e);
        }
    }

    let mut data = HashMap::new();
    data.insert("id_coll".into(), "DEMO01".into());
    data.insert("nama_collector".into(), "Demo Collector".into());
    data.insert("no_wa".into(), "08123456789".into());
    data.insert("status".into(), "Aktif".into());

    info!("Creating new PIC");
    match client.input_pic_store(data).await {
        Ok(resp) if resp.status().is_redirection() || resp.status().is_success() => {
            info!("PIC created successfully");
        }
        Ok(resp) => {
            warn!("Create PIC returned status {}", resp.status());
        }
        Err(e) => {
            warn!("Create PIC failed: {}", e);
        }
    }

    info!("Logging out");
    if let Err(e) = client.logout().await {
        warn!("Logout failed: {}", e);
    }

    Ok(())
}
