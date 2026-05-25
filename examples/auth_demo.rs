use librcekunit::{Client, Config, Error};
use std::env;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    librcekunit::init_tracing();
    dotenvy::dotenv().ok();

    let email = env::var("DEMO_EMAIL").unwrap_or_else(|_| "admin@example.com".to_string());
    let password = env::var("DEMO_PASSWORD").unwrap_or_else(|_| "rahasia123".to_string());

    let config = Config::from_env()
        .unwrap_or_else(|_| {
            warn!("BASE_URL not set; using http://localhost:8080");
            Config::new("http://localhost:8080")
        })
        .with_user_agent("librcekunit-demo/2.0")
        .with_timeout(10);

    info!(base_url = %config.base_url, "Creating client");
    let client = Client::new(config).await?;

    info!(%email, "Logging in");
    match client.login(&email, &password).await {
        Ok(()) => {
            info!("Login successful");

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

            info!("Logging out");
            match client.logout().await {
                Ok(()) => {
                    info!("Logout successful");
                    println!("Authentication flow completed successfully.");
                }
                Err(Error::Api(419, msg)) => {
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
