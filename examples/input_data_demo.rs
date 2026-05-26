//! Example: Submit input data to the server.
//!
//! This example demonstrates how to:
//! - Initialize tracing and load environment variables.
//! - Create a client from configuration (fallback to localhost).
//! - Log in with email/password.
//! - Build a `HashMap` of form data for an input data record.
//! - Submit the data via `client.input_data_store()`.
//! - Handle success, HTTP errors, and request failures.
//! - Log out gracefully (ignoring logout errors).
//!
//! # Prerequisites
//!
//! - A running server at `BASE_URL` (default `http://localhost:8080`)
//! - Valid credentials (`DEMO_EMAIL`, `DEMO_PASSWORD`)
//! - The endpoint `/dashboard/input-data` must accept POST requests with form data.
//!
//! # Usage
//!
//! Set environment variables (optional, but recommended):
//! ```bash
//! export BASE_URL="https://myapp.example.com"
//! export DEMO_EMAIL="user@example.com"
//! export DEMO_PASSWORD="secret"
//! ```
//!
//! Then run:
//! ```bash
//! cargo run --example input_data_demo
//! ```
//!
//! # Exit codes
//!
//! | Code | Meaning                          |
//! |------|----------------------------------|
//! | 0    | Success                          |
//! | 1    | Login failed                     |
//! | 2    | Server returned non‑success HTTP status |
//! | 3    | Request failed (network/CSRF/etc.) |

use librcekunit::{Client, Config};
use std::collections::HashMap;
use std::env;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize global tracing subscriber. Log level controlled by RUST_LOG.
    librcekunit::init_tracing();

    // Load .env file for local development (ignored if missing).
    dotenvy::dotenv().ok();

    // Read credentials from environment, with fallback defaults.
    // In production, always set these via environment variables.
    let email = env::var("DEMO_EMAIL").unwrap_or_else(|_| "admin@example.com".into());
    let password = env::var("DEMO_PASSWORD").unwrap_or_else(|_| "rahasia123".into());

    // Build configuration: try Config::from_env(), else fallback to localhost.
    let config = Config::from_env()
        .unwrap_or_else(|_| {
            warn!("BASE_URL not set; falling back to http://localhost:8080");
            Config::new("http://localhost:8080")
        })
        .with_user_agent("librcekunit-input-data-demo/2.0")
        .with_timeout(10);

    info!(base_url = %config.base_url, "Creating client");
    let client = Client::new(config).await?;

    // Step 1: Login.
    info!(%email, "Logging in");
    if let Err(e) = client.login(&email, &password).await {
        eprintln!("Login failed: {}", e);
        std::process::exit(1);
    }
    info!("Login successful");

    // Step 2: Prepare form data for the input data endpoint.
    // The keys must match the server's expected field names.
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

    // Step 3: Submit the data using the client's `input_data_store` method.
    info!("Submitting input data...");
    match client.input_data_store(data).await {
        Ok(response) => {
            let status = response.status();
            // Success or redirect (e.g., after POST, server may redirect).
            if status.is_redirection() || status.is_success() {
                info!("Input data submitted successfully (HTTP {})", status);
                println!("Data submitted successfully.");
            } else {
                // Server responded with an error status (4xx, 5xx).
                let body = response.text().await.unwrap_or_default();
                eprintln!("Server responded with HTTP {}: {}", status, body);
                std::process::exit(2);
            }
        }
        Err(e) => {
            // Request failed at network/CSRF/client level.
            eprintln!("Request failed: {}", e);
            std::process::exit(3);
        }
    }

    // Step 4: Logout (best effort, ignore failures).
    info!("Logging out");
    if let Err(e) = client.logout().await {
        warn!("Logout failed (ignored): {}", e);
    }

    Ok(())
}
