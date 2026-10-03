//! client.login("user@example.com", "pass").await?;
//! login::login(&client, "user@example.com", "pass").await?;
pub mod login;
pub mod logout;

pub use login::*;
pub use logout::*;
