#![allow(unused_crate_dependencies)]
//! Full-table scan with grouping and CSV export.

#![allow(clippy::wildcard_imports)]

use core::time::Duration;
use std::path::PathBuf;

use librcekunit_examples::auth;
use librcekunit_examples::error;
use librcekunit_examples::parse;
use librcekunit_examples::prelude::*;
use librcekunit_examples::scan::{self, ScanOptions, ScanStop};

type DynError = Box<dyn core::error::Error + Send + Sync>;

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
                _ => {}
            }
        }
        opts
    }
}

#[tokio::main]
async fn main() -> Result<(), DynError> {
    let _ = librcekunit_client::tracing::init_tracing();
    let opts = Options::from_args();

    println!(
        "[opts] pages={} page={} delay_ms={} by={:?} csv={:?} sort={:?} direction={} search={:?}",
        opts.pages,
        opts.start_page,
        opts.delay_ms,
        opts.by,
        opts.csv,
        opts.sort,
        opts.direction,
        opts.search,
    );

    let client = match from_env().await {
        Ok(c) => c,
        Err(err) => {
            eprintln!("[config] {}", error::user_message(&err));
            return Err(err.into());
        }
    };
    ensure_session(&client).await?;

    let scan_opts = ScanOptions::default()
        .max_pages(opts.pages)
        .start_page(opts.start_page)
        .page_delay(Duration::from_millis(opts.delay_ms))
        .search(opts.search.clone())
        .sort(opts.sort.clone(), opts.direction.clone());

    println!(
        "[scan] max_pages={} start_page={} delay_ms={} search={:?}",
        scan_opts.max_pages, scan_opts.start_page, opts.delay_ms, scan_opts.search,
    );

    let result = match scan::scan_with(&client, &scan_opts, |p| {
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
        Ok(r) => r,
        Err(err) => {
            eprintln!("[scan] {}", error::user_message(&err));
            return Err(err.into());
        }
    };

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

    if let Some(column) = &opts.by {
        print_grouped(&result.rows, column)?;
    }

    if let Some(path) = &opts.csv {
        let csv = parse::rows_to_csv(&result.rows);
        std::fs::write(path, csv.as_bytes())
            .map_err(|e| -> DynError { format!("write {}: {e}", path.display()).into() })?;
        println!(
            "[csv] wrote {} rows ({} bytes) to {}",
            result.rows.len(),
            csv.len(),
            path.display()
        );
    }

    println!("done");
    Ok(())
}

fn print_grouped(rows: &[parse::CekUnitRow], column: &str) -> Result<(), DynError> {
    let counts = match column {
        "no" => parse::count_by(rows, |r| r.no.clone()),
        "no_perjanjian" => parse::count_by(rows, |r| r.no_perjanjian.clone()),
        "nama_nasabah" => parse::count_by(rows, |r| r.nama_nasabah.clone()),
        "nopol" => parse::count_by(rows, |r| r.nopol.clone()),
        "coll" => parse::count_by(rows, |r| r.coll.clone()),
        "pic" => parse::count_by(rows, |r| r.pic.clone()),
        "kategori" => parse::count_by(rows, |r| r.kategori.clone()),
        "jto" => parse::count_by(rows, |r| r.jto.clone()),
        "no_rangka" => parse::count_by(rows, |r| r.no_rangka.clone()),
        "no_mesin" => parse::count_by(rows, |r| r.no_mesin.clone()),
        "merk" => parse::count_by(rows, |r| r.merk.clone()),
        "type" => parse::count_by(rows, |r| r.r#type.clone()),
        "warna" => parse::count_by(rows, |r| r.warna.clone()),
        "status" => parse::count_by(rows, |r| r.status.clone()),
        "actual_penyelesaian" => parse::count_by(rows, |r| r.actual_penyelesaian.clone()),
        "angsuran_ke" => parse::count_by(rows, |r| r.angsuran_ke.clone()),
        "tenor" => parse::count_by(rows, |r| r.tenor.clone()),
        other => {
            eprintln!("[by] unknown column: {other}");
            return Err(format!("unknown column: {other}").into());
        }
    };

    println!("[by] column={column}");
    let grand_total: usize = counts.iter().map(|(_, n)| *n).sum();
    for (label, count) in &counts {
        let pct = if grand_total == 0 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            {
                (*count as f64) * 100.0 / (grand_total as f64)
            }
        };
        println!("[by]   {label}: {count} ({pct:.1}%)");
    }
    Ok(())
}

async fn ensure_session(client: &Client) -> Result<(), DynError> {
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

fn env_required(key: &str) -> Result<String, DynError> {
    std::env::var(key).map_err(|e| format!("{key}: {e}").into())
}
