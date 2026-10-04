//! `cekunit replace`.

use librcekunit_client::prelude::Dashboard;
use librcekunit_examples::csv_check;
use librcekunit_examples::parse;
use librcekunit_examples::scan::{self, ScanOptions};
use serde_json::{Value, json};

use crate::cli::ReplaceArgs;
use crate::error::{Error, Result};
use crate::output;
use crate::safety;

/// Jalankan `replace`.
///
/// # Errors
///
/// Return error kalau salah satu step gagal.
pub async fn run(args: ReplaceArgs) -> Result<Value> {
    if !args.file.exists() {
        return Err(Error::Io(format!(
            "file tidak ada: {}",
            args.file.display()
        )));
    }

    let text = std::fs::read_to_string(&args.file)?;
    let summary =
        csv_check::validate(&text).map_err(|e| Error::Csv(format!("validasi gagal: {e}")))?;

    let execute = safety::gate(args.execute)?;
    if !execute {
        return Ok(json!({
            "action": "replace",
            "file": args.file.display().to_string(),
            "data_rows": summary.data_rows,
            "columns": summary.header.len(),
            "executed": false,
            "reason": "flag --execute tidak diberikan",
        }));
    }

    // Backup.
    let backup_path = if args.no_backup {
        None
    } else {
        let path = output::default_backup_path();
        let client = crate::client::connect().await?;
        let opts = ScanOptions::default().max_pages(5);
        let scan_result = scan::scan_all(&client, &opts).await?;
        let csv = parse::rows_to_csv(&scan_result.rows);
        std::fs::write(&path, csv.as_bytes())?;
        Some(path)
    };

    let client = crate::client::connect().await?;

    let delete_resp = client.delete_all().await?;
    let delete_status = delete_resp.status().as_u16();

    let upload_resp = client.import_csv(&args.file).await?;
    let upload_status = upload_resp.status().as_u16();

    Ok(json!({
        "action": "replace",
        "file": args.file.display().to_string(),
        "executed": true,
        "backup": backup_path.map(|p| p.display().to_string()),
        "delete_status": delete_status,
        "upload_status": upload_status,
    }))
}
