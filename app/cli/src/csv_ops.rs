//! Operasi CSV lokal.
//!
//! Fungsi mengembalikan data terstruktur yang siap di-serialize ke JSON.

use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, HashSet};

use serde::Serialize;

use crate::error::{Error, Result};

/// Split kolom dengan koma.
#[must_use]
pub fn split_columns(s: &str) -> Vec<String> {
    s.split(',')
        .map(|p| p.trim().to_owned())
        .filter(|p| !p.is_empty())
        .collect()
}

/// Hasil dedup.
#[derive(Debug, Serialize)]
pub struct DedupResult {
    pub input_rows: usize,
    pub output_rows: usize,
    pub removed: usize,
    pub csv: String,
}

/// Bucket untuk pivot.
#[derive(Debug, Default)]
struct Bucket {
    count: u64,
    distinct: HashSet<String>,
    sum: f64,
}

/// Hasil pivot.
#[derive(Debug, Serialize)]
pub struct PivotResult {
    pub rows: usize,
    pub columns: Vec<String>,
    pub csv: String,
    pub preview: Vec<Vec<String>>,
}

/// Dedup CSV text.
///
/// # Errors
///
/// Return error kalau CSV malformed atau kolom tidak ditemukan.
pub fn dedupe_csv(text: &str, by: &[String], keep_last: bool) -> Result<DedupResult> {
    let mut records = librcekunit_examples::csv_check::parse_records(text)?;
    if records.is_empty() {
        return Err(Error::Csv(String::from("CSV kosong")));
    }

    let header = records.remove(0);
    let key_indices = resolve_indices(&header, by)?;

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

        if keep_last {
            if let Some(&prev) = index_of.get(&key) {
                if let Some(slot) = out.get_mut(prev) {
                    slot.clone_from(&row);
                }
            } else {
                let _ = index_of.insert(key, out.len());
                out.push(row);
            }
        } else if let Entry::Vacant(e) = index_of.entry(key) {
            let _ = e.insert(out.len());
            out.push(row);
        }
    }

    let output_rows = out.len();
    let removed = input_rows.saturating_sub(output_rows);

    let mut all = Vec::with_capacity(output_rows.saturating_add(1));
    all.push(header);
    all.extend(out);

    Ok(DedupResult {
        input_rows,
        output_rows,
        removed,
        csv: records_to_csv(&all),
    })
}

/// Pivot CSV text.
///
/// # Errors
///
/// Return error kalau CSV malformed atau kolom tidak ditemukan.
pub fn pivot_csv(text: &str, rows: &[String], values: &str, agg: &str) -> Result<PivotResult> {
    let mut records = librcekunit_examples::csv_check::parse_records(text)?;
    if records.is_empty() {
        return Err(Error::Csv(String::from("CSV kosong")));
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
            "count" => {}
            "sum" => {
                if let Ok(n) = value.trim().parse::<f64>() {
                    bucket.sum += n;
                }
            }
            // default count-distinct
            _ => {
                let _ = bucket.distinct.insert(value);
            }
        }
    }

    let mut rows_out: Vec<(Vec<String>, f64)> = buckets
        .into_iter()
        .map(|(key, b)| {
            let value = match agg {
                "count" => {
                    #[allow(clippy::cast_precision_loss)]
                    {
                        b.count as f64
                    }
                }
                "sum" => b.sum,
                _ => {
                    #[allow(clippy::cast_precision_loss)]
                    {
                        b.distinct.len() as f64
                    }
                }
            };
            (key, value)
        })
        .collect();

    rows_out.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(core::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });

    let suffix = match agg {
        "count" => "count",
        "sum" => "sum",
        _ => "count_distinct",
    };

    let mut out_header: Vec<String> = rows.to_vec();
    out_header.push(format!("{suffix}_{values}"));

    let mut csv = String::new();
    csv.push_str(
        &out_header
            .iter()
            .map(|h| escape_field(h))
            .collect::<Vec<_>>()
            .join(","),
    );
    csv.push('\n');

    let mut preview: Vec<Vec<String>> = Vec::new();
    preview.push(out_header.clone());

    for (i, (key, value)) in rows_out.iter().enumerate() {
        let mut parts = key.clone();
        parts.push(fmt_value(*value));
        csv.push_str(
            &parts
                .iter()
                .map(|f| escape_field(f))
                .collect::<Vec<_>>()
                .join(","),
        );
        csv.push('\n');
        if i < 10 {
            preview.push(parts);
        }
    }

    Ok(PivotResult {
        rows: rows_out.len(),
        columns: out_header,
        csv,
        preview,
    })
}

fn resolve_indices(header: &[String], names: &[String]) -> Result<Vec<usize>> {
    let mut indices = Vec::with_capacity(names.len());
    for name in names {
        if let Some(i) = header.iter().position(|h| h == name) {
            indices.push(i);
        } else {
            return Err(Error::Csv(format!(
                "kolom tidak ditemukan: {name}; tersedia: {}",
                header.join(", ")
            )));
        }
    }
    Ok(indices)
}

fn resolve_single_index(header: &[String], name: &str) -> Result<usize> {
    header.iter().position(|h| h == name).ok_or_else(|| {
        Error::Csv(format!(
            "kolom tidak ditemukan: {name}; tersedia: {}",
            header.join(", ")
        ))
    })
}

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

fn escape_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') || field.contains('\r') {
        let escaped = field.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        field.to_owned()
    }
}
