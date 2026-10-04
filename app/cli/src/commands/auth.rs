//! `cekunit auth`.

use librcekunit_client::prelude::{Auth, Dashboard};
use serde_json::{Value, json};

use crate::cli::AuthCmd;
use crate::error::Result;

/// Dispatch `auth`.
///
/// # Errors
///
/// Return error kalau request gagal.
pub async fn run(cmd: AuthCmd) -> Result<Value> {
    match cmd {
        AuthCmd::Login => login().await,
        AuthCmd::Logout => logout().await,
        AuthCmd::Info => super::info::run().await,
    }
}

async fn login() -> Result<Value> {
    let client = crate::client::connect_fresh().await?;
    let probe = client.dashboard_index().await?;
    Ok(json!({
        "logged_in": probe.status().is_success(),
        "probe_status": probe.status().as_u16(),
    }))
}

async fn logout() -> Result<Value> {
    let client = librcekunit_client::env::from_env().await?;
    client.logout().await?;
    Ok(json!({ "logged_out": true }))
}
