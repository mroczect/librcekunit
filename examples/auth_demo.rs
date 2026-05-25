//! # Authentication Demo (Robust)
//!
//! Demonstrates login and logout against a real server.
//! After login it performs a quick GET to `/dashboard/cekunit` to verify
//! that the session is truly authenticated.
//!
//! ## Prerequisites
//!
//! * `.env` file with `BASE_URL` set.
//! * Optional: `DEMO_EMAIL`, `DEMO_PASSWORD`.
//!
//! ## Running
//! ```bash
//! cargo run --example auth_demo
//! ```

use librcekunit::{Client, Config, Error};
use std::env;
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    librcekunit::init_tracing();
    dotenvy::dotenv().ok();

    let email = env::var("DEMO_EMAIL")
        .unwrap_or_else(|_| "admin@example.com".to_string());
    let password = env::var("DEMO_PASSWORD")
        .unwrap_or_else(|_| "rahasia123".to_string());

    let config = Config::from_env()
        .unwrap_or_else(|_| {
            warn!("BASE_URL not set; using http://localhost:8080");
            Config::new("http://localhost:8080")
        })
        .with_user_agent("librcekunit-demo/2.0")
        .with_timeout(10);

    info!(base_url = %config.base_url, "Creating client");
    let client = Client::new(config).await?;

    // ---- Login ----
    info!(%email, "Logging in");
    match client.login(&email, &password).await {
        Ok(()) => {
            info!("Login successful");

            // Verify session by fetching a protected page
            info!("Fetching /dashboard/cekunit to verify session");
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

            // ---- Logout ----
            info!("Logging out");
            match client.logout().await {
                Ok(()) => {
                    info!("Logout successful");
                    println!("Authentication flow completed successfully.");
                }
                Err(Error::Api(419, msg)) => {
                    // 419 typically indicates CSRF token mismatch.
                    // This can happen if the server's /logout endpoint expects
                    // a different token source or session expiry.
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
            error!("Login rejected: {}", msg);
            std::process::exit(1);
        }
        Err(e) => {
            error!("Login error: {}", e);
            std::process::exit(2);
        }
    }

    Ok(())
}