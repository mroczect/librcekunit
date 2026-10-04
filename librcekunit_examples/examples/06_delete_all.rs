#![allow(unused_crate_dependencies)]

//! # Delete all dashboard data (gated, with backup and typed confirmation)
//!
//! This is the most destructive example in the workspace. It deletes
//! every row from the dashboard table. The safety model is stricter
//! than any other example.
//!
//! ## Safety model
//!
//! 1. **Read-only by default.** Running without `--execute` scans a
//!    few pages, prints a summary, writes a backup, and exits. No
//!    write request is sent.
//!
//! 2. **Automatic backup.** Before any delete, the first N pages are
//!    saved to a CSV file. Skip with `--no-backup`, but the file is
//!    your only recovery.
//!
//! 3. **Interactive terminal required.** `--execute` refuses to run
//!    when stdin is not a TTY. Piping or redirecting input is
//!    rejected before any request is sent.
//!
//! 4. **Typed confirmation with the exact count.** After the terminal
//!    check, the example prompts for `DELETE-<count>`, where `<count>`
//!    is the server-reported total. Typing anything else aborts.
//!
//! 5. **Post-delete verification.** After the delete, one page is
//!    fetched and parsed. If rows remain, a warning is printed.
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
//! Preview and backup only (safe, no delete):
//!
//! ```sh
//! loadenv
//! cargo run --example 06_delete_all
//! ```
//!
//! Preview with a custom backup path:
//!
//! ```sh
//! cargo run --example 06_delete_all -- --backup my_backup.csv
//! ```
//!
//! Execute the delete. Interactive terminal required:
//!
//! ```sh
//! cargo run --example 06_delete_all -- --execute
//! ```
//!
//! Execute without a backup (not recommended):
//!
//! ```sh
//! cargo run --example 06_delete_all -- --execute --no-backup
//! ```

#![allow(clippy::wildcard_imports)]

use std::io::IsTerminal as _;
use std::io::Write as _;
use std::path::PathBuf;

use librcekunit_examples::auth;
use librcekunit_examples::error;
use librcekunit_examples::parse;
use librcekunit_examples::prelude::*;
use librcekunit_examples::scan::{self, ScanOptions, ScanResult};

/// Boxed, thread-safe error type used as the return of `main`.
type DynError = Box<dyn core::error::Error + Send + Sync>;

/// Number of sample pages scanned in preview. Kept small because the
/// goal is a summary, not a full backup.
const SAMPLE_PAGES: u32 = 3;

/// Command-line options.
#[derive(Debug, Clone, Default)]
struct Options {
    /// When true, actually send the delete request.
    execute: bool,
    /// When true, skip the backup step.
    no_backup: bool,
    /// Custom backup path. `None` means auto-generate.
    backup_path: Option<PathBuf>,
}

impl Options {
    /// Parse command-line arguments.
    fn from_args() -> Self {
        let mut opts = Self::default();
        let args: Vec<String> = std::env::args().collect();
        let mut iter = args.iter().skip(1);

        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--execute" => opts.execute = true,
                "--no-backup" => opts.no_backup = true,
                "--backup" => {
                    if let Some(v) = iter.next() {
                        opts.backup_path = Some(PathBuf::from(v));
                    }
                }
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

    // Hard safety gate: execute mode requires a real terminal. This
    // runs before any network activity so a script cannot reach the
    // server with a write request at all.
    if opts.execute && !std::io::stdin().is_terminal() {
        eprintln!("[safety] refusing --execute: stdin is not a terminal");
        eprintln!("[safety] run without --execute to preview and back up only");
        return Err("--execute requires an interactive terminal".into());
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

    // Step 1: sample the table (read-only).
    println!("[scan] reading first {SAMPLE_PAGES} page(s) to build a summary");
    let scan_result = scan_sample(&client).await?;
    print_scan_summary(&scan_result);

    if scan_result.rows.is_empty() && scan_result.total_reported == Some(0) {
        println!("[summary] the table appears to be empty already");
        println!("done");
        return Ok(());
    }

    // Step 2: backup (unless explicitly skipped).
    let backup_path = if opts.no_backup {
        println!("[backup] SKIPPED because --no-backup was passed");
        println!("[backup] there will be no recovery path if the delete succeeds");
        None
    } else {
        let path = opts.backup_path.clone().unwrap_or_else(default_backup_path);
        if path.exists() {
            eprintln!(
                "[backup] refusing to overwrite existing file: {}",
                path.display()
            );
            eprintln!("[backup] pass --backup <other-path> or remove the file first");
            return Err(format!("backup file already exists: {}", path.display()).into());
        }
        let csv = parse::rows_to_csv(&scan_result.rows);
        std::fs::write(&path, csv.as_bytes())
            .map_err(|e| -> DynError { format!("write {}: {e}", path.display()).into() })?;
        println!(
            "[backup] wrote {} sampled rows ({} bytes) to {}",
            scan_result.rows.len(),
            csv.len(),
            path.display()
        );
        Some(path)
    };

    // Step 3: preview the delete.
    print_delete_summary(&scan_result, backup_path.as_deref());

    // Step 4: if not in execute mode, stop here.
    if !opts.execute {
        println!("[action] preview only; no delete request was sent");
        println!("[action] to actually delete, re-run with --execute");
        println!("done");
        return Ok(());
    }

    // Step 5: prompt for typed confirmation.
    let confirm_count = scan_result.total_reported.unwrap_or(scan_result.rows.len());
    let expected = format!("DELETE-{confirm_count}");
    print!("Type {expected:?} exactly to permanently delete every row: ");
    let _ = std::io::stdout().flush();

    let typed = read_line()?;
    if typed.trim() != expected {
        println!("[abort] confirmation did not match; nothing was deleted");
        println!("done");
        return Ok(());
    }

    // Step 6: send the delete request.
    println!("[delete-all] confirmation accepted, sending request");
    tracing::info!("calling delete_all");
    match client.delete_all().await {
        Ok(resp) => {
            println!("[delete-all] status={}", resp.status());
        }
        Err(err) => {
            eprintln!("[delete-all] {}", error::user_message(&err));
            return Err(err.into());
        }
    }

    // Step 7: verify.
    println!("[verify] re-reading the table to confirm it is empty");
    let after = scan_sample(&client).await?;
    print_scan_summary(&after);

    if after.rows.is_empty() && after.total_reported.unwrap_or(0) == 0 {
        println!("[verify] the table is empty; delete confirmed");
    } else {
        println!("[verify] WARNING: the table still contains rows");
        println!("[verify] the delete may have been partial or the server may have rejected it");
    }

    if let Some(path) = backup_path {
        println!("[backup] your backup is at {}", path.display());
    }

    println!("done");
    Ok(())
}

/// Print a banner that makes the destructive nature of this example
/// obvious before any work happens.
fn print_banner(opts: &Options) {
    println!("============================================================");
    println!("DELETE-ALL: Cek Unit dashboard");
    println!("============================================================");
    if opts.execute {
        println!("mode:    EXECUTE (will delete every row after confirmation)");
    } else {
        println!("mode:    PREVIEW (read-only, no delete request)");
    }
    println!(
        "backup:  {}",
        if opts.no_backup {
            "disabled"
        } else {
            "enabled"
        }
    );
    if let Some(p) = &opts.backup_path {
        println!("path:    {}", p.display());
    }
    println!("============================================================");
}

/// Sample the first few pages of the dashboard.
///
/// # Errors
///
/// Returns an error when the first page cannot be fetched or parsed.
async fn scan_sample(client: &Client) -> Result<ScanResult, DynError> {
    let opts = ScanOptions::default()
        .max_pages(SAMPLE_PAGES)
        .page_delay(core::time::Duration::from_millis(120));

    let result = scan::scan_all(client, &opts).await?;
    Ok(result)
}

/// Print a summary of the scan.
fn print_scan_summary(result: &ScanResult) {
    println!("[scan] pages fetched:  {}", result.pages_fetched);
    println!("[scan] rows sampled:   {}", result.rows.len());
    if let Some(total) = result.total_reported {
        println!("[scan] server reports: {total} total rows");
    } else {
        println!("[scan] server reports: (total not visible on sampled pages)");
    }
}

/// Print a summary of the delete that is about to happen.
fn print_delete_summary(result: &ScanResult, backup_path: Option<&std::path::Path>) {
    println!("============================================================");
    println!("ABOUT TO DELETE");
    println!("============================================================");
    if let Some(total) = result.total_reported {
        println!("Rows reported by server:  {total}");
    } else {
        println!("Rows reported by server:  (unknown, using sampled count)");
    }
    println!("Rows sampled in backup:   {}", result.rows.len());
    match backup_path {
        Some(p) => println!("Backup file:              {}", p.display()),
        None => println!("Backup file:              NONE (--no-backup)"),
    }
    println!(
        "Recovery available:       {}",
        if backup_path.is_some() { "yes" } else { "no" }
    );
    println!("============================================================");
}

/// Build a default backup path using the current unix epoch.
fn default_backup_path() -> PathBuf {
    let epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    PathBuf::from(format!("backup_dashboard_{epoch}.csv"))
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
