#![allow(unused_crate_dependencies)]

//! # Input-user report: scan, group, export
//!
//! Walks `/dashboard/input-user` (the "Pengecekan unit" page), parses
//! each page into structured rows, groups them by a chosen column,
//! and optionally writes them to a CSV file.
//!
//! This page is different from `/dashboard` (Data Nasabah): it has
//! more columns, more total records, and supports a date range filter.
//!
//! ## Safety
//!
//! Read-only. Sends only `GET` requests.
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
//! Scan 5 pages and print a summary:
//!
//! ```sh
//! loadenv
//! cargo run --example 09_input_user_report
//! ```
//!
//! Group by `nama` (the field officer):
//!
//! ```sh
//! cargo run --example 09_input_user_report -- --pages 20 --by nama
//! ```
//!
//! Group by `kategori`:
//!
//! ```sh
//! cargo run --example 09_input_user_report -- --pages 50 --by kategori
//! ```
//!
//! Filter by creation date:
//!
//! ```sh
//! cargo run --example 09_input_user_report -- \
//!     --pages 20 --start 2026-09-01 --end 2026-09-30 --by nama
//! ```
//!
//! Export to CSV:
//!
//! ```sh
//! cargo run --example 09_input_user_report -- --pages 100 --csv input_user.csv
//! ```

#![allow(clippy::wildcard_imports)]

use core::time::Duration;
use std::path::PathBuf;

use librcekunit_examples::auth;
use librcekunit_examples::error;
use librcekunit_examples::parse;
use librcekunit_examples::prelude::*;
use librcekunit_examples::scan::{self, ScanOptions, ScanStop};

/// Boxed, thread-safe error type used as the return of `main`.
type DynError = Box<dyn core::error::Error + Send + Sync>;

/// Command-line options.
#[derive(Debug, Clone)]
struct Options {
    pages: u32,
    start_page: u32,
    delay_ms: u64,
    by: Option<String>,
    csv: Option<PathBuf>,
    sort: String,
    direction: String,
    search: String,
    start_date: Option<String>,
    end_date: Option<String>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            pages: 5,
            start_page: 1,
            delay_ms: 150,
            by: None,
            csv: None,
            sort: String::new(),
            direction: String::from("asc"),
            search: String::new(),
            start_date: None,
            end_date: None,
        }
    }
}

impl Options {
    fn from_args() -> Self {
        let mut opts = Self::default();
        let args: Vec<String> = std::env::args().collect();
        let mut iter = args.iter().skip(1);

        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--pages" => {
                    if let Some(v) = iter.next()
                        && let Ok(n) = v.parse::<u32>()
                    {
                        opts.pages = n;
                    }
                }
                "--page" => {
                    if let Some(v) = iter.next()
                        && let Ok(n) = v.parse::<u32>()
                    {
                        opts.start_page = n;
                    }
                }
                "--delay-ms" => {
                    if let Some(v) = iter.next()
                        && let Ok(n) = v.parse::<u64>()
                    {
                        opts.delay_ms = n;
                    }
                }
                "--by" => {
                    if let Some(v) = iter.next() {
                        opts.by = Some(v.clone());
                    }
                }
                "--csv" => {
                    if let Some(v) = iter.next() {
                        opts.csv = Some(PathBuf::from(v));
                    }
                }
                "--sort" => {
                    if let Some(v) = iter.next() {
                        opts.sort.clone_from(v);
                    }
                }
                "--direction" => {
                    if let Some(v) = iter.next() {
                        opts.direction.clone_from(v);
                    }
                }
                "--search" => {
                    if let Some(v) = iter.next() {
                        opts.search.clone_from(v);
                    }
                }
                "--start" => {
                    if let Some(v) = iter.next() {
                        opts.start_date = Some(v.clone());
                    }
                }
                "--end" => {
                    if let Some(v) = iter.next() {
                        opts.end_date = Some(v.clone());
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
    print_options(&opts);

    let client = connect_client().await?;

    let scan_opts = ScanOptions::default()
        .max_pages(opts.pages)
        .start_page(opts.start_page)
        .page_delay(Duration::from_millis(opts.delay_ms))
        .search(opts.search.clone())
        .sort(opts.sort.clone(), opts.direction.clone());

    println!(
        "[scan] target=input-user max_pages={} start={} delay_ms={}",
        scan_opts.max_pages, scan_opts.start_page, opts.delay_ms,
    );

    let result = run_scan(&client, &scan_opts).await?;
    print_scan_summary(&result);

    let filtered = filter_by_date(
        &result.rows,
        opts.start_date.as_deref(),
        opts.end_date.as_deref(),
    );
    if opts.start_date.is_some() || opts.end_date.is_some() {
        println!(
            "[filter] date range {:?}..{:?} matched {} of {} rows",
            opts.start_date,
            opts.end_date,
            filtered.len(),
            result.rows.len()
        );
    }

    if let Some(column) = &opts.by {
        print_grouped(&filtered, column)?;
    }

    if let Some(path) = &opts.csv {
        write_csv(&filtered, path)?;
    }

    println!("done");
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

/// Run the input-user scan with progress logging.
///
/// # Errors
///
/// Returns an error when the first page fails.
async fn run_scan(
    client: &Client,
    scan_opts: &ScanOptions,
) -> Result<scan::InputUserScanResult, DynError> {
    match scan::scan_input_user_with(client, scan_opts, |p| {
        let total = p
            .total_reported
            .map_or_else(|| String::from("?"), |n| n.to_string());
        println!(
            "[scan] page {}: +{} rows ({} total, server reports {})",
            p.page, p.rows_on_page, p.rows_total, total,
        );
    })
    .await
    {
        Ok(r) => Ok(r),
        Err(err) => {
            eprintln!("[scan] {}", error::user_message(&err));
            Err(err.into())
        }
    }
}

/// Print a summary of the scan result.
fn print_scan_summary(result: &scan::InputUserScanResult) {
    let stop = match result.stopped {
        ScanStop::MaxPagesReached => "max pages reached",
        ScanStop::EmptyPage => "empty page",
        ScanStop::ServerError => "server error",
        _ => "unknown",
    };
    println!("[summary] pages fetched: {}", result.pages_fetched);
    println!("[summary] rows collected: {}", result.rows.len());
    if let Some(total) = result.total_reported {
        println!("[summary] server reported total: {total}");
    }
    println!("[summary] stopped because: {stop}");
}

/// Print parsed options.
fn print_options(opts: &Options) {
    println!(
        "[opts] pages={} page={} delay_ms={} by={:?} csv={:?} sort={:?} direction={} search={:?} start={:?} end={:?}",
        opts.pages,
        opts.start_page,
        opts.delay_ms,
        opts.by,
        opts.csv,
        opts.sort,
        opts.direction,
        opts.search,
        opts.start_date,
        opts.end_date,
    );
}

/// Filter rows by `created_at` date range.
///
/// Dates must be `YYYY-MM-DD`. Rows are compared by the first ten
/// characters of `created_at`.
fn filter_by_date(
    rows: &[parse::InputUserRow],
    start: Option<&str>,
    end: Option<&str>,
) -> Vec<parse::InputUserRow> {
    if start.is_none() && end.is_none() {
        return rows.to_vec();
    }
    rows.iter()
        .filter(|r| {
            let date = r.created_at.get(..10).unwrap_or("");
            if let Some(s) = start
                && date < s
            {
                return false;
            }
            if let Some(e) = end
                && date > e
            {
                return false;
            }
            true
        })
        .cloned()
        .collect()
}

/// Group rows and print a sorted summary.
///
/// # Errors
///
/// Returns an error when the column name is not recognised.
#[allow(clippy::cast_precision_loss)]
fn print_grouped(rows: &[parse::InputUserRow], column: &str) -> Result<(), DynError> {
    let counts = match column {
        "no" => parse::count_by_input_user(rows, |r| r.no.clone()),
        "created_at" => parse::count_by_input_user(rows, |r| r.created_at.clone()),
        "user_id" => parse::count_by_input_user(rows, |r| r.user_id.clone()),
        "nopol" => parse::count_by_input_user(rows, |r| r.nopol.clone()),
        "lokasi" => parse::count_by_input_user(rows, |r| r.lokasi.clone()),
        "forn" => parse::count_by_input_user(rows, |r| r.forn.clone()),
        "nama" => parse::count_by_input_user(rows, |r| r.nama.clone()),
        "kategori" => parse::count_by_input_user(rows, |r| r.kategori.clone()),
        "nama_nasabah" => parse::count_by_input_user(rows, |r| r.nama_nasabah.clone()),
        "no_perjanjian" => parse::count_by_input_user(rows, |r| r.no_perjanjian.clone()),
        other => {
            eprintln!("[by] unknown column: {other}");
            eprintln!(
                "[by] valid: no, created_at, user_id, nopol, lokasi, forn, \
                 nama, kategori, nama_nasabah, no_perjanjian"
            );
            return Err(format!("unknown column: {other}").into());
        }
    };

    println!("[by] column={column}");
    let grand_total: usize = counts.iter().map(|(_, n)| *n).sum();
    for (label, count) in counts.iter().take(30) {
        let pct = if grand_total == 0 {
            0.0
        } else {
            (*count as f64) * 100.0 / (grand_total as f64)
        };
        println!("[by]   {label}: {count} ({pct:.1}%)");
    }
    if counts.len() > 30 {
        println!("[by]   ... and {} more", counts.len().saturating_sub(30));
    }
    Ok(())
}

/// Write the rows to a CSV file.
///
/// # Errors
///
/// Returns an error when the file cannot be written.
fn write_csv(rows: &[parse::InputUserRow], path: &std::path::Path) -> Result<(), DynError> {
    let csv = parse::input_user_rows_to_csv(rows);
    std::fs::write(path, csv.as_bytes())
        .map_err(|e| -> DynError { format!("write {}: {e}", path.display()).into() })?;
    println!(
        "[csv] wrote {} rows ({} bytes) to {}",
        rows.len(),
        csv.len(),
        path.display()
    );
    Ok(())
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
