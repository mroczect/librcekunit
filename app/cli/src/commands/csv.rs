//! `cekunit csv` — operasi lokal.

use librcekunit_examples::csv_check;
use serde_json::{Value, json};

use crate::cli::{CsvCmd, DedupArgs, PivotArgs};
use crate::csv_ops;
use crate::error::{Error, Result};
use crate::output;

/// Dispatch subcommand `csv`.
///
/// # Errors
///
/// Return error kalau file tidak bisa dibaca atau output gagal ditulis.
pub fn run(cmd: CsvCmd) -> Result<Value> {
    match cmd {
        CsvCmd::Check { file } => check(&file),
        CsvCmd::Dedup(args) => dedup(&args),
        CsvCmd::Pivot(args) => pivot(&args),
    }
}

fn check(file: &std::path::Path) -> Result<Value> {
    let text = std::fs::read_to_string(file)?;
    let summary =
        csv_check::validate(&text).map_err(|e| Error::Csv(format!("validasi gagal: {e}")))?;
    Ok(json!({
        "file": file.display().to_string(),
        "header": summary.header,
        "data_rows": summary.data_rows,
        "valid": true,
    }))
}

fn dedup(args: &DedupArgs) -> Result<Value> {
    let text = std::fs::read_to_string(&args.file)?;
    let by = csv_ops::split_columns(&args.by);
    if by.is_empty() {
        return Err(Error::Usage(String::from("--by tidak boleh kosong")));
    }

    let result = csv_ops::dedupe_csv(&text, &by, args.keep.is_last())?;
    let target = args
        .out
        .clone()
        .unwrap_or_else(|| output::default_suffixed_path(&args.file, "deduped"));

    if target.exists() && !args.force {
        return Err(Error::Io(format!(
            "file sudah ada: {}; pakai --force",
            target.display()
        )));
    }

    std::fs::write(&target, result.csv.as_bytes())?;

    Ok(json!({
        "input": args.file.display().to_string(),
        "output": target.display().to_string(),
        "input_rows": result.input_rows,
        "output_rows": result.output_rows,
        "removed": result.removed,
        "bytes": result.csv.len(),
    }))
}

fn pivot(args: &PivotArgs) -> Result<Value> {
    let text = std::fs::read_to_string(&args.file)?;
    let rows = csv_ops::split_columns(&args.rows);
    if rows.is_empty() {
        return Err(Error::Usage(String::from("--rows tidak boleh kosong")));
    }

    let result = csv_ops::pivot_csv(&text, &rows, &args.values, args.agg.as_str())?;
    let target = args
        .out
        .clone()
        .unwrap_or_else(|| output::default_suffixed_path(&args.file, "pivot"));

    if target.exists() && !args.force {
        return Err(Error::Io(format!(
            "file sudah ada: {}; pakai --force",
            target.display()
        )));
    }

    std::fs::write(&target, result.csv.as_bytes())?;

    Ok(json!({
        "input": args.file.display().to_string(),
        "output": target.display().to_string(),
        "rows": result.rows,
        "columns": result.columns,
        "bytes": result.csv.len(),
        "preview": result.preview,
    }))
}
