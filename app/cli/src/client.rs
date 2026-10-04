//! Helper koneksi ke server.

use librcekunit_client::prelude::{Auth, Client, Dashboard};

use crate::error::{Error, Result};

/// Bangun client dari env, login kalau perlu.
///
/// # Errors
///
/// Return error kalau client tidak bisa dibangun atau login gagal.
pub async fn connect() -> Result<Client> {
    let client = librcekunit_client::env::from_env().await?;

    let probe = client.dashboard_index().await?;
    if probe.status().is_success() {
        return Ok(client);
    }

    let email = env_required("LIBRCEKUNIT_EMAIL")?;
    let password = env_required("LIBRCEKUNIT_PASSWORD")?;
    client.login(&email, &password).await?;

    let after = client.dashboard_index().await?;
    if !after.status().is_success() {
        return Err(Error::Auth(String::from(
            "login sukses tapi probe tetap gagal",
        )));
    }
    Ok(client)
}

/// Bangun client dan paksa login.
///
/// # Errors
///
/// Return error kalau client tidak bisa dibangun atau login gagal.
pub async fn connect_fresh() -> Result<Client> {
    let client = librcekunit_client::env::from_env().await?;
    let email = env_required("LIBRCEKUNIT_EMAIL")?;
    let password = env_required("LIBRCEKUNIT_PASSWORD")?;
    client.login(&email, &password).await?;
    Ok(client)
}

fn env_required(key: &str) -> Result<String> {
    std::env::var(key).map_err(|_err| Error::Config(format!("{key} tidak di-set")))
}
