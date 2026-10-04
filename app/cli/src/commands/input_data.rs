//! `cekunit input-data`.

use librcekunit_client::prelude::{Form, InputData};
use serde_json::{Value, json};

use crate::cli::{CsvUploadArgs, InputDataCmd, SingleArgs};
use crate::error::{Error, Result};
use crate::safety;

/// Dispatch `input-data`.
///
/// # Errors
///
/// Return error kalau request gagal.
pub async fn run(cmd: InputDataCmd) -> Result<Value> {
    match cmd {
        InputDataCmd::Single(args) => single(&args).await,
        InputDataCmd::Csv(args) => csv_upload(&args).await,
    }
}

async fn single(args: &SingleArgs) -> Result<Value> {
    let fields = parse_fields(&args.fields)?;
    if fields.is_empty() {
        return Err(Error::Usage(String::from(
            "butuh minimal satu field --<key> <value>",
        )));
    }

    let mut form = Form::new();
    for (k, v) in &fields {
        let _ = form.insert(k.clone(), v.clone());
    }

    let execute = safety::gate(args.execute)?;
    if !execute {
        return Ok(json!({
            "action": "input-data.single",
            "fields": fields,
            "executed": false,
            "reason": "flag --execute tidak diberikan",
        }));
    }

    let client = crate::client::connect().await?;
    let resp = client.input_data_store(form).await?;
    Ok(json!({
        "action": "input-data.single",
        "executed": true,
        "status": resp.status().as_u16(),
    }))
}

async fn csv_upload(args: &CsvUploadArgs) -> Result<Value> {
    if !args.file.exists() {
        return Err(Error::Io(format!(
            "file tidak ada: {}",
            args.file.display()
        )));
    }

    let text = std::fs::read_to_string(&args.file)?;
    let data_rows = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .count()
        .saturating_sub(1);

    let execute = safety::gate(args.execute)?;
    if !execute {
        return Ok(json!({
            "action": "input-data.csv",
            "file": args.file.display().to_string(),
            "data_rows": data_rows,
            "executed": false,
            "reason": "flag --execute tidak diberikan",
        }));
    }

    let client = crate::client::connect().await?;
    let resp = client.import_csv(&args.file).await?;
    Ok(json!({
        "action": "input-data.csv",
        "executed": true,
        "status": resp.status().as_u16(),
    }))
}

fn parse_fields(tokens: &[String]) -> Result<Vec<(String, String)>> {
    let mut fields = Vec::new();
    let mut iter = tokens.iter();
    while let Some(tok) = iter.next() {
        if let Some(key) = tok.strip_prefix("--") {
            let value = iter
                .next()
                .ok_or_else(|| Error::Usage(format!("{key} butuh value")))?;
            fields.push((key.to_owned(), value.clone()));
        }
    }
    Ok(fields)
}
