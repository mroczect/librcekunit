//! Example: Manage PIC (Person In Charge) resources.
//!
//! This example demonstrates how to:
//! - Initialize tracing and load environment variables.
//! - Create a client from configuration (fallback to localhost).
//! - Log in with email/password.
//! - Fetch the list of PICs via `client.pic_index()` and log the response length.
//! - Create a new PIC by submitting form data via `client.input_pic_store()`.
//! - Handle success and failure scenarios for each operation.
//! - Log out gracefully (ignoring logout errors).
//!
//! # Prerequisites
//!
//! - A running server at `BASE_URL` (default `http://localhost:8080`)
//! - Valid credentials (`DEMO_EMAIL`, `DEMO_PASSWORD`)
//! - The endpoint `/pic` must support GET requests (for listing)
//! - The endpoint `/dashboard/input-PIC` must support POST requests (for creation)
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
//! cargo run --example pic_demo
//! ```
//!
//! # Exit codes
//!
//! | Code | Meaning                          |
//! |------|----------------------------------|
//! | 0    | Success (all operations completed, even if some failed) |
//! | 1    | Login failed                     |
//! | 2    | Reserved for future use          |
//! | 3    | Reserved for future use          |

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
    let email = env::var("DEMO_EMAIL").unwrap_or_else(|_| "admin@example.com".into());
    let password = env::var("DEMO_PASSWORD").unwrap_or_else(|_| "rahasia123".into());

    // Build configuration: try Config::from_env(), else fallback to localhost.
    let config = Config::from_env()
        .unwrap_or_else(|_| {
            warn!("BASE_URL not set; using http://localhost:8080");
            Config::new("http://localhost:8080")
        })
        .with_user_agent("librcekunit-pic-demo/2.0")
        .with_timeout(10);

    info!(base_url = %config.base_url, "Creating client");
    let client = Client::new(config).await?;

    // Step 1: Login. Required for protected endpoints.
    info!(%email, "Logging in");
    if let Err(e) = client.login(&email, &password).await {
        eprintln!("Login failed: {}", e);
        std::process::exit(1);
    }
    info!("Login successful");

    // Step 2: Fetch the list of existing PICs.
    // This demonstrates a GET request to `/pic`.
    info!("Fetching PIC list");
    match client.pic_index().await {
        Ok(resp) if resp.status().is_success() => {
            let body = resp.text().await.unwrap_or_default();
            info!("PIC list retrieved ({} bytes)", body.len());
            // In a real application, you might parse the response (e.g., HTML or JSON).
        }
        Ok(resp) => {
            // Server returned a non‑success status (e.g., 403, 500).
            warn!("PIC index returned status {}", resp.status());
        }
        Err(e) => {
            // Network, CSRF, or other client error.
            warn!("Failed to fetch PIC list: {}", e);
        }
    }

    // Step 3: Prepare form data for creating a new PIC.
    // The keys must match the server's expected field names.
    let mut data = HashMap::new();
    data.insert("id_coll".into(), "DEMO01".into());
    data.insert("nama_collector".into(), "Demo Collector".into());
    data.insert("no_wa".into(), "08123456789".into());
    data.insert("status".into(), "Aktif".into());

    // Step 4: Create a new PIC using the input‑PIC endpoint.
    info!("Creating new PIC");
    match client.input_pic_store(data).await {
        Ok(resp) if resp.status().is_redirection() || resp.status().is_success() => {
            // Creation successful (server may redirect after POST).
            info!("PIC created successfully");
        }
        Ok(resp) => {
            // Server responded with a non‑success status (e.g., validation error).
            warn!("Create PIC returned status {}", resp.status());
        }
        Err(e) => {
            // Request failed at network/CSRF/client level.
            warn!("Create PIC failed: {}", e);
        }
    }

    // Step 5: Logout (best effort, ignore failures).
    info!("Logging out");
    if let Err(e) = client.logout().await {
        warn!("Logout failed: {}", e);
    }

    Ok(())
}
