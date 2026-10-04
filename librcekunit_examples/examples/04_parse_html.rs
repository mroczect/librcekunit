#![allow(unused_crate_dependencies)]
//! Parse dashboard HTML into structured data.

#![allow(clippy::wildcard_imports)]

use std::path::PathBuf;

use librcekunit_examples::auth;
use librcekunit_examples::error;
use librcekunit_examples::parse;
use librcekunit_examples::prelude::*;

type DynError = Box<dyn core::error::Error + Send + Sync>;

#[derive(Debug, Clone)]
struct Options {
    sort: String,
    direction: String,
    search: String,
    start_page: u32,
    pages: u32,
    count_by: Option<String>,
    csv: Option<PathBuf>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            sort: String::new(),
            direction: String::from("asc"),
            search: String::new(),
            start_page: 1,
            pages: 1,
            count_by: None,
            csv: None,
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
                "--count-by" => {
                    if let Some(v) = iter.next() {
                        opts.count_by = Some(v.clone());
                    }
                }
                "--csv" => {
                    if let Some(v) = iter.next() {
                        opts.csv = Some(PathBuf::from(v));
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
        "[opts] sort={:?} direction={} search={:?} page={} pages={} count_by={:?} csv={:?}",
        opts.sort,
        opts.direction,
        opts.search,
        opts.start_page,
        opts.pages,
        opts.count_by,
        opts.csv,
    );

    let client = match from_env().await {
        Ok(c) => c,
        Err(err) => {
            eprintln!("[config] {}", error::user_message(&err));
            return Err(err.into());
        }
    };
    ensure_session(&client).await?;

    let mut all_rows: Vec<parse::CekUnitRow> = Vec::new();
    let mut reported_total: Option<usize> = None;

    let last = opts.start_page.saturating_add(opts.pages).saturating_sub(1);
    for page in opts.start_page..=last {
        let html = fetch_page(&client, &opts, page).await?;

        if reported_total.is_none() {
            reported_total = parse::parse_total_records(&html);
        }

        match parse::parse_dashboard_rows(&html) {
            Ok(rows) => {
                println!("[page {page}] parsed {} rows", rows.len());
                all_rows.extend(rows);
            }
            Err(err) => {
                eprintln!("[page {page}] parse error: {err}");
                return Err(err.into());
            }
        }
    }

    println!("[summary] fetched {} pages", opts.pages);
    println!("[summary] parsed {} total rows", all_rows.len());
    if let Some(total) = reported_total {
        println!("[summary] server reports {total} rows across all pages");
    }

    if let Some(column) = &opts.count_by {
        let counts = group_by(&all_rows, column)?;
        println!("[count-by] column={column}");
        for (label, count) in &counts {
            println!("[count-by]   {label}: {count}");
        }
    }

    if let Some(path) = &opts.csv {
        let csv = parse::rows_to_csv(&all_rows);
        std::fs::write(path, csv.as_bytes())
            .map_err(|e| -> DynError { format!("write {}: {e}", path.display()).into() })?;
        println!(
            "[csv] wrote {} rows ({} bytes) to {}",
            all_rows.len(),
            csv.len(),
            path.display()
        );
    }

    println!("done");
    Ok(())
}

fn group_by(rows: &[parse::CekUnitRow], column: &str) -> Result<Vec<(String, usize)>, DynError> {
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
            eprintln!("[count-by] unknown column: {other}");
            return Err(format!("unknown column: {other}").into());
        }
    };
    Ok(counts)
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

async fn fetch_page(client: &Client, opts: &Options, page: u32) -> Result<String, DynError> {
    let mut params = Form::new();
    let _ = params.insert(String::from("page"), page.to_string());
    let _ = params.insert(String::from("direction"), opts.direction.clone());
    if !opts.sort.is_empty() {
        let _ = params.insert(String::from("sort"), opts.sort.clone());
    }
    if !opts.search.is_empty() {
        let _ = params.insert(String::from("search"), opts.search.clone());
    }

    let resp = client.dashboard_index_with_params(params).await?;
    let status = resp.status();
    if !status.is_success() {
        return Err(format!("server returned {status}").into());
    }
    resp.text()
        .await
        .map_err(|e| -> DynError { format!("read body: {e}").into() })
}

fn env_required(key: &str) -> Result<String, DynError> {
    std::env::var(key).map_err(|e| format!("{key}: {e}").into())
}
