//! `cekunit input-user`.

use core::time::Duration;

use librcekunit_client::prelude::{Form, InputUser};
use librcekunit_examples::parse;
use librcekunit_examples::scan::{self, ScanOptions};
use serde_json::{Value, json};

use crate::cli::{DateArg, ExportArgs, InputUserCmd, InputUserReportArgs, ListArgs};
use crate::error::{Error, Result};
use crate::output;

/// Dispatch `input-user`.
///
/// # Errors
///
/// Return error kalau request gagal.
pub async fn run(cmd: InputUserCmd) -> Result<Value> {
    match cmd {
        InputUserCmd::List(args) => list(&args).await,
        InputUserCmd::Export(args) => export(&args).await,
        InputUserCmd::Report(args) => report(&args).await,
    }
}

async fn list(args: &ListArgs) -> Result<Value> {
    let client = crate::client::connect().await?;
    let mut pages = Vec::new();
    for page in 1..=args.pages {
        let mut params = Form::new();
        let _ = params.insert(String::from("page"), page.to_string());
        let _ = params.insert(String::from("direction"), args.direction.clone());
        if !args.sort.is_empty() {
            let _ = params.insert(String::from("sort"), args.sort.clone());
        }
        if !args.search.is_empty() {
            let _ = params.insert(String::from("search"), args.search.clone());
        }
        let resp = client.input_user_index_with_params(params).await?;
        let status = resp.status();
        let html = resp.text().await.unwrap_or_default();
        pages.push(json!({
            "page": page,
            "status": status.as_u16(),
            "bytes": html.len(),
        }));
    }
    Ok(json!({ "pages": pages }))
}

async fn export(args: &ExportArgs) -> Result<Value> {
    let (start, end, label) = resolve_date(&args.date)?;
    let dir = output::resolve_out_dir(args.out.as_deref())?;
    let target = dir.join(format!("cekunit_{label}.raw.csv"));

    if target.exists() && !args.force {
        return Err(Error::Io(format!(
            "file sudah ada: {}; pakai --force",
            target.display()
        )));
    }

    let bytes = output::download_csv(&start, &end).await?;
    std::fs::write(&target, &bytes)?;

    Ok(json!({
        "range": { "start": start, "end": end, "label": label },
        "path": target.display().to_string(),
        "bytes": bytes.len(),
    }))
}

async fn report(args: &InputUserReportArgs) -> Result<Value> {
    let client = crate::client::connect().await?;
    let opts = ScanOptions::default()
        .max_pages(args.pages)
        .page_delay(Duration::from_millis(150));

    let result = scan::scan_input_user_with(&client, &opts, |_| {}).await?;

    let by_json: Option<Value> = match args.by.as_deref() {
        None => None,
        Some(column) => {
            let counts = match column {
                "no" => parse::count_by_input_user(&result.rows, |r| r.no.clone()),
                "created_at" => parse::count_by_input_user(&result.rows, |r| r.created_at.clone()),
                "user_id" => parse::count_by_input_user(&result.rows, |r| r.user_id.clone()),
                "nopol" => parse::count_by_input_user(&result.rows, |r| r.nopol.clone()),
                "lokasi" => parse::count_by_input_user(&result.rows, |r| r.lokasi.clone()),
                "forn" => parse::count_by_input_user(&result.rows, |r| r.forn.clone()),
                "nama" => parse::count_by_input_user(&result.rows, |r| r.nama.clone()),
                "kategori" => parse::count_by_input_user(&result.rows, |r| r.kategori.clone()),
                "nama_nasabah" => {
                    parse::count_by_input_user(&result.rows, |r| r.nama_nasabah.clone())
                }
                "no_perjanjian" => {
                    parse::count_by_input_user(&result.rows, |r| r.no_perjanjian.clone())
                }
                other => {
                    return Err(Error::Usage(format!("kolom tidak dikenal: {other}")));
                }
            };
            let groups: Vec<Value> = counts
                .iter()
                .map(|(k, v)| json!({ "key": k, "count": v }))
                .collect();
            Some(json!({ "column": column, "groups": groups }))
        }
    };

    let csv_json: Option<Value> = match args.csv.as_deref() {
        None => None,
        Some(path) => {
            let text = parse::input_user_rows_to_csv(&result.rows);
            std::fs::write(path, text.as_bytes())?;
            Some(json!({
                "path": path.display().to_string(),
                "bytes": text.len(),
            }))
        }
    };

    Ok(json!({
        "pages_fetched": result.pages_fetched,
        "rows_collected": result.rows.len(),
        "total_reported": result.total_reported,
        "by": by_json,
        "csv": csv_json,
    }))
}

/// Resolve `DateArg` ke `(start, end, label)`.
///
/// # Errors
///
/// Return error kalau tanggal invalid.
pub fn resolve_date(arg: &DateArg) -> Result<(String, String, String)> {
    match arg {
        DateArg::Today => {
            let today = today_local()?;
            Ok((today.clone(), today.clone(), format!("today_{today}")))
        }
        DateArg::Date(date) => {
            validate_date(date)?;
            Ok((date.clone(), date.clone(), date.clone()))
        }
        DateArg::Range { start, end } => {
            validate_date(start)?;
            validate_date(end)?;
            if start > end {
                return Err(Error::Usage(format!("start {start} > end {end}")));
            }
            Ok((start.clone(), end.clone(), format!("{start}_to_{end}")))
        }
    }
}

fn today_local() -> Result<String> {
    let output = std::process::Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .map_err(|e| Error::Io(format!("jalankan date: {e}")))?;
    if !output.status.success() {
        return Err(Error::Io(String::from("date gagal")));
    }
    let raw = String::from_utf8(output.stdout).map_err(|e| Error::Io(e.to_string()))?;
    Ok(raw.trim().to_string())
}

fn validate_date(date: &str) -> Result<()> {
    let bytes = date.as_bytes();
    if bytes.len() != 10 {
        return Err(Error::Usage(format!(
            "tanggal harus YYYY-MM-DD, dapat {date:?}"
        )));
    }
    if bytes.get(4) != Some(&b'-') || bytes.get(7) != Some(&b'-') {
        return Err(Error::Usage(format!(
            "tanggal harus YYYY-MM-DD, dapat {date:?}"
        )));
    }
    for (i, b) in bytes.iter().enumerate() {
        if i == 4 || i == 7 {
            continue;
        }
        if !b.is_ascii_digit() {
            return Err(Error::Usage(format!(
                "tanggal harus YYYY-MM-DD, dapat {date:?}"
            )));
        }
    }
    Ok(())
}
