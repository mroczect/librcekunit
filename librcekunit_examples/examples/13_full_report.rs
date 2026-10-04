#![allow(unused_crate_dependencies)]
#![allow(clippy::std_instead_of_alloc)]

//! # Full report: download, dedupe, pivot
//!
//! Runs the entire reporting pipeline in a single command:
//!
//! 1. Download a CSV from the server (like example 10).
//! 2. Deduplicate rows by chosen columns (like example 11).
//! 3. Pivot the result (like example 12).
//!
//! Three files are written to the output directory:
//!
//! - `cekunit_<label>.raw.csv`       - the raw download
//! - `cekunit_<label>.deduped.csv`   - after deduplication
//! - `cekunit_<label>.report.csv`    - the final pivot
//!
//! ## Safety
//!
//! - The download step is read-only against the server.
//! - No file is overwritten unless `--force` is passed.
//! - If any step fails, no further step runs.
//!
//! ## Large date ranges
//!
//! The server builds the CSV in memory before responding, so a wide
//! range (a month or more) can take several minutes. The example
//! streams the response body chunk by chunk and prints progress every
//! MiB, so a slow download is visible rather than silent.
//!
//! If the default 30 second timeout is not enough, set:
//!
//! ```sh
//! export LIBRCEKUNIT_TIMEOUT_SECS=1800
//! ```
//!
//! If the server itself refuses to build a wide range, split it into
//! weekly batches.
//!
//! ## Environment variables
//!
//! | Name                      | Required | Purpose                                       |
//! |---------------------------|----------|-----------------------------------------------|
//! | `LIBRCEKUNIT_BASE_URL`    | yes      | Root URL of the Laravel backend.              |
//! | `LIBRCEKUNIT_COOKIE_FILE` | no       | When set, session is persisted between runs.  |
//! | `LIBRCEKUNIT_EMAIL`       | if no session | Login email address.                     |
//! | `LIBRCEKUNIT_PASSWORD`    | if no session | Login password.                          |
//! | `LIBRCEKUNIT_TIMEOUT_SECS`| no       | Request timeout in seconds. Default 30.       |
//! | `RUST_LOG`                | no       | Tracing filter. Defaults to `info`.           |
//!
//! ## Usage
//!
//! ```sh
//! # Today's report with default options
//! cargo run --example 13_full_report -- today
//!
//! # A specific date
//! cargo run --example 13_full_report -- date 2026-09-01
//!
//! # A date range
//! cargo run --example 13_full_report -- range 2026-09-01 2026-09-30
//!
//! # Change the dedupe key
//! cargo run --example 13_full_report -- today --dedup-by user_id,nopol
//!
//! # Change the pivot
//! cargo run --example 13_full_report -- today \
//!     --pivot-rows nama --pivot-values nopol --pivot-agg count-distinct
//!
//! # Custom output directory
//! cargo run --example 13_full_report -- today --out ~/Documents/reports
//! ```
//!
//! ## Defaults
//!
//! - `--dedup-by user_id,nopol`
//! - `--pivot-rows nama`
//! - `--pivot-values nopol`
//! - `--pivot-agg count-distinct`
//! - `--keep first`
//! - `--out $HOME/Downloads`

#![allow(clippy::wildcard_imports)]

use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use librcekunit_examples::auth;
use librcekunit_examples::csv_check;
use librcekunit_examples::error;
use librcekunit_examples::prelude::*;

/// Boxed, thread-safe error type used as the return of `main`.
type DynError = Box<dyn core::error::Error + Send + Sync>;

/// Which occurrence to keep when deduplicating.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Keep {
    /// Keep the first occurrence.
    First,
    /// Keep the last occurrence.
    Last,
}

impl Keep {
    /// Parse a `--keep` value.
    fn parse(s: &str) -> Result<Self, DynError> {
        match s {
            "first" => Ok(Self::First),
            "last" => Ok(Self::Last),
            other => Err(format!("--keep must be first or last, got {other:?}").into()),
        }
    }

    /// Human label.
    const fn label(self) -> &'static str {
        match self {
            Self::First => "first",
            Self::Last => "last",
        }
    }
}

/// Aggregation for the pivot step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Agg {
    /// Row count per group.
    Count,
    /// Distinct values per group.
    CountDistinct,
    /// Sum of the value column.
    Sum,
}

impl Agg {
    /// Parse an `--pivot-agg` value.
    fn parse(s: &str) -> Result<Self, DynError> {
        match s {
            "count" => Ok(Self::Count),
            "count-distinct" => Ok(Self::CountDistinct),
            "sum" => Ok(Self::Sum),
            other => Err(format!(
                "--pivot-agg must be count, count-distinct, or sum, got {other:?}"
            )
            .into()),
        }
    }

    /// Column header suffix.
    const fn header_suffix(self) -> &'static str {
        match self {
            Self::Count => "count",
            Self::CountDistinct => "count_distinct",
            Self::Sum => "sum",
        }
    }
}

/// Which date range to request.
#[derive(Debug, Clone)]
enum DateSpec {
    /// Today's local date.
    Today,
    /// A single date.
    Single(String),
    /// Inclusive range.
    Range { start: String, end: String },
}

impl DateSpec {
    /// Return the `(start, end, label)` triple for this spec.
    ///
    /// # Errors
    ///
    /// Returns an error when the `date` command cannot run.
    fn resolve(&self) -> Result<(String, String, String), DynError> {
        match self {
            Self::Today => {
                let today = today_local()?;
                validate_date(&today)?;
                Ok((today.clone(), today.clone(), format!("today_{today}")))
            }
            Self::Single(d) => {
                validate_date(d)?;
                Ok((d.clone(), d.clone(), d.clone()))
            }
            Self::Range { start, end } => {
                validate_date(start)?;
                validate_date(end)?;
                if start > end {
                    return Err(format!("start {start} is after end {end}").into());
                }
                Ok((start.clone(), end.clone(), format!("{start}_to_{end}")))
            }
        }
    }
}

/// Accumulator for one group during a pivot.
#[derive(Debug, Default)]
struct Bucket {
    /// Number of rows in the group.
    count: u64,
    /// Distinct values in the group.
    distinct: HashSet<String>,
    /// Sum of the parsed value column.
    sum: f64,
}

/// Command-line options.
#[derive(Debug, Clone)]
struct Options {
    /// Date range to request.
    date: DateSpec,
    /// Dedupe key columns.
    dedup_by: Vec<String>,
    /// Which occurrence to keep.
    keep: Keep,
    /// Pivot row columns.
    pivot_rows: Vec<String>,
    /// Pivot value column.
    pivot_values: String,
    /// Pivot aggregation.
    pivot_agg: Agg,
    /// Output directory.
    out_dir: Option<PathBuf>,
    /// Overwrite existing files.
    force: bool,
}

impl Options {
    /// Parse arguments.
    fn from_args() -> Result<Self, DynError> {
        let args: Vec<String> = std::env::args().collect();
        let mut iter = args.iter().skip(1);

        let subcommand = iter
            .next()
            .ok_or("missing subcommand: today | date <YYYY-MM-DD> | range <START> <END>")?;

        let date = match subcommand.as_str() {
            "today" => DateSpec::Today,
            "date" => {
                let d = iter.next().ok_or("date requires a YYYY-MM-DD argument")?;
                DateSpec::Single(d.clone())
            }
            "range" => {
                let s = iter.next().ok_or("range requires a START argument")?;
                let e = iter.next().ok_or("range requires an END argument")?;
                DateSpec::Range {
                    start: s.clone(),
                    end: e.clone(),
                }
            }
            other => {
                return Err(
                    format!("unknown subcommand: {other}. expected today | date | range").into(),
                );
            }
        };

        let mut dedup_by = vec![String::from("user_id"), String::from("nopol")];
        let mut keep = Keep::First;
        let mut pivot_rows = vec![String::from("nama")];
        let mut pivot_values = String::from("nopol");
        let mut pivot_agg = Agg::CountDistinct;
        let mut out_dir: Option<PathBuf> = None;
        let mut force = false;

        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--dedup-by" => {
                    if let Some(v) = iter.next() {
                        dedup_by = split_columns(v);
                    }
                }
                "--keep" => {
                    if let Some(v) = iter.next() {
                        keep = Keep::parse(v)?;
                    }
                }
                "--pivot-rows" => {
                    if let Some(v) = iter.next() {
                        pivot_rows = split_columns(v);
                    }
                }
                "--pivot-values" => {
                    if let Some(v) = iter.next() {
                        pivot_values.clone_from(v);
                    }
                }
                "--pivot-agg" => {
                    if let Some(v) = iter.next() {
                        pivot_agg = Agg::parse(v)?;
                    }
                }
                "--out" => {
                    if let Some(v) = iter.next() {
                        out_dir = Some(PathBuf::from(v));
                    }
                }
                "--force" => force = true,
                _ => {}
            }
        }

        if dedup_by.is_empty() {
            return Err("--dedup-by must contain at least one column".into());
        }
        if pivot_rows.is_empty() {
            return Err("--pivot-rows must contain at least one column".into());
        }

        Ok(Self {
            date,
            dedup_by,
            keep,
            pivot_rows,
            pivot_values,
            pivot_agg,
            out_dir,
            force,
        })
    }
}

/// Split a comma-separated column list.
fn split_columns(s: &str) -> Vec<String> {
    s.split(',')
        .map(|p| p.trim().to_owned())
        .filter(|p| !p.is_empty())
        .collect()
}

/// Entry point.
#[tokio::main]
async fn main() -> Result<(), DynError> {
    let _ = librcekunit_client::tracing::init_tracing();

    let opts = Options::from_args()?;
    print_banner(&opts);

    // Resolve the date range.
    let (start_date, end_date, label) = opts.date.resolve()?;
    println!("[date] range: {start_date} .. {end_date}");
    println!("[date] label: {label}");

    // Resolve output directory.
    let out_dir = match &opts.out_dir {
        Some(d) => d.clone(),
        None => default_downloads_dir()?,
    };
    if !out_dir.exists() {
        std::fs::create_dir_all(&out_dir)
            .map_err(|e| -> DynError { format!("create {}: {e}", out_dir.display()).into() })?;
        println!("[fs] created directory {}", out_dir.display());
    }

    // Resolve file paths.
    let raw_path = out_dir.join(format!("cekunit_{label}.raw.csv"));
    let deduped_path = out_dir.join(format!("cekunit_{label}.deduped.csv"));
    let report_path = out_dir.join(format!("cekunit_{label}.report.csv"));

    // Refuse to overwrite unless forced.
    for path in [&raw_path, &deduped_path, &report_path] {
        if path.exists() && !opts.force {
            return Err(format!(
                "output already exists: {}. pass --force to overwrite",
                path.display()
            )
            .into());
        }
    }

    // Step 1: download.
    println!("[step 1] downloading to {}", raw_path.display());
    let raw_bytes = download(&start_date, &end_date).await?;
    std::fs::write(&raw_path, &raw_bytes)
        .map_err(|e| -> DynError { format!("write {}: {e}", raw_path.display()).into() })?;
    println!("[step 1] wrote {} bytes", raw_bytes.len());

    // Step 2: dedupe.
    println!(
        "[step 2] deduping by [{}] keep={}",
        opts.dedup_by.join(", "),
        opts.keep.label()
    );
    let raw_text = String::from_utf8_lossy(&raw_bytes);
    let deduped_csv = dedupe_csv(&raw_text, &opts.dedup_by, opts.keep)?;
    std::fs::write(&deduped_path, deduped_csv.as_bytes())
        .map_err(|e| -> DynError { format!("write {}: {e}", deduped_path.display()).into() })?;
    println!(
        "[step 2] wrote {} bytes to {}",
        deduped_csv.len(),
        deduped_path.display()
    );

    // Step 3: pivot.
    println!(
        "[step 3] pivoting rows=[{}] values={} agg={:?}",
        opts.pivot_rows.join(", "),
        opts.pivot_values,
        opts.pivot_agg
    );
    let report_csv = pivot_csv(
        &deduped_csv,
        &opts.pivot_rows,
        &opts.pivot_values,
        opts.pivot_agg,
    )?;
    std::fs::write(&report_path, report_csv.as_bytes())
        .map_err(|e| -> DynError { format!("write {}: {e}", report_path.display()).into() })?;
    println!(
        "[step 3] wrote {} bytes to {}",
        report_csv.len(),
        report_path.display()
    );

    // Summary.
    println!();
    println!("============================================================");
    println!("FILES WRITTEN");
    println!("============================================================");
    println!("raw:       {}", raw_path.display());
    println!("deduped:   {}", deduped_path.display());
    println!("report:    {}", report_path.display());
    println!("============================================================");

    println!("[preview] report head:");
    for line in report_csv.lines().take(11) {
        println!("[preview]   {line}");
    }

    println!("done");
    Ok(())
}

/// Print a banner.
fn print_banner(opts: &Options) {
    println!("============================================================");
    println!("FULL REPORT: download, dedupe, pivot");
    println!("============================================================");
    println!("dedup by:     {}", opts.dedup_by.join(", "));
    println!("keep:         {}", opts.keep.label());
    println!("pivot rows:   {}", opts.pivot_rows.join(", "));
    println!("pivot values: {}", opts.pivot_values);
    println!("pivot agg:    {:?}", opts.pivot_agg);
    match &opts.out_dir {
        Some(p) => println!("out dir:      {}", p.display()),
        None => println!("out dir:      ~/Downloads"),
    }
    println!("force:        {}", opts.force);
    println!("============================================================");
}

/// Download the CSV from the server, streaming the body.
///
/// The server builds the entire CSV in memory before responding and
/// uses `Transfer-Encoding: chunked`, so there is no content length.
/// We stream the body in chunks so large downloads do not spike
/// memory and progress is visible.
///
/// # Errors
///
/// Returns an error when the client cannot be built, the session cannot
/// be established, the request fails, or the body cannot be read.
async fn download(start_date: &str, end_date: &str) -> Result<Vec<u8>, DynError> {
    let client = match from_env().await {
        Ok(c) => c,
        Err(err) => {
            eprintln!("[config] {}", error::user_message(&err));
            return Err(err.into());
        }
    };

    // Ensure session.
    let probe = client.dashboard_index().await?;
    if auth::is_authenticated_status(probe.status().as_u16()) {
        println!("[session] reused existing session");
    } else {
        println!("[session] logging in");
        login_from_env(&client).await?;
        println!("[session] login ok");
    }

    let mut params = Form::new();
    let _ = params.insert(String::from("format"), String::from("csv"));
    let _ = params.insert(String::from("sort"), String::from("created_at"));
    let _ = params.insert(String::from("direction"), String::from("asc"));
    let _ = params.insert(String::from("start_date"), start_date.to_owned());
    let _ = params.insert(String::from("end_date"), end_date.to_owned());
    let _ = params.insert(String::from("search"), String::new());

    println!("[download] requesting {start_date} .. {end_date}");

    let mut resp = client.input_user_export(params).await?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        let preview_len = body.len().min(300);
        let preview = body.get(..preview_len).unwrap_or(&body);
        return Err(format!("server returned {status}; preview: {preview}").into());
    }

    let content_length = resp
        .content_length()
        .map_or_else(|| String::from("unknown (chunked)"), |n| n.to_string());
    println!("[download] status={status} content_length={content_length}");

    // Stream the body chunk by chunk.
    let mut bytes: Vec<u8> = Vec::new();
    let mut chunks: u64 = 0;
    let mut last_mb: usize = 0;

    while let Some(chunk) = resp
        .chunk()
        .await
        .map_err(|e| -> DynError { format!("read chunk: {e}").into() })?
    {
        bytes.extend_from_slice(&chunk);
        chunks = chunks.saturating_add(1);

        let mb = bytes.len() / (1024 * 1024);
        if mb > last_mb {
            last_mb = mb;
            println!("[download] {mb} MiB ({} bytes)", bytes.len());
        }
    }

    println!(
        "[download] done: {} bytes in {} chunks",
        bytes.len(),
        chunks
    );

    if bytes.is_empty() {
        return Err("server returned an empty body".into());
    }

    Ok(bytes)
}

/// Deduplicate a CSV text and return the result as CSV text.
///
/// # Errors
///
/// Returns an error when the CSV is malformed or a column is missing.
fn dedupe_csv(text: &str, by: &[String], keep: Keep) -> Result<String, DynError> {
    let mut records = csv_check::parse_records(text)?;
    if records.is_empty() {
        return Err("input CSV is empty".into());
    }

    let header = records.remove(0);
    let key_indices = resolve_indices(&header, by)?;
    println!("[step 2] key indices: {key_indices:?}");

    let input_rows = records.len();
    let mut index_of: BTreeMap<Vec<String>, usize> = BTreeMap::new();
    let mut out: Vec<Vec<String>> = Vec::new();

    for row in records {
        if row.len() == 1 && row.first().is_some_and(String::is_empty) {
            continue;
        }
        let key: Vec<String> = key_indices
            .iter()
            .map(|&i| row.get(i).cloned().unwrap_or_default())
            .collect();

        match keep {
            Keep::First => {
                if let Entry::Vacant(e) = index_of.entry(key) {
                    let _ = e.insert(out.len());
                    out.push(row);
                }
            }
            Keep::Last => {
                if let Some(&prev) = index_of.get(&key) {
                    if let Some(slot) = out.get_mut(prev) {
                        slot.clone_from(&row);
                    }
                } else {
                    let _ = index_of.insert(key, out.len());
                    out.push(row);
                }
            }
        }
    }

    println!(
        "[step 2] {input_rows} input rows -> {} unique rows",
        out.len()
    );

    let mut result = Vec::with_capacity(out.len().saturating_add(1));
    result.push(header);
    result.extend(out);
    Ok(records_to_csv(&result))
}

/// Pivot a CSV text and return the result as CSV text.
///
/// # Errors
///
/// Returns an error when the CSV is malformed or a column is missing.
fn pivot_csv(text: &str, rows: &[String], values: &str, agg: Agg) -> Result<String, DynError> {
    let mut records = csv_check::parse_records(text)?;
    if records.is_empty() {
        return Err("input CSV is empty".into());
    }

    let header = records.remove(0);
    let row_indices = resolve_indices(&header, rows)?;
    let value_index = resolve_single_index(&header, values)?;

    let mut buckets: BTreeMap<Vec<String>, Bucket> = BTreeMap::new();

    for row in records {
        if row.len() == 1 && row.first().is_some_and(String::is_empty) {
            continue;
        }
        let key: Vec<String> = row_indices
            .iter()
            .map(|&i| row.get(i).cloned().unwrap_or_default())
            .collect();
        let value = row.get(value_index).cloned().unwrap_or_default();

        let bucket = buckets.entry(key).or_default();
        bucket.count = bucket.count.saturating_add(1);
        match agg {
            Agg::Count => {}
            Agg::CountDistinct => {
                let _ = bucket.distinct.insert(value);
            }
            Agg::Sum => {
                if let Ok(n) = value.trim().parse::<f64>() {
                    bucket.sum += n;
                }
            }
        }
    }

    let mut rows_out: Vec<(Vec<String>, f64)> = buckets
        .into_iter()
        .map(|(key, b)| {
            let value = match agg {
                Agg::Count => {
                    #[allow(clippy::cast_precision_loss)]
                    {
                        b.count as f64
                    }
                }
                Agg::CountDistinct => {
                    #[allow(clippy::cast_precision_loss)]
                    {
                        b.distinct.len() as f64
                    }
                }
                Agg::Sum => b.sum,
            };
            (key, value)
        })
        .collect();

    rows_out.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(core::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });

    let mut out = String::new();
    let mut header_parts: Vec<String> = rows.to_vec();
    header_parts.push(format!("{}_{}", agg.header_suffix(), values));
    out.push_str(
        &header_parts
            .iter()
            .map(|h| escape_field(h))
            .collect::<Vec<_>>()
            .join(","),
    );
    out.push('\n');

    for (key, value) in rows_out {
        let mut parts = key;
        parts.push(fmt_value(value));
        out.push_str(
            &parts
                .iter()
                .map(|f| escape_field(f))
                .collect::<Vec<_>>()
                .join(","),
        );
        out.push('\n');
    }

    Ok(out)
}

/// Resolve a list of column names to indices.
///
/// # Errors
///
/// Returns an error when a column is missing.
fn resolve_indices(header: &[String], names: &[String]) -> Result<Vec<usize>, DynError> {
    let mut indices = Vec::with_capacity(names.len());
    for name in names {
        if let Some(i) = header.iter().position(|h| h == name) {
            indices.push(i);
        } else {
            eprintln!("[error] column not found: {name}");
            eprintln!("[error] available columns: {}", header.join(", "));
            return Err(format!("column not found: {name}").into());
        }
    }
    Ok(indices)
}

/// Resolve one column name to an index.
///
/// # Errors
///
/// Returns an error when the column is missing.
fn resolve_single_index(header: &[String], name: &str) -> Result<usize, DynError> {
    header.iter().position(|h| h == name).ok_or_else(|| {
        eprintln!("[error] column not found: {name}");
        eprintln!("[error] available columns: {}", header.join(", "));
        format!("column not found: {name}").into()
    })
}

/// Serialize records to CSV text.
fn records_to_csv(records: &[Vec<String>]) -> String {
    let mut out = String::new();
    for record in records {
        let line = record
            .iter()
            .map(|f| escape_field(f))
            .collect::<Vec<_>>()
            .join(",");
        out.push_str(&line);
        out.push('\n');
    }
    out
}

/// Format a numeric value: integer if integral, decimal otherwise.
fn fmt_value(v: f64) -> String {
    #[allow(clippy::float_cmp)]
    if v.fract() == 0.0 {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let i = v as i64;
        i.to_string()
    } else {
        format!("{v}")
    }
}

/// Escape a single CSV field.
fn escape_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') || field.contains('\r') {
        let escaped = field.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        field.to_owned()
    }
}

/// Default downloads directory: `$HOME/Downloads`.
///
/// # Errors
///
/// Returns an error when `$HOME` is not set.
fn default_downloads_dir() -> Result<PathBuf, DynError> {
    let home = std::env::var("HOME").map_err(|e| -> DynError { format!("$HOME: {e}").into() })?;
    Ok(Path::new(&home).join("Downloads"))
}

/// Today's local date, `YYYY-MM-DD`.
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
