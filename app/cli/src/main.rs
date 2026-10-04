#![allow(unused_crate_dependencies)]
#![allow(clippy::std_instead_of_alloc)]
#![allow(clippy::std_instead_of_core)]
#![allow(unreachable_pub)]
#![allow(unnameable_types)]

//! # cekunit — machine-oriented CLI

mod cli;
mod client;
mod commands;
mod csv_ops;
mod error;
mod logging;
mod output;
mod safety;

use clap::Parser;

use crate::cli::Cli;
use crate::error::ExitCode;

#[tokio::main]
async fn main() -> std::process::ExitCode {
    logging::init();

    let cli = match Cli::try_parse() {
        Ok(c) => c,
        Err(err) => {
            use clap::error::ErrorKind;
            #[allow(clippy::wildcard_enum_match_arm)]
            match err.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                    let _ = err.print();
                    return ExitCode::Ok.into();
                }
                _ => {
                    let msg = err.to_string();
                    output::print_error("cli", &error::Error::Usage(msg));
                    return ExitCode::Usage.into();
                }
            }
        }
    };

    let command_name = cli.command.name();
    let result = commands::dispatch(cli.command).await;

    match result {
        Ok(data) => {
            output::print_ok(command_name, &data);
            ExitCode::Ok.into()
        }
        Err(err) => {
            output::print_error(command_name, &err);
            err.exit_code().into()
        }
    }
}
