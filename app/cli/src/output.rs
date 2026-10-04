//! Envelope JSON untuk stdout.

use serde::Serialize;

use crate::error::{Error, Result};

#[derive(Debug, Serialize)]
struct OkEnvelope<'a, T> {
    ok: bool,
    command: &'a str,
    data: T,
}

#[derive(Debug, Serialize)]
struct ErrEnvelope<'a> {
    ok: bool,
    command: &'a str,
    error: ErrorBody,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    kind: &'static str,
    code: u8,
    message: String,
    retryable: bool,
}

/// Cetak envelope sukses ke stdout.
pub fn print_ok<T: Serialize>(command: &str, data: &T) {
    let envelope = OkEnvelope {
        ok: true,
        command,
        data,
    };
    emit(&envelope);
}

/// Cetak envelope error ke stdout.
pub fn print_error(command: &str, err: &Error) {
    let envelope = ErrEnvelope {
        ok: false,
        command,
        error: ErrorBody {
            kind: err.kind(),
            code: err.exit_code().as_u8(),
            message: err.message(),
            retryable: err.retryable(),
        },
    };
    emit(&envelope);
}

fn emit<T: Serialize>(value: &T) {
    match serde_json::to_string(value) {
        Ok(json) => println!("{json}"),
        Err(e) => {
            eprintln!(
                r#"{{"ok":false,"command":"internal","error":{{"kind":"internal","code":1,"message":"serialize failed: {e}","retryable":false}}}}"#
            );
        }
    }
}

/// Path dengan suffix: `foo.csv` в†’ `foo.<suffix>.csv`.
#[must_use]
pub fn default_suffixed_path(input: &std::path::Path, suffix: &str) -> std::path::PathBuf {
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let filename = format!("{stem}.{suffix}.csv");
    match input.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir.join(filename),
        _ => std::path::PathBuf::from(filename),
    }
}

/// Backup path default.
#[must_use]
pub fn default_backup_path() -> std::path::PathBuf {
    let epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    std::path::PathBuf::from(format!("pre_replace_backup_{epoch}.csv"))
}

/// Resolve output directory.
///
/// # Errors
///
/// Return error kalau mkdir gagal.
pub fn resolve_out_dir(out_dir: Option<&std::path::Path>) -> Result<std::path::PathBuf> {
    let dir = match out_dir {
        Some(d) => d.to_path_buf(),
        None => default_downloads_dir()?,
    };
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

fn default_downloads_dir() -> Result<std::path::PathBuf> {
    let home =
        std::env::var("HOME").map_err(|_err| Error::Config(String::from("$HOME tidak di-set")))?;
    Ok(std::path::Path::new(&home).join("Downloads"))
}

// =============================================================================
// Download CSV
// =============================================================================

/// Download CSV range.
///
/// Kalau server balas HTML (biasanya karena session expired), coba login
/// ulang otomatis sekali. Kalau tetap HTML, baru menyerah dengan error auth.
///
/// # Errors
///
/// Return error kalau request atau baca body gagal, atau server tetap
/// mengembalikan HTML setelah login ulang.
pub async fn download_csv(start_date: &str, end_date: &str) -> Result<Vec<u8>> {
    // Attempt 1: pakai session apa adanya (connect() yang akan login kalau perlu).
    match fetch_csv_once(start_date, end_date, false).await {
        Ok(bytes) => Ok(bytes),
        Err(Error::Auth(_)) => {
            crate::logging::warn("download_csv: session expired, retry dengan login fresh");
            fetch_csv_once(start_date, end_date, true).await
        }
        Err(Error::Api { status, .. }) if status == 401 || status == 403 => {
            crate::logging::warn("download_csv: server balas 401/403, retry login fresh");
            fetch_csv_once(start_date, end_date, true).await
        }
        Err(e) => Err(e),
    }
}

async fn fetch_csv_once(
    start_date: &str,
    end_date: &str,
    fresh_login: bool,
) -> Result<Vec<u8>> {
    use librcekunit_client::prelude::{Form, InputUser};

    let client = if fresh_login {
        crate::client::connect_fresh().await?
    } else {
        crate::client::connect().await?
    };

    let mut params = Form::new();
    let _ = params.insert(String::from("format"), String::from("csv"));
    let _ = params.insert(String::from("sort"), String::from("created_at"));
    let _ = params.insert(String::from("direction"), String::from("asc"));
    let _ = params.insert(String::from("start_date"), start_date.to_owned());
    let _ = params.insert(String::from("end_date"), end_date.to_owned());
    let _ = params.insert(String::from("search"), String::new());

    let mut resp = client.input_user_export(params).await?;
    let status = resp.status();
    let url = resp.url().to_string();

    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        let n = body.len().min(300);
        let preview = body.get(..n).unwrap_or(&body);
        return Err(Error::Api {
            status: status.as_u16(),
            body: preview.to_owned(),
        });
    }

    let mut bytes: Vec<u8> = Vec::new();
    let mut checked_first = false;
    let mut chunks: u64 = 0;
    let mut last_mb: usize = 0;

    while let Some(chunk) = resp
        .chunk()
        .await
        .map_err(|e| Error::Network(e.to_string()))?
    {
        if !checked_first {
            checked_first = true;
            if looks_like_html(&chunk) {
                let preview = preview_text(&chunk, 200);
                return Err(Error::Auth(format!(
                    "server balas HTML di {url}, bukan CSV (session expired?): {preview}"
                )));
            }
        }

        bytes.extend_from_slice(&chunk);
        chunks = chunks.saturating_add(1);
        let mb = bytes.len() / (1024 * 1024);
        if mb > last_mb {
            last_mb = mb;
            crate::logging::progress(mb, bytes.len());
        }
    }

    if bytes.is_empty() {
        return Err(Error::Api {
            status: 200,
            body: String::from("empty body"),
        });
    }

    // Sanity check sekali lagi di buffer utuh вЂ” jaga-jaga kalau chunk
    // pertama cuma BOM/whitespace.
    if looks_like_html(&bytes) {
        let preview = preview_text(&bytes, 200);
        return Err(Error::Auth(format!(
            "server balas HTML di {url}, bukan CSV (session expired?): {preview}"
        )));
    }

    Ok(bytes)
}

/// Deteksi apakah bytes awal terlihat seperti dokumen HTML.
fn looks_like_html(bytes: &[u8]) -> bool {
    let n = bytes.len().min(256);
    let head = &bytes[..n];
    let s = String::from_utf8_lossy(head);
    let s = s.trim_start_matches(|c: char| c.is_ascii_whitespace() || c == '\u{feff}');
    let lower = s.to_ascii_lowercase();
    lower.starts_with("<!doctype") || lower.starts_with("<html")
}

/// Potong preview pendek, buang newline supaya rapi di JSON.
fn preview_text(bytes: &[u8], max: usize) -> String {
    let n = bytes.len().min(max);
    let s = String::from_utf8_lossy(&bytes[..n]);
    s.trim().replace('\n', " ")
}
