//! `cekunit dashboard`.

use librcekunit_client::prelude::{Dashboard, Form};
use serde_json::{Value, json};

use crate::cli::{DashboardCmd, DeleteCategoryArgs, ExecuteArgs, ListArgs};
use crate::error::{Error, Result};
use crate::safety;

/// Dispatch `dashboard`.
///
/// # Errors
///
/// Return error kalau request gagal.
pub async fn run(cmd: DashboardCmd) -> Result<Value> {
    match cmd {
        DashboardCmd::List(args) => list(&args).await,
        DashboardCmd::Unique { column } => unique(&column).await,
        DashboardCmd::DeleteCategory(args) => delete_category(&args).await,
        DashboardCmd::DeleteAll(args) => delete_all(&args).await,
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
        let resp = client.dashboard_index_with_params(params).await?;
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

async fn unique(column: &str) -> Result<Value> {
    let client = crate::client::connect().await?;
    let resp = client.get_unique_values(column).await?;
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    Ok(json!({
        "column": column,
        "status": status.as_u16(),
        "raw": body,
    }))
}

async fn delete_category(args: &DeleteCategoryArgs) -> Result<Value> {
    let (column, value) = args
        .column_value
        .split_once('=')
        .ok_or_else(|| Error::Usage(String::from("format: column=value")))?;

    let execute = safety::gate(args.execute)?;
    if !execute {
        return Ok(json!({
            "action": "delete-category",
            "column": column,
            "value": value,
            "executed": false,
            "reason": "flag --execute tidak diberikan",
        }));
    }

    let client = crate::client::connect().await?;
    let resp = client.delete_by_category(column, value).await?;
    Ok(json!({
        "action": "delete-category",
        "column": column,
        "value": value,
        "executed": true,
        "status": resp.status().as_u16(),
    }))
}

async fn delete_all(args: &ExecuteArgs) -> Result<Value> {
    let execute = safety::gate(args.execute)?;
    if !execute {
        return Ok(json!({
            "action": "delete-all",
            "executed": false,
            "reason": "flag --execute tidak diberikan",
        }));
    }

    let client = crate::client::connect().await?;
    let resp = client.delete_all().await?;
    Ok(json!({
        "action": "delete-all",
        "executed": true,
        "status": resp.status().as_u16(),
    }))
}
