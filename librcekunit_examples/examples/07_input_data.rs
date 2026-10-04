#![allow(unused_crate_dependencies)]

//! # Insert data: single record or CSV upload
//!
//! This example covers both ways to add data to the Cek Unit dashboard,
//! mirroring the "Insert Data" page on the admin UI:
//!
//! 1. Single record via form fields (`POST /dashboard/input-data`).
//! 2. Bulk upload via CSV multipart (`POST /dashboard/input_user/insert`).
//!
//! Both are write operations. This example applies the same safety
//! model used by examples 03 and 06.
//!
//! ## Safety model
//!
//! 1. **Read-only by default.** Without `--execute`, the example
//!    parses its arguments, validates the payload, prints a preview,
//!    and exits without contacting the server for a write.
//!
//! 2. **Interactive terminal required.** `--execute` refuses to run
//!    when stdin is not a TTY. Piping is rejected before any request
//!    is sent.
//!
//! 3. **Typed confirmation.** After passing the terminal check, the
//!    example prompts for `INSERT-1` (single) or `INSERT-<rows>`
//!    (CSV). Typing anything else aborts.
//!
//! 4. **CSV file inspected locally.** Row count and header are read
//!    from the file before upload so the summary is accurate.
//!
//! ## Environment variables
//!
//! | Name                      | Required | Purpose                                       |
//! |---------------------------|----------|-----------------------------------------------|
//! | `LIBRCEKUNIT_BASE_URL`    | yes      | Root URL of the Laravel backend.              |
//! | `LIBRCEKUNIT_COOKIE_FILE` | no       | When set, session is persisted between runs.  |
//! | `LIBRCEKUNIT_EMAIL`       | if no session | Login email address.                     |
//! | `LIBRCEKUNIT_PASSWORD`    | if no session | Login password.                          |
//! | `RUST_LOG`                | no       | Tracing filter. Defaults to `info`.           |
//!
//! ## Usage — single record
//!
//! Preview without sending:
//!
//! ```sh
//! loadenv
//! cargo run --example 07_input_data -- single \
//!     --no_perjanjian SP-25-F0001843 \
//!     --nama_nasabah SADIMAN \
//!     --nopol BP2927GU \
//!     --kategori A
//! ```
//!
//! Execute (interactive confirmation required):
//!
//! ```sh
//! cargo run --example 07_input_data -- single --execute \
//!     --no_perjanjian SP-25-F0001843 \
//!     --nama_nasabah SADIMAN \
//!     --nopol BP2927GU
//! ```
//!
//! ## Usage — CSV upload
//!
//! Preview (no upload):
//!
//! ```sh
//! cargo run --example 07_input_data -- csv --file input.csv
//! ```
//!
//! Execute:
//!
//! ```sh
//! cargo run --example 07_input_data -- csv --file input.csv --execute
//! ```

#![allow(clippy::wildcard_imports)]

use std::io::IsTerminal as _;
use std::io::Write as _;
use std::path::PathBuf;

use librcekunit_examples::auth;
use librcekunit_examples::error;
use librcekunit_examples::form::FormBuilder;
use librcekunit_examples::prelude::*;

/// Boxed, thread-safe error type used as the return of `main`.
type DynError = Box<dyn core::error::Error + Send + Sync>;

/// Which sub-command to run.
#[derive(Debug, Clone)]
enum Mode {
    /// Insert one record from form fields.
    Single {
        /// Field values collected from CLI flags.
        fields: Vec<(String, String)>,
        /// When true, sends the request after confirmation.
        execute: bool,
    },
    /// Upload a CSV file as multipart.
    Csv {
        /// Path to the CSV file.
        file: PathBuf,
        /// When true, sends the request after confirmation.
        execute: bool,
    },
}

/// Known field names accepted by `single`. Used for validation.
const KNOWN_FIELDS: &[&str] = &[
    "no_perjanjian",
    "nama_nasabah",
    "nopol",
    "coll",
    "pic",
    "kategori",
    "jto",
    "no_rangka",
    "no_mesin",
    "merk",
    "type",
    "warna",
    "status",
    "actual_penyelesaian",
    "angsuran_ke",
    "tenor",
];

/// Entry point.
#[tokio::main]
async fn main() -> Result<(), DynError> {
    let _ = librcekunit_client::tracing::init_tracing();

    let mode = parse_args()?;
    print_banner(&mode);

    // Hard safety gate: execute mode requires a real terminal.
    if mode_is_execute(&mode) && !std::io::stdin().is_terminal() {
        eprintln!("[safety] refusing --execute: stdin is not a terminal");
        eprintln!("[safety] run without --execute to preview the payload");
        return Err("--execute requires an interactive terminal".into());
    }

    // Local validation before any network activity.
    match &mode {
        Mode::Single { fields, .. } => validate_single(fields)?,
        Mode::Csv { file, .. } => validate_csv(file)?,
    }

    tracing::info!("building client from environment");
    let client = match from_env().await {
        Ok(c) => c,
        Err(err) => {
            eprintln!("[config] {}", error::user_message(&err));
            return Err(err.into());
        }
    };
    ensure_session(&client).await?;

    // Dispatch.
    match mode {
        Mode::Single { fields, execute } => {
            run_single(&client, &fields, execute).await?;
        }
        Mode::Csv { file, execute } => {
            run_csv(&client, &file, execute).await?;
        }
    }

    println!("done");
    Ok(())
}

/// Whether the selected mode is an execute run.
const fn mode_is_execute(mode: &Mode) -> bool {
    match mode {
        Mode::Single { execute, .. } | Mode::Csv { execute, .. } => *execute,
    }
}

/// Parse command-line arguments.
///
/// # Errors
///
/// Returns an error when the sub-command is missing, unknown, or when
/// a `single` mode is invoked with no fields.
fn parse_args() -> Result<Mode, DynError> {
    let args: Vec<String> = std::env::args().collect();
    let mut iter = args.iter().skip(1);

    let subcommand = if let Some(s) = iter.next() {
        s.as_str()
    } else {
        eprintln!(
            "[usage] cargo run --example 07_input_data -- single [--execute] --<field> <value> ..."
        );
        eprintln!("[usage] cargo run --example 07_input_data -- csv --file <path> [--execute]");
        return Err("missing subcommand: expected `single` or `csv`".into());
    };

    let mut execute = false;
    let mut file: Option<PathBuf> = None;
    let mut fields: Vec<(String, String)> = Vec::new();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--execute" => execute = true,
            "--file" => {
                if let Some(v) = iter.next() {
                    file = Some(PathBuf::from(v));
                }
            }
            other if other.starts_with("--") => {
                let key = other.trim_start_matches("--").to_owned();
                if let Some(v) = iter.next() {
                    fields.push((key, v.clone()));
                }
            }
            _ => {}
        }
    }

    match subcommand {
        "single" => {
            if fields.is_empty() {
                return Err("single mode requires at least one field flag".into());
            }
            Ok(Mode::Single { fields, execute })
        }
        "csv" => {
            let file = file.ok_or("csv mode requires --file <path>")?;
            Ok(Mode::Csv { file, execute })
        }
        other => Err(format!("unknown subcommand: {other}").into()),
    }
}

/// Validate single-record fields locally.
///
/// # Errors
///
/// Returns an error when an unknown field is passed or when
/// `no_perjanjian` is missing or empty.
fn validate_single(fields: &[(String, String)]) -> Result<(), DynError> {
    for (key, _) in fields {
        if !KNOWN_FIELDS.contains(&key.as_str()) {
            eprintln!("[validate] unknown field: {key}");
            eprintln!("[validate] known fields: {}", KNOWN_FIELDS.join(", "));
            return Err(format!("unknown field: {key}").into());
        }
    }

    let no_perjanjian = fields
        .iter()
        .find(|(k, _)| k == "no_perjanjian")
        .map_or("", |(_, v)| v.as_str());

    if no_perjanjian.trim().is_empty() {
        eprintln!("[validate] no_perjanjian is required for single insert");
        return Err("no_perjanjian is required".into());
    }

    Ok(())
}

/// Validate the CSV file locally.
///
/// # Errors
///
/// Returns an error when the file does not exist, is not readable, or
/// appears to be empty.
fn validate_csv(file: &std::path::Path) -> Result<(), DynError> {
    if !file.exists() {
        return Err(format!("file not found: {}", file.display()).into());
    }
    let meta = std::fs::metadata(file)
        .map_err(|e| -> DynError { format!("cannot stat {}: {e}", file.display()).into() })?;
    if meta.len() == 0 {
        return Err(format!("file is empty: {}", file.display()).into());
    }
    Ok(())
}

/// Print a banner describing the mode.
fn print_banner(mode: &Mode) {
    println!("============================================================");
    match mode {
        Mode::Single { fields, execute } => {
            println!("INPUT DATA: single record");
            println!("mode:    {}", if *execute { "EXECUTE" } else { "PREVIEW" });
            println!("fields:  {}", fields.len());
        }
        Mode::Csv { file, execute } => {
            println!("INPUT DATA: CSV upload");
            println!("mode:    {}", if *execute { "EXECUTE" } else { "PREVIEW" });
            println!("file:    {}", file.display());
        }
    }
    println!("============================================================");
}

/// Insert one record.
///
/// # Errors
///
/// Returns an error when the insert request fails.
async fn run_single(
    client: &Client,
    fields: &[(String, String)],
    execute: bool,
) -> Result<(), DynError> {
    let mut builder = FormBuilder::new();
    for (key, value) in fields {
        builder = builder.set(key, value);
    }
    let form = builder.build();

    println!("[preview] payload ({} field(s)):", form.len());
    let mut keys: Vec<&String> = form.keys().collect();
    keys.sort();
    for key in keys {
        if let Some(value) = form.get(key) {
            println!("[preview]   {key} = {value}");
        }
    }

    if !execute {
        println!("[action] preview only; no write request was sent");
        println!("[action] to actually insert, re-run with --execute");
        return Ok(());
    }

    let expected = String::from("INSERT-1");
    print!("Type {expected:?} to insert this record: ");
    let _ = std::io::stdout().flush();

    let typed = read_line()?;
    if typed.trim() != expected {
        println!("[abort] confirmation did not match; nothing was inserted");
        return Ok(());
    }

    println!("[insert] confirmation accepted, sending request");
    match client.input_data_store(form).await {
        Ok(resp) => {
            println!("[insert] status={}", resp.status());
            let body = resp.text().await.unwrap_or_default();
            let preview_len = body.len().min(200);
            let preview = body.get(..preview_len).unwrap_or(&body);
            println!("[insert] response preview: {preview}");
        }
        Err(err) => {
            eprintln!("[insert] {}", error::user_message(&err));
            return Err(err.into());
        }
    }

    Ok(())
}

/// Upload a CSV file as multipart.
///
/// # Errors
///
/// Returns an error when the file cannot be read or the upload fails.
async fn run_csv(client: &Client, file: &std::path::Path, execute: bool) -> Result<(), DynError> {
    let contents = std::fs::read_to_string(file)
        .map_err(|e| -> DynError { format!("read {}: {e}", file.display()).into() })?;
    let lines: Vec<&str> = contents.lines().filter(|l| !l.trim().is_empty()).collect();

    let header = lines.first().copied().unwrap_or("");
    let data_rows = lines.len().saturating_sub(1);

    println!("[preview] file: {}", file.display());
    println!("[preview] header: {header}");
    println!("[preview] data rows: {data_rows}");
    println!("[preview] first 3 data rows:");
    for (i, line) in lines.iter().skip(1).take(3).enumerate() {
        println!("[preview]   {}: {}", i.saturating_add(1), line);
    }

    if !execute {
        println!("[action] preview only; no upload was sent");
        println!("[action] to actually upload, re-run with --execute");
        return Ok(());
    }

    let expected = format!("INSERT-{data_rows}");
    print!("Type {expected:?} to upload {data_rows} row(s): ");
    let _ = std::io::stdout().flush();

    let typed = read_line()?;
    if typed.trim() != expected {
        println!("[abort] confirmation did not match; nothing was uploaded");
        return Ok(());
    }

    println!("[upload] confirmation accepted, sending multipart request");
    match client.import_csv(file).await {
        Ok(resp) => {
            println!("[upload] status={}", resp.status());
            let body = resp.text().await.unwrap_or_default();
            let preview_len = body.len().min(200);
            let preview = body.get(..preview_len).unwrap_or(&body);
            println!("[upload] response preview: {preview}");
        }
        Err(err) => {
            eprintln!("[upload] {}", error::user_message(&err));
            return Err(err.into());
        }
    }

    Ok(())
}

/// Read one line from stdin.
///
/// # Errors
///
/// Returns a boxed error when reading fails.
fn read_line() -> Result<String, DynError> {
    let mut buffer = String::new();
    let _: usize = std::io::stdin()
        .read_line(&mut buffer)
        .map_err(|e| -> DynError { format!("failed to read stdin: {e}").into() })?;
    Ok(buffer)
}

/// Ensure the client has an authenticated session.
///
/// # Errors
///
/// Returns an error when the probe or login step fails, or when the
/// post-login probe still reports no session.
async fn ensure_session(client: &Client) -> Result<(), DynError> {
    tracing::info!("probing session via /dashboard");
    let probe = match client.dashboard_index().await {
        Ok(r) => r,
        Err(err) => {
            eprintln!("[probe] {}", error::user_message(&err));
            return Err(err.into());
        }
    };

    if auth::is_authenticated_status(probe.status().as_u16()) {
        println!("[session] reused existing session");
        return Ok(());
    }

    println!("[session] no valid session, logging in");
    let email = env_required("LIBRCEKUNIT_EMAIL")?;
    let password = env_required("LIBRCEKUNIT_PASSWORD")?;

    if let Err(err) = client.login(&email, &password).await {
        eprintln!("[login] {}", error::user_message(&err));
        return Err(err.into());
    }
    println!("[login] ok as {email}");

    let after = client.dashboard_index().await?;
    if !auth::is_authenticated_status(after.status().as_u16()) {
        return Err("session still not authenticated after login".into());
    }
    println!("[session] session established");
    Ok(())
}

/// Read an environment variable or return a descriptive error.
///
/// # Errors
///
/// Returns a boxed error when the variable is missing or contains
/// invalid UTF-8.
fn env_required(key: &str) -> Result<String, DynError> {
    std::env::var(key).map_err(|e| format!("{key}: {e}").into())
}
