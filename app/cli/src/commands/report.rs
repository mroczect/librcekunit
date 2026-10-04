//! `cekunit report`.

use serde_json::{Value, json};

use crate::cli::ReportArgs;
use crate::csv_ops;
use crate::error::{Error, Result};
use crate::output;

/// Jalankan `report`.
///
/// # Errors
///
/// Return error kalau salah satu step gagal.
pub async fn run(args: ReportArgs) -> Result<Value> {
    let (start, end, label) = super::input_user::resolve_date(&args.date)?;
    let dir = output::resolve_out_dir(args.out.as_deref())?;
    let raw_path = dir.join(format!("cekunit_{label}.raw.csv"));
    let deduped_path = dir.join(format!("cekunit_{label}.deduped.csv"));
    let report_path = dir.join(format!("cekunit_{label}.report.csv"));

    for path in [&raw_path, &deduped_path, &report_path] {
        if path.exists() && !args.force {
            return Err(Error::Io(format!(
                "file sudah ada: {}; pakai --force",
                path.display()
            )));
        }
    }

    let bytes = output::download_csv(&start, &end).await?;
    std::fs::write(&raw_path, &bytes)?;

    let by = csv_ops::split_columns(&args.dedup_by);
    let text = String::from_utf8_lossy(&bytes);
    let deduped = csv_ops::dedupe_csv(&text, &by, args.keep.is_last())?;
    std::fs::write(&deduped_path, deduped.csv.as_bytes())?;

    let rows = csv_ops::split_columns(&args.pivot_rows);
    let pivot = csv_ops::pivot_csv(
        &deduped.csv,
        &rows,
        &args.pivot_values,
        args.pivot_agg.as_str(),
    )?;
    std::fs::write(&report_path, pivot.csv.as_bytes())?;

    Ok(json!({
        "range": { "start": start, "end": end, "label": label },
        "files": {
            "raw": raw_path.display().to_string(),
            "deduped": deduped_path.display().to_string(),
            "report": report_path.display().to_string(),
        },
        "counts": {
            "raw_bytes": bytes.len(),
            "deduped_input_rows": deduped.input_rows,
            "deduped_output_rows": deduped.output_rows,
            "deduped_removed": deduped.removed,
            "report_rows": pivot.rows,
        },
        "report_columns": pivot.columns,
        "preview": pivot.preview,
    }))
}
