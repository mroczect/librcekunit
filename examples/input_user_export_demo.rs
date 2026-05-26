//! Example: Export input user data as CSV (or other format).
//!
//! This example demonstrates how to:
//! - Initialize tracing and load environment variables.
//! - Create a client from configuration (fallback to localhost).
//! - Log in with email/password.
//! - Build query parameters for an export request (format, sorting, date filters).
//! - Call `client.input_user_export()` to retrieve the exported file.
//! - Save the response body to a local file.
//! - Handle success, HTTP errors, and request failures.
//! - Log out gracefully (ignoring logout errors).
//!
//! # Prerequisites
//!
//! - A running server at `BASE_URL` (default `http://localhost:8080`)
//! - Valid credentials (`DEMO_EMAIL`, `DEMO_PASSWORD`)
//! - The endpoint `/dashboard/input_user/export` must accept GET requests with query parameters.
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
//! cargo run --example export_demo
//! ```
//!
//! # Output
//!
//! On success, a file named `export_input_user.csv` is created in the current directory.
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
    let email = env::var("DEMO_EMAIL").unwrap_or_else(|_| "admin@example.com".into());
    let password = env::var("DEMO_PASSWORD").unwrap_or_else(|_| "rahasia123".into());

    // Build configuration: try Config::from_env(), else fallback to localhost.
    let config = Config::from_env()
        .unwrap_or_else(|_| {
            warn!("BASE_URL not set; falling back to http://localhost:8080");
            Config::new("http://localhost:8080")
        })
        .with_user_agent("librcekunit-export-demo/2.0")
        .with_timeout(30); // Longer timeout for potentially large exports.

    info!(base_url = %config.base_url, "Creating client");
    let client = Client::new(config).await?;

    // Step 1: Login.
    info!(%email, "Logging in");
    if let Err(e) = client.login(&email, &password).await {
        eprintln!("Login failed: {}", e);
        std::process::exit(1);
    }
    info!("Login successful");

    // Step 2: Configure export parameters.
    // These values can be changed or read from environment.
    let export_format = "csv"; // Format: csv, json, xlsx, etc.
    let sort_column = "created_at"; // Column to sort by.
    let sort_direction = "asc"; // asc or desc.
    let search_query = ""; // Optional search keyword.
    let start_date = "2026-05-26"; // Filter records from this date (inclusive).
    let end_date = "2026-05-26"; // Filter records to this date (inclusive).

    // Build query parameters HashMap.
    let mut params = HashMap::new();
    params.insert("format".into(), export_format.into());
    params.insert("sort".into(), sort_column.into());
    params.insert("direction".into(), sort_direction.into());

    // Only add optional parameters if they are non‑empty.
    if !search_query.is_empty() {
        params.insert("search".into(), search_query.into());
    }
    if !start_date.is_empty() {
        params.insert("start_date".into(), start_date.into());
    }
    if !end_date.is_empty() {
        params.insert("end_date".into(), end_date.into());
    }

    // Step 3: Send export request.
    info!("Requesting export with parameters: {:?}", params);
    match client.input_user_export(params).await {
        Ok(response) => {
            // Check HTTP status code.
            if response.status().is_success() {
                // Read the entire response body as bytes.
                let body = response.bytes().await?;

                // Define output filename (could be dynamic based on format/date).
                let filename = "export_input_user.csv";
                std::fs::write(filename, &body)?;

                info!(
                    "Export successful, saved {} bytes to `{}`",
                    body.len(),
                    filename
                );
                println!("File saved as `{}`", filename);
            } else {
                // Server returned an error status (4xx, 5xx).
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                eprintln!("Export returned HTTP {}: {}", status, body);
                std::process::exit(2);
            }
        }
        Err(e) => {
            // Request failed at network/CSRF/client level.
            eprintln!("Export request failed: {}", e);
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
