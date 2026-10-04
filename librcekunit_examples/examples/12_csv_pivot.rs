#![allow(unused_crate_dependencies)]

//! # Pivot a CSV: group by one column, aggregate another
//!
//! Reads a CSV, groups rows by one or more key columns, and produces
//! a summary table. This is the command-line equivalent of Excel's
//! PivotTable: pick a row label, pick a value column, choose an
//! aggregation.
//!
//! ## Use case
//!
//! After deduplicating an export (example 11), you want to know how
//! many unique vehicles each field officer checked:
//!
//! ```sh
//! cargo run --example 12_csv_pivot -- \
//!     input.deduped.csv \
//!     --rows nama \
//!     --values nopol \
//!     --agg count-distinct
//! ```
//!
//! Output:
//!
//! ```csv
//! nama,count_distinct_nopol
//! Marudut horas Pardomuan Silalahi,15
//! Muhammad farhan,12
//! FIDELIS ABDI CHRISTIAN NAIBAHO,8
//! ```
//!
//! ## Aggregations
//!
//! | Value              | Meaning                                            |
//! |--------------------|----------------------------------------------------|
//! | `count`            | Number of rows per group                           |
//! | `count-distinct`   | Number of unique values in `--values` per group    |
//! | `sum`              | Sum of `--values` parsed as a number per group     |
//!
//! ## Safety
//!
//! - Read-only against the input file.
//! - Refuses to overwrite the output unless `--force` is passed.
//! - All processing is local. No network.
//!
//! ## Usage
//!
//! ```sh
//! # Simple row count per officer
//! cargo run --example 12_csv_pivot -- input.csv --rows nama --values nopol --agg count
//!
//! # Unique vehicles per officer
//! cargo run --example 12_csv_pivot -- input.csv --rows nama --values nopol --agg count-distinct
//!
//! # Group by two columns
//! cargo run --example 12_csv_pivot -- input.csv --rows user_id,nama --values nopol
//!
//! # Custom output
//! cargo run --example 12_csv_pivot -- input.csv --rows nama --values nopol --out pivot.csv
//! ```

#![allow(clippy::wildcard_imports)]
#![allow(clippy::std_instead_of_alloc)]

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use librcekunit_examples::csv_check;

/// Boxed, thread-safe error type used as the return of `main`.
type DynError = Box<dyn core::error::Error + Send + Sync>;

/// Aggregation to apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Agg {
    /// Number of rows in the group.
    Count,
    /// Number of distinct values in the value column.
    CountDistinct,
    /// Sum of the value column parsed as f64.
    Sum,
}

impl Agg {
    /// Parse `--agg`.
    fn parse(s: &str) -> Result<Self, DynError> {
        match s {
            "count" => Ok(Self::Count),
            "count-distinct" => Ok(Self::CountDistinct),
            "sum" => Ok(Self::Sum),
            other => {
                Err(format!("--agg must be count, count-distinct, or sum, got {other:?}").into())
            }
        }
    }

    /// Column header suffix for the output.
    const fn header_suffix(self) -> &'static str {
        match self {
            Self::Count => "count",
            Self::CountDistinct => "count_distinct",
            Self::Sum => "sum",
        }
    }
}

/// One group's accumulated values.
#[derive(Debug, Default)]
struct Bucket {
    /// Number of rows seen.
    row_count: u64,
    /// Distinct values observed (only used by `CountDistinct`).
    distinct: HashSet<String>,
    /// Sum of parsed numbers (only used by `Sum`).
    sum: f64,
    /// Non-numeric values encountered during a `Sum` aggregation.
    non_numeric: u64,
}

/// Command-line options.
#[derive(Debug, Clone)]
struct Options {
    /// Input CSV path.
    input: PathBuf,
    /// Columns used as the row label.
    rows: Vec<String>,
    /// Column used as the aggregated value.
    values: String,
    /// Aggregation mode.
    agg: Agg,
    /// Output path. `None` means derive from input.
    output: Option<PathBuf>,
    /// Overwrite the output if it exists.
    force: bool,
}

impl Options {
    /// Parse command-line arguments.
    fn from_args() -> Result<Self, DynError> {
        let args: Vec<String> = std::env::args().collect();
        let mut iter = args.iter().skip(1);

        let input = iter
            .next()
            .ok_or("missing input file. usage: <input.csv> --rows col --values col")?;
        let input = PathBuf::from(input);

        let mut rows_raw: Option<String> = None;
        let mut values: Option<String> = None;
        let mut agg = Agg::CountDistinct;
        let mut output: Option<PathBuf> = None;
        let mut force = false;

        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--rows" => {
                    if let Some(v) = iter.next() {
                        rows_raw = Some(v.clone());
                    }
                }
                "--values" => {
                    if let Some(v) = iter.next() {
                        values = Some(v.clone());
                    }
                }
                "--agg" => {
                    if let Some(v) = iter.next() {
                        agg = Agg::parse(v)?;
                    }
                }
                "--out" => {
                    if let Some(v) = iter.next() {
                        output = Some(PathBuf::from(v));
                    }
                }
                "--force" => force = true,
                _ => {}
            }
        }

        let rows_raw = rows_raw.ok_or("missing --rows <col1[,col2,...]>")?;
        let rows: Vec<String> = rows_raw
            .split(',')
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .collect();

        if rows.is_empty() {
            return Err("--rows must contain at least one column name".into());
        }

        let values = values.ok_or("missing --values <col>")?;

        Ok(Self {
            input,
            rows,
            values,
            agg,
            output,
            force,
        })
    }
}

/// A single output row: the group key plus its aggregated value.
#[derive(Debug)]
struct PivotRow {
    /// Key parts, one per `--rows` column, in order.
    key: Vec<String>,
    /// Aggregated value.
    value: f64,
}

/// Entry point.
fn main() -> Result<(), DynError> {
    let opts = Options::from_args()?;
    print_banner(&opts);

    // Step 1: read and validate.
    println!("[step 1] reading {}", opts.input.display());
    let text = std::fs::read_to_string(&opts.input)
        .map_err(|e| -> DynError { format!("read {}: {e}", opts.input.display()).into() })?;

    let mut records = csv_check::parse_records(&text)?;
    if records.is_empty() {
        return Err("file contains no records".into());
    }

    let header = records.remove(0);
    println!("[step 1] header has {} columns", header.len());
    for (i, col) in header.iter().enumerate() {
        println!("[step 1]   {i}: {col}");
    }

    // Step 2: resolve column indices.
    let row_indices = resolve_indices(&header, &opts.rows)?;
    let value_index = resolve_single_index(&header, &opts.values)?;
    println!(
        "[step 2] rows: [{}] -> indices {:?}",
        opts.rows.join(", "),
        row_indices
    );
    println!("[step 2] values: {} -> index {}", opts.values, value_index);
    println!("[step 2] aggregation: {:?}", opts.agg);

    // Step 3: aggregate.
    println!("[step 3] aggregating {} data rows", records.len());
    let (buckets, non_numeric_total) = aggregate(&records, &row_indices, value_index, opts.agg);
    println!("[step 3] {} distinct groups", buckets.len());
    if non_numeric_total > 0 {
        println!(
            "[step 3] {} non-numeric values in {} (used 0 for sum)",
            non_numeric_total, opts.values
        );
    }

    // Step 4: turn buckets into sorted output rows.
    let mut pivot_rows = finalize(buckets, opts.agg);
    pivot_rows.sort_by(|a, b| {
        b.value
            .partial_cmp(&a.value)
            .unwrap_or(core::cmp::Ordering::Equal)
            .then_with(|| a.key.cmp(&b.key))
    });
    println!(
        "[step 4] sorted {} rows by value (descending)",
        pivot_rows.len()
    );

    // Step 5: resolve output path.
    let output = match &opts.output {
        Some(p) => p.clone(),
        None => default_output_path(&opts.input),
    };
    println!("[step 5] output: {}", output.display());

    if output.exists() && !opts.force {
        return Err(format!(
            "output already exists: {}. pass --force to overwrite",
            output.display()
        )
        .into());
    }

    // Step 6: write.
    let csv = render_csv(&opts.rows, opts.agg, &opts.values, &pivot_rows);
    std::fs::write(&output, csv.as_bytes())
        .map_err(|e| -> DynError { format!("write {}: {e}", output.display()).into() })?;
    println!(
        "[step 6] wrote {} rows ({} bytes) to {}",
        pivot_rows.len().saturating_add(1),
        csv.len(),
        output.display()
    );

    // Step 7: preview.
    println!("[preview] top 10:");
    for row in pivot_rows.iter().take(10) {
        println!(
            "[preview]   {} = {}",
            row.key.join(" | "),
            fmt_value(row.value)
        );
    }

    println!("done");
    Ok(())
}

/// Print a banner with the run configuration.
fn print_banner(opts: &Options) {
    println!("============================================================");
    println!("CSV pivot");
    println!("============================================================");
    println!("input:   {}", opts.input.display());
    println!("rows:    {}", opts.rows.join(", "));
    println!("values:  {}", opts.values);
    println!("agg:     {:?}", opts.agg);
    match &opts.output {
        Some(p) => println!("output:  {}", p.display()),
        None => println!("output:  (derived from input path)"),
    }
    println!("force:   {}", opts.force);
    println!("============================================================");
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

/// Aggregate rows into buckets keyed by the row columns.
///
/// Returns the bucket map and the total number of non-numeric values
/// seen when the aggregation is `Sum`.
fn aggregate(
    rows: &[Vec<String>],
    row_indices: &[usize],
    value_index: usize,
    agg: Agg,
) -> (BTreeMap<Vec<String>, Bucket>, u64) {
    let mut buckets: BTreeMap<Vec<String>, Bucket> = BTreeMap::new();
    let mut non_numeric_total: u64 = 0;

    for row in rows {
        // Skip empty trailing rows.
        if row.len() == 1 && row.first().is_some_and(String::is_empty) {
            continue;
        }

        let key: Vec<String> = row_indices
            .iter()
            .map(|&i| row.get(i).cloned().unwrap_or_default())
            .collect();

        let value = row.get(value_index).cloned().unwrap_or_default();

        let bucket = buckets.entry(key).or_default();
        bucket.row_count = bucket.row_count.saturating_add(1);

        match agg {
            Agg::Count => {}
            Agg::CountDistinct => {
                let _ = bucket.distinct.insert(value);
            }
            Agg::Sum => {
                if let Ok(n) = value.trim().parse::<f64>() {
                    bucket.sum += n;
                } else {
                    bucket.non_numeric = bucket.non_numeric.saturating_add(1);
                    non_numeric_total = non_numeric_total.saturating_add(1);
                }
            }
        }
    }

    (buckets, non_numeric_total)
}

/// Turn the bucket map into sorted pivot rows.
fn finalize(buckets: BTreeMap<Vec<String>, Bucket>, agg: Agg) -> Vec<PivotRow> {
    buckets
        .into_iter()
        .map(|(key, bucket)| {
            let value = match agg {
                Agg::Count => {
                    #[allow(clippy::cast_precision_loss)]
                    {
                        bucket.row_count as f64
                    }
                }
                Agg::CountDistinct => {
                    #[allow(clippy::cast_precision_loss)]
                    {
                        bucket.distinct.len() as f64
                    }
                }
                Agg::Sum => bucket.sum,
            };
            PivotRow { key, value }
        })
        .collect()
}

/// Render the pivot table as CSV.
fn render_csv(row_names: &[String], agg: Agg, value_col: &str, rows: &[PivotRow]) -> String {
    let mut out = String::new();

    // Header.
    let mut header_parts: Vec<String> = row_names.to_vec();
    let value_header = format!("{}_{}", agg.header_suffix(), value_col);
    header_parts.push(value_header);
    out.push_str(
        &header_parts
            .iter()
            .map(|h| escape_field(h))
            .collect::<Vec<_>>()
            .join(","),
    );
    out.push('\n');

    // Rows.
    for row in rows {
        let mut parts: Vec<String> = row.key.clone();
        parts.push(fmt_value(row.value));
        out.push_str(
            &parts
                .iter()
                .map(|f| escape_field(f))
                .collect::<Vec<_>>()
                .join(","),
        );
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

/// Derive an output path from the input path:
/// `input.csv` -> `input.pivot.csv`.
fn default_output_path(input: &Path) -> PathBuf {
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let filename = format!("{stem}.pivot.csv");
    match input.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir.join(filename),
        _ => PathBuf::from(filename),
    }
}
