#![allow(unused_crate_dependencies)]

//! # Replace all dashboard data from a CSV file
//!
//! Validates a CSV file locally, then — only with explicit
//! confirmation on a terminal — backs up the current server data,
//! deletes every row, and uploads the CSV.
//!
//! ## Why this example is dangerous
//!
//! It combines the two most destructive operations in the workspace:
//! `delete_all` and a bulk insert. A mistake here removes production
//! data and replaces it with something else.
//!
//! ## Safety model
//!
//! 1. **Local validation first.** The CSV is parsed and checked before
//!    any network activity. A malformed file stops the workflow
//!    immediately.
//!
//! 2. **Preview by default.** Without `--execute`, the example prints
//!    the validation summary and the planned steps, then exits. No
//!    write request is sent.
//!
//! 3. **Automatic backup.** Before any delete, the first N pages of
//!    the current server data are saved to a CSV file. If the upload
//!    fails partway, the backup is the recovery path.
//!
//! 4. **Interactive terminal required.** `--execute` refuses to run
//!    when stdin is not a TTY.
//!
//! 5. **Typed confirmation includes the row count.** The prompt is
//!    `REPLACE-<rows>`. Any mismatch aborts.
//!
//! 6. **Post-insert verification.** After the upload, the first page
//!    is re-scanned and the row count is compared with the CSV.
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
//! ## Usage
//!
//! Validate only (safe, no writes):
//!
//! ```sh
//! loadenv
//! cargo run --example 08_replace_all -- --file data.csv
//! ```
//!
//! Execute the full replace (interactive terminal required):
//!
//! ```sh
//! cargo run --example 08_replace_all -- --file data.csv --execute
//! ```
//!
//! Skip the automatic backup (not recommended):
//!
//! ```sh
//! cargo run --example 08_replace_all -- --file data.csv --execute --no-backup
//! ```

#![allow(clippy::wildcard_imports)]

use std::io::IsTerminal as _;
use std::io::Write as _;
use std::path::PathBuf;

use librcekunit_examples::auth;
use librcekunit_examples::csv_check;
use librcekunit_examples::error;
use librcekunit_examples::parse;
use librcekunit_examples::prelude::*;
use librcekunit_examples::scan::{self, ScanOptions, ScanResult};

/// Boxed, thread-safe error type used as the return of `main`.
type DynError = Box<dyn core::error::Error + Send + Sync>;

/// Number of pages scanned to build the pre-delete backup.
const BACKUP_PAGES: u32 = 5;

/// Command-line options.
#[derive(Debug, Clone, Default)]
struct Options {
    /// Path to the CSV file to validate and upload.
    file: Option<PathBuf>,
    /// When true, runs the destructive workflow.
    execute: bool,
    /// When true, skip the pre-delete backup.
    no_backup: bool,
}

impl Options {
    /// Parse command-line arguments.
    fn from_args() -> Self {
        let mut opts = Self::default();
        let args: Vec<String> = std::env::args().collect();
        let mut iter = args.iter().skip(1);

        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--file" => {
                    if let Some(v) = iter.next() {
                        opts.file = Some(PathBuf::from(v));
                    }
                }
                "--execute" => opts.execute = true,
                "--no-backup" => opts.no_backup = true,
                _ => {}
            }
        }
        opts
    }
}

/// Entry point.
#[tokio::main]
async fn main() -> Result<(), DynError> {
    let _ = librcekunit_client::tracing::init_tracing();

    let opts = Options::from_args();
    print_banner(&opts);

    let file = opts
        .file
        .as_ref()
        .ok_or("missing required flag: --file <path>")?;

    // Step 1: validate the CSV locally. No network, no disk write.
    println!("[step 1] validating CSV: {}", file.display());
    let summary = validate_file(file)?;
    print_summary(&summary);

    // Step 2: if not in execute mode, stop here.
    if !opts.execute {
        print_plan(&summary, false);
        println!("[action] validation-only mode; no write request was sent");
        println!("[action] to run the full workflow, re-run with --execute");
        println!("done");
        return Ok(());
    }

    // Step 3: hard safety gate.
    check_terminal()?;

    // Step 4: build the client and ensure a session.
    let client = connect_client().await?;

    // Step 5: pre-delete backup.
    let backup_path = prepare_backup(&client, &opts).await?;

    // Step 6: confirm.
    if !confirm_replace(&summary, backup_path.as_deref())? {
        println!("done");
        return Ok(());
    }

    // Step 7: delete all.
    delete_all(&client).await?;

    // Step 8: upload.
    upload_csv(&client, file, backup_path.as_deref()).await?;

    // Step 9: verify.
    verify_replace(&client, &summary, backup_path.as_deref()).await?;

    println!("done");
    Ok(())
}

/// Ensure stdin is an interactive terminal.
///
/// # Errors
///
/// Returns an error when stdin is not a TTY.
fn check_terminal() -> Result<(), DynError> {
    if !std::io::stdin().is_terminal() {
        eprintln!("[safety] refusing --execute: stdin is not a terminal");
        eprintln!("[safety] run without --execute to validate the file only");
        return Err("--execute requires an interactive terminal".into());
    }
    Ok(())
}

/// Build the client from environment and ensure a session.
///
/// # Errors
///
/// Returns an error when the client cannot be built or login fails.
async fn connect_client() -> Result<Client, DynError> {
    tracing::info!("building client from environment");
    let client = match from_env().await {
        Ok(c) => c,
        Err(err) => {
            eprintln!("[config] {}", error::user_message(&err));
            return Err(err.into());
        }
    };
    ensure_session(&client).await?;
    Ok(client)
}

/// Prepare the pre-delete backup.
///
/// # Errors
///
/// Returns an error when the backup file cannot be written.
async fn prepare_backup(client: &Client, opts: &Options) -> Result<Option<PathBuf>, DynError> {
    if opts.no_backup {
        println!("[step 2] backup SKIPPED (--no-backup was passed)");
        println!("[step 2] no recovery path if the upload fails");
        return Ok(None);
    }

    println!("[step 2] backing up current server data (first {BACKUP_PAGES} pages)");
    let scan = scan_sample(client, BACKUP_PAGES).await?;
    let path = default_backup_path();
    if path.exists() {
        return Err(format!("backup file already exists: {}", path.display()).into());
    }
    let csv = parse::rows_to_csv(&scan.rows);
    std::fs::write(&path, csv.as_bytes())
        .map_err(|e| -> DynError { format!("write {}: {e}", path.display()).into() })?;
    println!(
        "[step 2] wrote {} rows ({} bytes) to {}",
        scan.rows.len(),
        csv.len(),
        path.display()
    );
    Ok(Some(path))
}

/// Ask for typed confirmation.
///
/// # Errors
///
/// Returns an error when reading from stdin fails.
fn confirm_replace(
    summary: &csv_check::CsvSummary,
    backup_path: Option<&std::path::Path>,
) -> Result<bool, DynError> {
    print_plan(summary, true);
    let expected = format!("REPLACE-{}", summary.data_rows);
    print!("Type {expected:?} to proceed: ");
    let _ = std::io::stdout().flush();

    let typed = read_line()?;
    if typed.trim() != expected {
        println!("[abort] confirmation did not match; nothing was changed");
        if let Some(p) = backup_path {
            println!("[note] backup is still at {}", p.display());
        }
        return Ok(false);
    }
    Ok(true)
}

/// Send the delete-all request.
///
/// # Errors
///
/// Returns an error when the request fails.
async fn delete_all(client: &Client) -> Result<(), DynError> {
    println!("[step 3] delete-all: sending request");
    tracing::info!("calling delete_all");
    match client.delete_all().await {
        Ok(resp) => {
            println!("[step 3] status={}", resp.status());
            Ok(())
        }
        Err(err) => {
            eprintln!("[step 3] {}", error::user_message(&err));
            Err(err.into())
        }
    }
}

/// Upload the CSV file.
///
/// # Errors
///
/// Returns an error when the upload fails.
async fn upload_csv(
    client: &Client,
    file: &std::path::Path,
    backup_path: Option<&std::path::Path>,
) -> Result<(), DynError> {
    println!("[step 4] uploading CSV: {}", file.display());
    tracing::info!("calling import_csv");
    match client.import_csv(file).await {
        Ok(resp) => {
            println!("[step 4] status={}", resp.status());
            let body = resp.text().await.unwrap_or_default();
            let preview_len = body.len().min(300);
            let preview = body.get(..preview_len).unwrap_or(&body);
            println!("[step 4] response preview: {preview}");
            Ok(())
        }
        Err(err) => {
            eprintln!("[step 4] {}", error::user_message(&err));
            if let Some(p) = backup_path {
                eprintln!("[step 4] upload failed; restore from {}", p.display());
            } else {
                eprintln!("[step 4] upload failed; no backup available");
            }
            Err(err.into())
        }
    }
}

/// Verify the replace by re-scanning the first page.
///
/// # Errors
///
/// Returns an error when the verification scan fails.
async fn verify_replace(
    client: &Client,
    summary: &csv_check::CsvSummary,
    backup_path: Option<&std::path::Path>,
) -> Result<(), DynError> {
    println!("[step 5] verifying: re-scanning first page");
    let after = scan_sample(client, 1).await?;
    println!("[step 5] rows visible now: {}", after.rows.len());
    if let Some(total) = after.total_reported {
        println!("[step 5] server reports: {total} total rows");
        if total == summary.data_rows {
            println!("[step 5] row count matches the CSV; replace confirmed");
        } else {
            println!(
                "[step 5] WARNING: expected {}, server reports {total}",
                summary.data_rows
            );
        }
    }
    if let Some(p) = backup_path {
        println!("[note] previous data backed up at {}", p.display());
    }
    Ok(())
}

/// Print a banner.
fn print_banner(opts: &Options) {
    println!("============================================================");
    println!("REPLACE-ALL: validate, backup, delete-all, upload");
    println!("============================================================");
    println!(
        "mode:    {}",
        if opts.execute {
            "EXECUTE"
        } else {
            "VALIDATE-ONLY (no writes)"
        }
    );
    println!(
        "backup:  {}",
        if opts.no_backup {
            "disabled"
        } else {
            "enabled"
        }
    );
    if let Some(f) = &opts.file {
        println!("file:    {}", f.display());
    } else {
        println!("file:    (not provided; pass --file <path>)");
    }
    println!("============================================================");
}

/// Read and validate the file.
///
/// # Errors
///
/// Returns an error when the file does not exist, cannot be read, or
/// fails validation.
fn validate_file(path: &std::path::Path) -> Result<csv_check::CsvSummary, DynError> {
    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }
    let text = std::fs::read_to_string(path)
        .map_err(|e| -> DynError { format!("read {}: {e}", path.display()).into() })?;
    let summary = csv_check::validate(&text)
        .map_err(|e| -> DynError { format!("validation failed: {e}").into() })?;
    Ok(summary)
}

/// Print the validation summary.
fn print_summary(summary: &csv_check::CsvSummary) {
    println!("[validate] header ({} columns):", summary.header.len());
    for (i, col) in summary.header.iter().enumerate() {
        println!("[validate]   {i}: {col}");
    }
    println!("[validate] data rows: {}", summary.data_rows);
    match summary.no_perjanjian_index {
        Some(i) => println!("[validate] no_perjanjian column index: {i}"),
        None => println!("[validate] no_perjanjian column: MISSING"),
    }
    println!("[validate] preview:");
    for (i, row) in summary.preview_rows.iter().enumerate() {
        let joined = row.join(" | ");
        println!("[validate]   {}: {joined}", i.saturating_add(1));
    }
    println!("[validate] file is well-formed");
}

/// Print the plan that would run in execute mode.
fn print_plan(summary: &csv_check::CsvSummary, execute: bool) {
    println!("============================================================");
    if execute {
        println!("PLAN (about to execute)");
    } else {
        println!("PLAN (validation only)");
    }
    println!("============================================================");
    println!("1. Back up current server data to a local CSV file.");
    println!("2. Delete every row on the server (delete_all).");
    println!(
        "3. Upload {} rows from the provided CSV file.",
        summary.data_rows
    );
    println!("4. Re-scan the first page and verify the row count.");
    println!("============================================================");
}

/// Sample the first N pages of the dashboard.
///
/// # Errors
///
/// Returns an error when the first page cannot be fetched or parsed.
async fn scan_sample(client: &Client, pages: u32) -> Result<ScanResult, DynError> {
    let opts = ScanOptions::default()
        .max_pages(pages)
        .page_delay(core::time::Duration::from_millis(120));
    let result = scan::scan_all(client, &opts).await?;
    Ok(result)
}

/// Build a default backup path using the current unix epoch.
fn default_backup_path() -> PathBuf {
    let epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    PathBuf::from(format!("pre_replace_backup_{epoch}.csv"))
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
/// Returns an error when the probe or login step fails.
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
