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
        .with_user_agent("librcekunit-input-data-demo/2.0")
        .with_timeout(10);

    info!(base_url = %config.base_url, "Creating client");
    let client = Client::new(config).await?;

    info!(%email, "Logging in");
    if let Err(e) = client.login(&email, &password).await {
        eprintln!("Login failed: {}", e);
        std::process::exit(1);
    }
    info!("Login successful");

    let mut data = HashMap::new();
    data.insert("no_perjanjian".into(), "OD-26-F0000999".into());
    data.insert("nama_nasabah".into(), "Budi Anduk".into());
    data.insert("nopol".into(), "BP 9999 ZZ".into());
    data.insert("coll".into(), "C".into());
    data.insert("pic".into(), "BELUM JTO".into());
    data.insert("kategori".into(), "A".into());
    data.insert("jto".into(), "12/04/2026".into());
    data.insert("no_rangka".into(), "MH1XXX000000".into());
    data.insert("no_mesin".into(), "ENG123456".into());
    data.insert("merk".into(), "HONDA".into());
    data.insert("type".into(), "BEAT".into());
    data.insert("warna".into(), "HITAM".into());
    data.insert("status".into(), "JG".into());

    info!("Submitting input data...");
    match client.input_data_store(data).await {
        Ok(response) => {
            let status = response.status();
            if status.is_redirection() || status.is_success() {
                info!("Input data submitted successfully (HTTP {})", status);
                println!("Data submitted successfully.");
            } else {
                let body = response.text().await.unwrap_or_default();
                eprintln!("Server responded with HTTP {}: {}", status, body);
                std::process::exit(2);
            }
        }
        Err(e) => {
            eprintln!("Request failed: {}", e);
            std::process::exit(3);
        }
    }

    info!("Logging out");
    if let Err(e) = client.logout().await {
        warn!("Logout failed (ignored): {}", e);
    }

    Ok(())
}
