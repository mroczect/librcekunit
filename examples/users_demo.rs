use librcekunit::{Client, Config};
use std::collections::HashMap;
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
            warn!("BASE_URL not set; falling back to http://localhost:8080");
            Config::new("http://localhost:8080")
        })
        .with_user_agent("librcekunit-users-demo/2.0")
        .with_timeout(10);

    info!(base_url = %config.base_url, "Creating client");
    let client = Client::new(config).await?;

    info!(%email, "Logging in");
    if let Err(e) = client.login(&email, &password).await {
        error!("Login failed: {}", e);
        std::process::exit(1);
    }
    info!("Login successful");

    info!("Fetching dashboard users page");
    match client.dashboard_users_index().await {
        Ok(resp) if resp.status().is_success() => {
            let body = resp.text().await.unwrap_or_default();
            info!("Dashboard users page retrieved ({} bytes)", body.len());
        }
        Ok(resp) => {
            warn!(
                "Dashboard users returned unexpected status: {}",
                resp.status()
            );
        }
        Err(e) => {
            error!("Failed to fetch dashboard users: {}", e);
        }
    }

    info!("Fetching public users list");
    match client.users_index().await {
        Ok(resp) if resp.status().is_success() => {
            let body = resp.text().await.unwrap_or_default();
            info!("Public users list retrieved ({} bytes)", body.len());
        }
        Ok(resp) => {
            warn!("Public users returned status: {}", resp.status());
        }
        Err(e) => {
            error!("Failed to fetch public users: {}", e);
        }
    }

    let user_id: u64 = 1;
    info!("Fetching detail for user ID {}", user_id);
    match client.users_show(user_id).await {
        Ok(resp) if resp.status().is_success() => {
            let body = resp.text().await.unwrap_or_default();
            info!("User detail for ID {}: {}", user_id, body);
        }
        Ok(resp) => {
            warn!("Show user returned status: {}", resp.status());
        }
        Err(e) => {
            error!("Failed to show user: {}", e);
        }
    }

    let update_id: u64 = 1;
    info!("Updating user ID {}", update_id);
    let mut update_data = HashMap::new();
    update_data.insert("status".into(), "Karyawan".into());
    update_data.insert("nama".into(), "Nama Demo Update".into());
    update_data.insert("no_wa".into(), "08123456789".into());
    update_data.insert("email".into(), "demo@example.com".into());

    match client.users_update(update_id, update_data).await {
        Ok(resp) if resp.status().is_redirection() || resp.status().is_success() => {
            info!("User ID {} updated successfully", update_id);
        }
        Ok(resp) => {
            let body = resp.text().await.unwrap_or_default();
            warn!("Update user returned status {}: {}", resp.status(), body);
        }
        Err(e) => {
            error!("Update user failed: {}", e);
        }
    }

    info!("Logging out");
    if let Err(e) = client.logout().await {
        warn!("Logout failed (session may already be expired): {}", e);
    } else {
        info!("Logout successful");
    }

    println!("Users demo completed.");
    Ok(())
}
