//! Example: Authentication flow demo for `librcekunit`.
//!
//! This example demonstrates how to use the library to:
//! - Initialize tracing
//! - Load configuration from environment (or fallback to defaults)
//! - Create a client
//! - Log in with email/password
//! - Verify the session by accessing a protected endpoint
//! - Log out gracefully
//! - Handle various error cases (invalid credentials, CSRF mismatch, etc.)
//!
//! # Prerequisites
//!
//! - A running server at `BASE_URL` (default `http://localhost:8080`)
//! - Valid credentials (`DEMO_EMAIL`, `DEMO_PASSWORD`) or the example defaults
//!
//! # Usage
//!
//! Set environment variables (optional):
//! ```bash
//! export BASE_URL="https://myapp.example.com"
//! export DEMO_EMAIL="user@example.com"
//! export DEMO_PASSWORD="secret"
//! ```
//!
//! Then run:
//! ```bash
//! cargo run --example demo
//! ```
//!
//! # Exit codes
//!
//! | Code | Meaning                          |
//! |------|----------------------------------|
//! | 0    | Success                          |
//! | 1    | Authentication failed (wrong password/user) |
//! | 2    | Login network or other error     |
//! | 3    | Logout CSRF mismatch (419)       |

use librcekunit::{Client, Config, Error};
use std::env;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize global tracing subscriber with `RUST_LOG` env filter.
    // Default log level is "info". Set RUST_LOG=debug for more details.
    librcekunit::init_tracing();

    // Load .env file if present (for local development).
    dotenvy::dotenv().ok();

    // Read email and password from environment, with fallback defaults.
    // In production, always provide credentials via environment variables.
    let email = env::var("DEMO_EMAIL").unwrap_or_else(|_| "admin@example.com".to_string());
    let password = env::var("DEMO_PASSWORD").unwrap_or_else(|_| "rahasia123".to_string());

    // Create configuration: try from environment first, otherwise use fallback.
    let config = Config::from_env()
        .unwrap_or_else(|_| {
            warn!("BASE_URL not set; using http://localhost:8080");
            Config::new("http://localhost:8080")
        })
        // Customize user agent and timeout for this demo.
        .with_user_agent("librcekunit-demo/2.0")
        .with_timeout(10);

    info!(base_url = %config.base_url, "Creating client");
    let client = Client::new(config).await?;

    // Attempt login.
    info!(%email, "Logging in");
    match client.login(&email, &password).await {
        Ok(()) => {
            info!("Login successful");

            // Verify that the session actually works by fetching a protected endpoint.
            info!("Fetching /dashboard to verify session");
            match client.cekunit_index().await {
                Ok(resp) if resp.status().is_success() => {
                    info!("Session verified (status {})", resp.status());
                }
                Ok(resp) => {
                    warn!(
                        "Unexpected status {} when accessing dashboard; session may be incomplete",
                        resp.status()
                    );
                }
                Err(e) => {
                    warn!("Failed to fetch dashboard: {}; session may be invalid", e);
                }
            }

            // Log out and clean up session.
            info!("Logging out");
            match client.logout().await {
                Ok(()) => {
                    info!("Logout successful");
                    println!("Authentication flow completed successfully.");
                }
                Err(Error::Api(419, msg)) => {
                    // HTTP 419 indicates CSRF token mismatch – common if server expects
                    // a different token handling or session expired.
                    error!(
                        "Logout failed with 419 (CSRF mismatch): {}. \
                         Ensure that the library's CSRF handling matches the server's expectations.",
                        msg
                    );
                    std::process::exit(3);
                }
                Err(e) => {
                    warn!("Logout failed: {}", e);
                    println!("Logout failed, but session may already be invalid.");
                }
            }
        }
        Err(Error::Auth(msg)) => {
            // Authentication error: wrong credentials.
            error!("Login rejected: {}", msg);
            std::process::exit(1);
        }
        Err(e) => {
            // Other errors (network, CSRF not found, etc.).
            error!("Login error: {}", e);
            std::process::exit(2);
        }
    }

    Ok(())
}
