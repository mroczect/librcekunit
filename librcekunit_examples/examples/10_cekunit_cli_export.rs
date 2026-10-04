#![allow(unused_crate_dependencies)]

//! # cekunit — export CLI
//!
//! A small command-line tool for exporting Cek Unit data to CSV.
//! Runs as an example binary; invoke it with `cargo run`.
//!
//! ## Usage
//!
//! ```text
//! cargo run --example 10_cekunit_cli -- today
//! cargo run --example 10_cekunit_cli -- date 2026-09-01
//! cargo run --example 10_cekunit_cli -- range 2026-09-01 2026-09-30
//! cargo run --example 10_cekunit_cli -- all
//! ```
//!
//! Every command saves the result to a file in `~/Downloads/`.
//! Override the destination with `--out`. Refuse to overwrite an
//! existing file unless `--force` is passed.
//!
//! ## Configuration
//!
//! Credentials and the server URL are read from environment
//! variables. See `librcekunit_client::env` for the full list.
//!
//! ```sh
//! export LIBRCEKUNIT_BASE_URL=http://103.31.38.200
//! export LIBRCEKUNIT_EMAIL=user@example.com
//! export LIBRCEKUNIT_PASSWORD=secret
//! export LIBRCEKUNIT_COOKIE_FILE=session.json
//! ```

#![allow(clippy::wildcard_imports)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};

use librcekunit_examples::auth;
use librcekunit_examples::error;
use librcekunit_examples::prelude::*;

/// Boxed, thread-safe error type.
type DynError = Box<dyn core::error::Error + Send + Sync>;

/// Program exit codes.
const EXIT_OK: u8 = 0;
const EXIT_USAGE: u8 = 2;
const EXIT_FAILURE: u8 = 1;

/// Top-level CLI.
#[derive(Debug, Parser)]
#[command(
    name = "cekunit",
    version,
    about = "Export Cek Unit data to CSV",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// Subcommands.
#[derive(Debug, Subcommand)]
enum Command {
    /// Export today's records (local timezone).
    Today(ExportArgs),

    /// Export records for a single date.
    Date {
        /// Date in `YYYY-MM-DD` format.
        #[arg(value_name = "YYYY-MM-DD")]
        date: String,
        #[command(flatten)]
        export: ExportArgs,
    },

    /// Export records between two dates, inclusive.
    Range {
        /// Start date in `YYYY-MM-DD` format.
        #[arg(value_name = "START")]
        start: String,
        /// End date in `YYYY-MM-DD` format.
        #[arg(value_name = "END")]
        end: String,
        #[command(flatten)]
        export: ExportArgs,
    },

    /// Export every record. May take a while.
    All(ExportArgs),
}

/// Arguments shared by every export subcommand.
#[derive(Debug, Args)]
struct ExportArgs {
    /// Directory to write the CSV into. Defaults to `~/Downloads`.
    #[arg(long, value_name = "DIR")]
    out: Option<PathBuf>,

    /// Overwrite the file if it already exists.
    #[arg(long)]
    force: bool,

    /// Free-text search term applied by the server before export.
    #[arg(long, default_value = "")]
    search: String,

    /// Sort column. Common values: `no`, `created_at`, `nopol`.
    #[arg(long, default_value = "created_at")]
    sort: String,

    /// Sort direction: `asc` or `desc`.
    #[arg(long, default_value = "desc")]
    direction: String,

    /// Skip downloading. Print the target path and exit.
    #[arg(long)]
    dry_run: bool,
}

/// Resolved export specification after parsing.
#[derive(Debug, Clone)]
struct Spec {
    /// Start date, `YYYY-MM-DD`. `None` means no lower bound.
    start_date: Option<String>,
    /// End date, `YYYY-MM-DD`. `None` means no upper bound.
    end_date: Option<String>,
    /// Filename label, without extension.
    label: String,
}

/// Entry point.
fn main() -> ExitCode {
    let _ = librcekunit_client::tracing::init_tracing();

    let cli = match Cli::try_parse() {
        Ok(c) => c,
        Err(err) => {
            let _ = err.print();
            return ExitCode::from(EXIT_USAGE);
        }
    };

    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(err) => {
            eprintln!("[fatal] cannot start runtime: {err}");
            return ExitCode::from(EXIT_FAILURE);
        }
    };

    match runtime.block_on(run(cli)) {
        Ok(()) => ExitCode::from(EXIT_OK),
        Err(err) => {
            eprintln!("[error] {err}");
            ExitCode::from(EXIT_FAILURE)
        }
    }
}

/// Top-level async entry.
async fn run(cli: Cli) -> Result<(), DynError> {
    let (spec, args) = resolve(cli)?;

    let dir = match &args.out {
        Some(d) => d.clone(),
        None => default_downloads_dir()?,
    };
    let filename = format!("cekunit_{}.csv", spec.label);
    let target = dir.join(&filename);

    println!("[target] {}", target.display());

    if args.dry_run {
        println!("[dry-run] would write to the path above; exiting");
        return Ok(());
    }

    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| -> DynError { format!("create {}: {e}", dir.display()).into() })?;
        println!("[fs] created directory {}", dir.display());
    }

    if target.exists() && !args.force {
        return Err(format!(
            "file already exists: {}. pass --force to overwrite",
            target.display()
        )
        .into());
    }

    let client = connect().await?;

    println!("[export] requesting CSV");
    let params = build_params(&spec, &args);
    let resp = client.input_user_export(params).await?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        let preview_len = body.len().min(300);
        let preview = body.get(..preview_len).unwrap_or(&body);
        return Err(format!("server returned {status}; body preview: {preview}").into());
    }

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| -> DynError { format!("read body: {e}").into() })?;
    std::fs::write(&target, &bytes)
        .map_err(|e| -> DynError { format!("write {}: {e}", target.display()).into() })?;

    let text = String::from_utf8_lossy(&bytes);
    let first_line = text.lines().next().unwrap_or("");
    let line_count = text.lines().count();

    println!("[done] wrote {} bytes", bytes.len());
    println!("[done] file: {}", target.display());
    println!("[done] first line: {first_line}");
    println!("[done] lines: {line_count}");
    if !(first_line.contains(',') || first_line.to_lowercase().contains("no")) {
        println!("[warn] content does not look like CSV");
    }

    Ok(())
}

/// Build the query form for the export endpoint.
fn build_params(spec: &Spec, args: &ExportArgs) -> Form {
    let mut params = Form::new();
    let _ = params.insert(String::from("format"), String::from("csv"));
    let _ = params.insert(String::from("sort"), args.sort.clone());
    let _ = params.insert(String::from("direction"), args.direction.clone());
    let _ = params.insert(String::from("search"), args.search.clone());
    if let Some(s) = &spec.start_date {
        let _ = params.insert(String::from("start_date"), s.clone());
    }
    if let Some(e) = &spec.end_date {
        let _ = params.insert(String::from("end_date"), e.clone());
    }
    params
}

/// Turn the parsed CLI into a `Spec` and `ExportArgs`.
fn resolve(cli: Cli) -> Result<(Spec, ExportArgs), DynError> {
    match cli.command {
        Command::Today(args) => {
            let today = today_local()?;
            validate_date(&today)?;
            Ok((
                Spec {
                    start_date: Some(today.clone()),
                    end_date: Some(today.clone()),
                    label: format!("today_{today}"),
                },
                args,
            ))
        }
        Command::Date { date, export } => {
            validate_date(&date)?;
            Ok((
                Spec {
                    start_date: Some(date.clone()),
                    end_date: Some(date.clone()),
                    label: date,
                },
                export,
            ))
        }
        Command::Range { start, end, export } => {
            validate_date(&start)?;
            validate_date(&end)?;
            if start > end {
                return Err(format!("start {start} is after end {end}").into());
            }
            Ok((
                Spec {
                    start_date: Some(start.clone()),
                    end_date: Some(end.clone()),
                    label: format!("{start}_to_{end}"),
                },
                export,
            ))
        }
        Command::All(args) => {
            let stamp = now_stamp()?;
            Ok((
                Spec {
                    start_date: None,
                    end_date: None,
                    label: format!("all_{stamp}"),
                },
                args,
            ))
        }
    }
}

/// Today's local date as `YYYY-MM-DD`.
///
/// Uses the system `date` command, which respects the local timezone.
///
/// # Errors
///
/// Returns an error when the `date` command is missing or fails.
fn today_local() -> Result<String, DynError> {
    let output = std::process::Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .map_err(|e| -> DynError { format!("run date: {e}").into() })?;
    if !output.status.success() {
        return Err("date command failed".into());
    }
    let raw = String::from_utf8(output.stdout)
        .map_err(|e| -> DynError { format!("date output: {e}").into() })?;
    Ok(raw.trim().to_string())
}

/// Timestamp for filenames, `YYYYMMDD_HHMMSS`.
///
/// # Errors
///
/// Returns an error when the `date` command is missing or fails.
fn now_stamp() -> Result<String, DynError> {
    let output = std::process::Command::new("date")
        .arg("+%Y%m%d_%H%M%S")
        .output()
        .map_err(|e| -> DynError { format!("run date: {e}").into() })?;
    if !output.status.success() {
        return Err("date command failed".into());
    }
    let raw = String::from_utf8(output.stdout)
        .map_err(|e| -> DynError { format!("date output: {e}").into() })?;
    Ok(raw.trim().to_string())
}

/// Validate `YYYY-MM-DD`.
///
/// # Errors
///
/// Returns an error when the format is wrong.
fn validate_date(date: &str) -> Result<(), DynError> {
    let bytes = date.as_bytes();
    if bytes.len() != 10 {
        return Err(format!("date must be YYYY-MM-DD, got {date:?}").into());
    }
    if bytes.get(4) != Some(&b'-') || bytes.get(7) != Some(&b'-') {
        return Err(format!("date must be YYYY-MM-DD, got {date:?}").into());
    }
    for (i, b) in bytes.iter().enumerate() {
        if i == 4 || i == 7 {
            continue;
        }
        if !b.is_ascii_digit() {
            return Err(format!("date must be YYYY-MM-DD, got {date:?}").into());
        }
    }
    Ok(())
}

/// Default downloads directory: `$HOME/Downloads`.
///
/// # Errors
///
/// Returns an error when `$HOME` is not set.
fn default_downloads_dir() -> Result<PathBuf, DynError> {
    let home =
        std::env::var("HOME").map_err(|e| -> DynError { format!("$HOME not set: {e}").into() })?;
    Ok(Path::new(&home).join("Downloads"))
}

/// Build the client from environment and ensure a session.
///
/// # Errors
///
/// Returns an error when the client cannot be built or login fails.
async fn connect() -> Result<Client, DynError> {
    let client = match from_env().await {
        Ok(c) => c,
        Err(err) => {
            eprintln!("[config] {}", error::user_message(&err));
            return Err(err.into());
        }
    };

    let probe = client.dashboard_index().await?;
    if auth::is_authenticated_status(probe.status().as_u16()) {
        println!("[session] reused existing session");
        return Ok(client);
    }

    println!("[session] no valid session, logging in");
    let email = env_required("LIBRCEKUNIT_EMAIL")?;
    let password = env_required("LIBRCEKUNIT_PASSWORD")?;

    if let Err(err) = client.login(&email, &password).await {
        eprintln!("[login] {}", error::user_message(&err));
        return Err(err.into());
    }
    println!("[login] ok as {email}");
    Ok(client)
}

/// Read an environment variable or return a descriptive error.
///
/// # Errors
///
/// Returns a boxed error when the variable is missing or invalid.
fn env_required(key: &str) -> Result<String, DynError> {
    std::env::var(key).map_err(|e| format!("{key}: {e}").into())
}
