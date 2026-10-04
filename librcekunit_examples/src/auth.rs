use librcekunit_client::prelude::{Auth, Client, Crud, Error, Result};

pub async fn login_and_verify(client: &Client, email: &str, password: &str) -> Result<()> {
    client.login(email, password).await?;

    let probe = client.index().await?;
    if probe.status().is_redirection() {
        return Err(Error::Auth("session not established".into()));
    }

    Ok(())
}

pub async fn safe_logout(client: &Client) -> Result<()> {
    match client.logout().await {
        Ok(()) => Ok(()),
        Err(Error::NotLoggedIn) => Ok(()),
        Err(other) => Err(other),
    }
}

#[must_use]
pub fn is_authenticated_status(status: u16) -> bool {
    (200..300).contains(&status)
}
