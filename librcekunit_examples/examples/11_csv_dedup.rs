#![allow(unused_crate_dependencies)]

//! # Remove duplicate rows from a CSV
//!
//! Reads a CSV file, deduplicates rows based on one or more chosen
//! columns, and writes the result. This is the command-line equivalent
//! of Excel's "Remove Duplicates" feature, but scriptable.
//!
//! ## Use case
//!
//! The CSV downloaded from the input-user page has one row per
//! checking event, so the same `user_id` + `nopol` pair can appear
//! multiple times. To get one row per vehicle per officer:
//!
//! ```sh
//! cargo run --example 11_csv_dedup -- \
//!     input.csv --by user_id,nopol
//! ```
//!
//! ## Picking which row to keep
//!
//! By default the first occurrence wins. Use `--keep last` to keep the
//! last instead, which is useful when the file is sorted oldest-first
//! and the newest row has the data you want.
//!
//! ## Safety
//!
//! - Never overwrites the output file unless `--force` is passed.
//! - Never writes anything until validation succeeds.
//! - All processing is local. No network.
//!
//! ## Usage
//!
//! ```sh
//! # Deduplicate by two columns
//! cargo run --example 11_csv_dedup -- input.csv --by user_id,nopol
//!
//! # Keep the last occurrence instead of the first
//! cargo run --example 11_csv_dedup -- input.csv --by user_id,nopol --keep last
//!
//! # Custom output path
//! cargo run --example 11_csv_dedup -- input.csv --by nopol --out clean.csv
//!
//! # Overwrite output if it exists
//! cargo run --example 11_csv_dedup -- input.csv --by nopol --force
//! ```

#![allow(clippy::wildcard_imports)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use librcekunit_examples::csv_check;

/// Boxed, thread-safe error type used as the return of `main`.
type DynError = Box<dyn core::error::Error + Send + Sync>;

/// Which occurrence to keep when a duplicate is found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Keep {
    /// Keep the first occurrence.
    First,
    /// Keep the last occurrence.
    Last,
}

impl Keep {
    /// Parse the `--keep` value.
    fn parse(s: &str) -> Result<Self, DynError> {
        match s {
            "first" => Ok(Self::First),
            "last" => Ok(Self::Last),
            other => Err(format!("--keep must be 'first' or 'last', got {other:?}").into()),
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

/// Command-line options.
#[derive(Debug, Clone)]
struct Options {
    /// Input CSV path.
    input: PathBuf,
    /// Comma-separated column names used as the dedupe key.
    by: Vec<String>,
    /// Which occurrence to keep.
    keep: Keep,
    /// Output path. `None` means derive from input.
    output: Option<PathBuf>,
    /// Overwrite output if it exists.
    force: bool,
}

impl Options {
    /// Parse command-line arguments.
    fn from_args() -> Result<Self, DynError> {
        let args: Vec<String> = std::env::args().collect();
        let mut iter = args.iter().skip(1);

        let input = iter
            .next()
            .ok_or("missing input file. usage: <input.csv> --by col1,col2")?;
        let input = PathBuf::from(input);

        let mut by_raw: Option<String> = None;
        let mut keep = Keep::First;
        let mut output: Option<PathBuf> = None;
        let mut force = false;

        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--by" => {
                    if let Some(v) = iter.next() {
                        by_raw = Some(v.clone());
                    }
                }
                "--keep" => {
                    if let Some(v) = iter.next() {
                        keep = Keep::parse(v)?;
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

        let by_raw = by_raw.ok_or("missing --by <col1,col2>")?;
        let by: Vec<String> = by_raw
            .split(',')
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .collect();

        if by.is_empty() {
            return Err("--by must contain at least one column name".into());
        }

        Ok(Self {
            input,
            by,
            keep,
            output,
            force,
        })
    }
}

/// Statistics produced by a dedupe run.
#[derive(Debug, Clone, Copy)]
struct Stats {
    /// Rows in the input (excluding the header).
    input_rows: usize,
    /// Rows in the output (excluding the header).
    output_rows: usize,
    /// Rows removed.
    removed: usize,
    /// Number of distinct keys that appeared more than once.
    groups_with_duplicates: usize,
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

    // Step 2: locate the key columns.
    let key_indices = resolve_key_indices(&header, &opts.by)?;
    println!(
        "[step 2] dedupe key: [{}] -> indices {:?}",
        opts.by.join(", "),
        key_indices
    );

    // Step 3: dedupe.
    println!("[step 3] processing {} data rows", records.len());
    let (deduped, stats) = dedupe(&records, &key_indices, opts.keep);
    print_stats(&stats, opts.keep);

    // Step 4: resolve output path.
    let output = match &opts.output {
        Some(p) => p.clone(),
        None => default_output_path(&opts.input),
    };
    println!("[step 4] output: {}", output.display());

    if output.exists() && !opts.force {
        return Err(format!(
            "output already exists: {}. pass --force to overwrite",
            output.display()
        )
        .into());
    }

    // Step 5: write.
    let mut out_records = Vec::with_capacity(deduped.len().saturating_add(1));
    out_records.push(header);
    out_records.extend(deduped);

    let csv = records_to_csv(&out_records);
    std::fs::write(&output, csv.as_bytes())
        .map_err(|e| -> DynError { format!("write {}: {e}", output.display()).into() })?;
    println!(
        "[step 5] wrote {} rows ({} bytes) to {}",
        out_records.len(),
        csv.len(),
        output.display()
    );

    println!("done");
    Ok(())
}

/// Print a banner with the run configuration.
fn print_banner(opts: &Options) {
    println!("============================================================");
    println!("CSV dedupe");
    println!("============================================================");
    println!("input:  {}", opts.input.display());
    println!("by:     {}", opts.by.join(", "));
    println!("keep:   {}", opts.keep.label());
    match &opts.output {
        Some(p) => println!("output: {}", p.display()),
        None => println!("output: (derived from input path)"),
    }
    println!("force:  {}", opts.force);
    println!("============================================================");
}

/// Resolve the `--by` column names to indices in the header.
///
/// # Errors
///
/// Returns an error when a column name is not present. The error
/// message lists all available columns.
fn resolve_key_indices(header: &[String], by: &[String]) -> Result<Vec<usize>, DynError> {
    let mut indices = Vec::with_capacity(by.len());
    for name in by {
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

/// Deduplicate rows by the given key columns.
///
/// The key is built by joining the values at `key_indices` for each
/// row. Rows with an identical key are collapsed to one, keeping either
/// the first or the last occurrence depending on `keep`.
fn dedupe(rows: &[Vec<String>], key_indices: &[usize], keep: Keep) -> (Vec<Vec<String>>, Stats) {
    let mut index_of: HashMap<Vec<String>, usize> = HashMap::new();
    let mut count_of: HashMap<Vec<String>, usize> = HashMap::new();
    let mut out: Vec<Vec<String>> = Vec::new();

    for row in rows {
        // Skip empty rows.
        if row.len() == 1 && row.first().is_some_and(String::is_empty) {
            continue;
        }

        let key: Vec<String> = key_indices
            .iter()
            .map(|&i| row.get(i).cloned().unwrap_or_default())
            .collect();

        let counter = count_of.entry(key.clone()).or_insert(0);
        *counter = counter.saturating_add(1);

        match keep {
            Keep::First => {
                if *counter == 1 {
                    let _ = index_of.insert(key, out.len());
                    out.push(row.clone());
                }
                // Otherwise it is a duplicate; skip.
            }
            Keep::Last => {
                if let Some(&prev) = index_of.get(&key)
                    && let Some(slot) = out.get_mut(prev)
                {
                    slot.clone_from(row);
                } else {
                    let _ = index_of.insert(key, out.len());
                    out.push(row.clone());
                }
            }
        }
    }

    let input_rows = rows.len();
    let output_rows = out.len();
    let removed = input_rows.saturating_sub(output_rows);
    let groups_with_duplicates = count_of.values().filter(|&&c| c > 1).count();

    (
        out,
        Stats {
            input_rows,
            output_rows,
            removed,
            groups_with_duplicates,
        },
    )
}
/// Print the dedupe statistics.
fn print_stats(stats: &Stats, keep: Keep) {
    println!("[stats] input rows:            {}", stats.input_rows);
    println!("[stats] output rows:           {}", stats.output_rows);
    println!("[stats] removed:               {}", stats.removed);
    println!(
        "[stats] duplicate groups:      {}",
        stats.groups_with_duplicates
    );
    println!("[stats] kept:                  {}", keep.label());
}

/// Derive an output path from the input path by inserting `.deduped`
/// before the extension.
///
/// `input.csv` -> `input.deduped.csv`
/// `input` -> `input.deduped.csv`
fn default_output_path(input: &Path) -> PathBuf {
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let filename = format!("{stem}.deduped.csv");
    match input.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir.join(filename),
        _ => PathBuf::from(filename),
    }
}

/// Serialize records to CSV text, escaping fields that contain commas,
/// quotes, or newlines.
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

/// Escape a single CSV field.
fn escape_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') || field.contains('\r') {
        let escaped = field.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        field.to_owned()
    }
}
