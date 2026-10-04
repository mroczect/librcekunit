//! Dispatcher subcommand.

pub mod auth;
pub mod csv;
pub mod dashboard;
pub mod info;
pub mod input_data;
pub mod input_user;
pub mod replace;
pub mod report;

use serde_json::Value;

use crate::cli::Command;
use crate::error::Result;

/// Dispatch dan hasilkan data JSON.
///
/// # Errors
///
/// Return error dari handler.
pub async fn dispatch(cmd: Command) -> Result<Value> {
    match cmd {
        Command::Info => info::run().await,
        Command::Auth { cmd } => auth::run(cmd).await,
        Command::Dashboard { cmd } => dashboard::run(cmd).await,
        Command::InputUser { cmd } => input_user::run(cmd).await,
        Command::InputData { cmd } => input_data::run(cmd).await,
        Command::Csv { cmd } => csv::run(cmd),
        Command::Report(args) => report::run(args).await,
        Command::Replace(args) => replace::run(args).await,
    }
}
