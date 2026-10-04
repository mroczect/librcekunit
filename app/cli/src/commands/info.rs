//! `cekunit info`.

use librcekunit_client::prelude::Dashboard;
use serde_json::{Value, json};

use crate::error::Result;

/// Jalankan `info`.
///
/// # Errors
///
/// Return error kalau request gagal.
pub async fn run() -> Result<Value> {
    let base_url = std::env::var("LIBRCEKUNIT_BASE_URL").unwrap_or_default();
    let client = librcekunit_client::env::from_env().await?;
    let probe = client.dashboard_index().await?;
    let status = probe.status();
    let authed = status.is_success();

    Ok(json!({
        "base_url": base_url,
        "session": if authed { "active" } else { "inactive" },
        "probe": {
            "path": "/dashboard",
            "status": status.as_u16(),
        }
    }))
}
