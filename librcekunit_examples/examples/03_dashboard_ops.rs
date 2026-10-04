#![allow(unused_crate_dependencies)]
//! # Dashboard operations (read-only by default)
//!
//! Demonstrates the endpoints used by the Cek Unit admin dashboard
//! while keeping destructive operations behind multiple safety gates.
//!
//! ## Safety model
//!
//! This example never modifies the server unless you explicitly ask it
//! to AND confirm interactively on a real terminal.
//!
//! 1. **Read-only by default.** Running without arguments performs
//!    only `GET` requests: list dashboard pages, fetch unique values.
//!
//! 2. **Preview mode.** `--preview-delete-category` and
//!    `--preview-delete-all` print a detailed warning and exit without
//!    sending any write request.
//!
//! 3. **Terminal required.** `--delete-category` and `--delete-all`
//!    refuse to run when stdin is not a TTY. Piping input from a
//!    script or a here-doc is rejected.
//!
//! 4. **Typed confirmation.** After passing the terminal check, the
//!    example prompts for a specific phrase on stdin. Any mismatch
//!    aborts the operation.
//!
//! 5. **No auto-execute flag.** There is no single flag that both
//!    selects the operation and confirms it.
//!
//! ## Read operations
//!
//! - `GET /dashboard` with sort, direction, search, pagination.
//! - `GET /dashboard/cekunit/get-unique-values` for dropdowns.
//!
//! ## Write operations (gated)
//!
//! - `POST /dashboard/cekunit/delete-by-category`
//! - `POST /dashboard/delete-all`
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
//! Read-only list:
//!
//! ```sh
//! loadenv
//! cargo run -p librcekunit_examples --example 03_dashboard_ops
//! ```
//!
//! Read-only list with sort and search:
//!
//! ```sh
//! cargo run -p librcekunit_examples --example 03_dashboard_ops -- \
//!     --sort nopol --direction desc --search BP2927GU --pages 2
//! ```
//!
//! Fetch unique values for a column:
//!
//! ```sh
//! cargo run -p librcekunit_examples --example 03_dashboard_ops -- \
//!     --unique kategori
//! ```
//!
//! Preview a delete (no request is sent):
//!
//! ```sh
//! cargo run -p librcekunit_examples --example 03_dashboard_ops -- \
//!     --preview-delete-category kategori=A
//! ```
//!
//! Perform a delete (interactive confirmation required):
//!
//! ```sh
//! cargo run -p librcekunit_examples --example 03_dashboard_ops -- \
//!     --delete-category kategori=A
//! ```
//!
//! Preview a delete-all (no request is sent):
//!
//! ```sh
//! cargo run -p librcekunit_examples --example 03_dashboard_ops -- \
//!     --preview-delete-all
//! ```
//!
//! Perform a delete-all (interactive confirmation required):
//!
//! ```sh
//! cargo run -p librcekunit_examples --example 03_dashboard_ops -- \
//!     --delete-all
//! ```

#![allow(clippy::wildcard_imports)]

use std::io::IsTerminal as _;
use std::io::Write as _;

use librcekunit_examples::auth;
use librcekunit_examples::error;
use librcekunit_examples::prelude::*;

/// Boxed, thread-safe error type used as the return of `main`.
type DynError = Box<dyn core::error::Error + Send + Sync>;

/// The phrase a user must type to confirm `delete-by-category`.
const CONFIRM_CATEGORY: &str = "HAPUS";

/// The phrase a user must type to confirm `delete-all`.
const CONFIRM_ALL: &str = "HAPUS SEMUA";

/// What the user asked us to do, after parsing arguments.
///
/// Only one operation can be selected per run. Later flags override
/// earlier ones, so `--preview-delete-all --delete-all` ends with
/// `DeleteAll`, and `--delete-all --preview-delete-all` ends with
/// `PreviewDeleteAll`.
#[derive(Debug, Clone)]
enum Action {
    /// Nothing beyond listing.
    ListOnly,
    /// Fetch unique values for this column.
    Unique(String),
    /// Show what would be deleted, send no request.
    PreviewDeleteCategory { column: String, value: String },
    /// Show what would be deleted, send no request.
    PreviewDeleteAll,
    /// Delete rows where `column` equals `value`, after confirmation.
    DeleteCategory { column: String, value: String },
    /// Delete every row, after confirmation.
    DeleteAll,
}

/// Command-line options after parsing.
#[derive(Debug, Clone)]
struct Options {
    /// Sort column for the dashboard list. Empty means server default.
    sort: String,
    /// Sort direction, `asc` or `desc`.
    direction: String,
    /// Search term. Empty means no filter.
    search: String,
    /// Starting page, 1-based.
    start_page: u32,
    /// Number of pages to fetch.
    pages: u32,
    /// What to do after listing.
    action: Action,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            sort: String::new(),
            direction: String::from("asc"),
            search: String::new(),
            start_page: 1,
            pages: 1,
            action: Action::ListOnly,
        }
    }
}

impl Options {
    /// Parse command-line arguments.
    ///
    /// Unknown flags are ignored. Malformed arguments such as
    /// `--delete-category` without `column=value` set the action to
    /// `ListOnly` and print a warning.
    fn from_args() -> Self {
        let mut opts = Self::default();
        let args: Vec<String> = std::env::args().collect();
        let mut iter = args.iter().skip(1);

        while let Some(arg) = iter.next() {
            match arg.as_str() {
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
                "--page" => {
                    if let Some(v) = iter.next()
                        && let Ok(n) = v.parse::<u32>()
                    {
                        opts.start_page = n;
                    }
                }
                "--pages" => {
                    if let Some(v) = iter.next()
                        && let Ok(n) = v.parse::<u32>()
                    {
                        opts.pages = n;
                    }
                }
                "--unique" => {
                    if let Some(v) = iter.next() {
                        opts.action = Action::Unique(v.clone());
                    }
                }
                "--preview-delete-category" => {
                    if let Some(v) = iter.next() {
                        if let Some((col, val)) = v.split_once('=') {
                            opts.action = Action::PreviewDeleteCategory {
                                column: col.to_owned(),
                                value: val.to_owned(),
                            };
                        } else {
                            eprintln!(
                                "[warn] --preview-delete-category expects column=value, got {v:?}"
                            );
                        }
                    }
                }
                "--delete-category" => {
                    if let Some(v) = iter.next() {
                        if let Some((col, val)) = v.split_once('=') {
                            opts.action = Action::DeleteCategory {
                                column: col.to_owned(),
                                value: val.to_owned(),
                            };
                        } else {
                            eprintln!("[warn] --delete-category expects column=value, got {v:?}");
                        }
                    }
                }
                "--preview-delete-all" => {
                    opts.action = Action::PreviewDeleteAll;
                }
                "--delete-all" => {
                    opts.action = Action::DeleteAll;
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

    // Hard safety gate: destructive operations require a real TTY.
    //
    // Checking before any network activity means a misconfigured script
    // cannot reach the server with a write request at all. The user
    // gets an immediate, clear rejection.
    if is_interactive_required(&opts.action) && !std::io::stdin().is_terminal() {
        eprintln!("[safety] refusing destructive operation: stdin is not a terminal");
        eprintln!(
            "[safety] pipe-safe alternative: --preview-delete-category / --preview-delete-all"
        );
        return Err("destructive operation requires an interactive terminal".into());
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

    // Always list. This is read-only and safe.
    list_pages(&client, &opts).await?;

    // Dispatch on the selected action.
    match &opts.action {
        Action::ListOnly => {
            println!("[action] list-only; no write request was sent");
        }
        Action::Unique(column) => {
            fetch_unique_values(&client, column).await?;
            println!("[action] unique; no write request was sent");
        }
        Action::PreviewDeleteCategory { column, value } => {
            preview_delete_category(&client, column, value).await?;
        }
        Action::PreviewDeleteAll => {
            preview_delete_all(&client).await?;
        }
        Action::DeleteCategory { column, value } => {
            // Extra safety: show the preview before the confirmation
            // prompt, so the user sees the same summary they would
            // have seen in preview mode.
            preview_delete_category(&client, column, value).await?;
            confirm_and_delete_category(&client, column, value).await?;
        }
        Action::DeleteAll => {
            preview_delete_all(&client).await?;
            confirm_and_delete_all(&client).await?;
        }
    }

    println!("done");
    Ok(())
}

/// Whether the given action requires an interactive terminal.
const fn is_interactive_required(action: &Action) -> bool {
    matches!(action, Action::DeleteCategory { .. } | Action::DeleteAll)
}

/// Print the parsed options so a reader can see exactly what will run.
fn print_options(opts: &Options) {
    println!(
        "[opts] sort={:?} direction={} search={:?} page={} pages={}",
        opts.sort, opts.direction, opts.search, opts.start_page, opts.pages,
    );
    println!("[opts] action={:?}", opts.action);
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

/// Fetch one or more pages from the dashboard list endpoint.
///
/// # Errors
///
/// Returns an error when any request fails.
async fn list_pages(client: &Client, opts: &Options) -> Result<(), DynError> {
    let last_page = opts.start_page.saturating_add(opts.pages).saturating_sub(1);
    for page in opts.start_page..=last_page {
        let mut params = Form::new();
        let _ = params.insert(String::from("page"), page.to_string());
        let _ = params.insert(String::from("direction"), opts.direction.clone());
        if !opts.sort.is_empty() {
            let _ = params.insert(String::from("sort"), opts.sort.clone());
        }
        if !opts.search.is_empty() {
            let _ = params.insert(String::from("search"), opts.search.clone());
        }

        tracing::info!(page, "fetching dashboard page");
        match client.dashboard_index_with_params(params).await {
            Ok(resp) => {
                let status = resp.status();
                let html = resp.text().await.unwrap_or_default();
                println!("[page {page}] status={status} bytes={}", html.len());
            }
            Err(err) => {
                eprintln!("[page {page}] {}", error::user_message(&err));
                return Err(err.into());
            }
        }
    }
    Ok(())
}

/// Fetch unique values for a column.
///
/// # Errors
///
/// Returns an error when the request fails.
async fn fetch_unique_values(client: &Client, column: &str) -> Result<(), DynError> {
    tracing::info!(column, "fetching unique values");
    let resp = match client.get_unique_values(column).await {
        Ok(r) => r,
        Err(err) => {
            eprintln!("[unique] {}", error::user_message(&err));
            return Err(err.into());
        }
    };

    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    println!("[unique] column={column} status={status}");
    println!("[unique] raw body: {body}");
    Ok(())
}

/// Show what `delete-by-category` would remove. Sends no write request.
///
/// # Errors
///
/// Returns an error when the read request fails.
async fn preview_delete_category(
    client: &Client,
    column: &str,
    value: &str,
) -> Result<(), DynError> {
    println!("[preview] would delete rows where {column} = {value}");
    println!("[preview] performing a read-only check first");

    let mut params = Form::new();
    let _ = params.insert(String::from("search"), value.to_owned());
    let _ = params.insert(String::from("page"), String::from("1"));

    match client.dashboard_index_with_params(params).await {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            let occurrences = body.matches(value).count();
            println!("[preview] status={status}");
            println!("[preview] the search term {value:?} appears {occurrences} time(s) on page 1");
        }
        Err(err) => {
            eprintln!("[preview] {}", error::user_message(&err));
            return Err(err.into());
        }
    }

    println!("[preview] no write request was sent");
    println!("[preview] to actually delete, run with --delete-category {column}={value}");
    println!("[preview] you will be asked to type {CONFIRM_CATEGORY:?} to confirm");
    Ok(())
}

/// Show what `delete-all` would remove. Sends no write request.
///
/// # Errors
///
/// Returns an error when the read request fails.
async fn preview_delete_all(client: &Client) -> Result<(), DynError> {
    println!("[preview] would delete every row in the table");
    println!("[preview] reading page 1 to estimate table size");

    match client.dashboard_index().await {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            let has_counter = body.contains("dari");
            println!("[preview] status={status}");
            println!("[preview] page contains a total counter: {has_counter}");
        }
        Err(err) => {
            eprintln!("[preview] {}", error::user_message(&err));
            return Err(err.into());
        }
    }

    println!("[preview] no write request was sent");
    println!("[preview] to actually delete everything, run with --delete-all");
    println!("[preview] you will be asked to type {CONFIRM_ALL:?} to confirm");
    Ok(())
}

/// Delete rows where `column` equals `value`, after confirmation.
///
/// # Errors
///
/// Returns an error when the request fails, or when the user does not
/// type the expected confirmation phrase.
async fn confirm_and_delete_category(
    client: &Client,
    column: &str,
    value: &str,
) -> Result<(), DynError> {
    println!("============================================================");
    println!("DESTRUCTIVE OPERATION");
    println!("============================================================");
    println!("This will delete every row where:");
    println!("  {column} = {value}");
    println!();
    println!("There is no undo.");
    println!();
    print!("Type {CONFIRM_CATEGORY:?} to proceed, anything else to abort: ");
    let _ = std::io::stdout().flush();

    let typed = read_line()?;
    if typed.trim() != CONFIRM_CATEGORY {
        println!("[abort] confirmation did not match; nothing was deleted");
        return Ok(());
    }

    println!("[delete-category] confirmed, sending request");
    match client.delete_by_category(column, value).await {
        Ok(resp) => {
            println!("[delete-category] status={}", resp.status());
        }
        Err(err) => {
            eprintln!("[delete-category] {}", error::user_message(&err));
            return Err(err.into());
        }
    }
    Ok(())
}

/// Delete every row, after confirmation.
///
/// # Errors
///
/// Returns an error when the request fails, or when the user does not
/// type the expected confirmation phrase.
async fn confirm_and_delete_all(client: &Client) -> Result<(), DynError> {
    println!("============================================================");
    println!("DESTRUCTIVE OPERATION");
    println!("============================================================");
    println!("This will delete every row in the table.");
    println!();
    println!("There is no undo.");
    println!();
    print!("Type {CONFIRM_ALL:?} to proceed, anything else to abort: ");
    let _ = std::io::stdout().flush();

    let typed = read_line()?;
    if typed.trim() != CONFIRM_ALL {
        println!("[abort] confirmation did not match; nothing was deleted");
        return Ok(());
    }

    println!("[delete-all] confirmed, sending request");
    match client.delete_all().await {
        Ok(resp) => {
            println!("[delete-all] status={}", resp.status());
        }
        Err(err) => {
            eprintln!("[delete-all] {}", error::user_message(&err));
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

/// Read an environment variable or return a descriptive error.
///
/// # Errors
///
/// Returns a boxed error when the variable is missing or contains
/// invalid UTF-8.
fn env_required(key: &str) -> Result<String, DynError> {
    std::env::var(key).map_err(|e| format!("{key}: {e}").into())
}
