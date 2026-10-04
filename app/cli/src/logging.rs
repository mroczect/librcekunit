#![allow(dead_code)]

//! Logger JSON-lines ke stderr.
//!
//! Aktif kalau `CEKUNIT_LOG` di-set. Nilai:
//! - `error` | `warn` | `info` | `debug`

use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Level {
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
}

impl Level {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
        }
    }

    fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "error" => Some(Self::Error),
            "warn" | "warning" => Some(Self::Warn),
            "info" => Some(Self::Info),
            "debug" => Some(Self::Debug),
            _ => None,
        }
    }
}

static LEVEL: OnceLock<Option<Level>> = OnceLock::new();

/// Baca `CEKUNIT_LOG` dan simpan level.
pub fn init() {
    let level = std::env::var("CEKUNIT_LOG")
        .ok()
        .and_then(|v| Level::parse(&v));
    let _ = LEVEL.set(level);
}

fn current() -> Option<Level> {
    *LEVEL.get().unwrap_or(&None)
}

/// Log ke stderr dengan level `info`.
pub fn info(msg: &str) {
    log(Level::Info, msg);
}

/// Log ke stderr dengan level `warn`.
pub fn warn(msg: &str) {
    log(Level::Warn, msg);
}

/// Log ke stderr dengan level `debug`.
pub fn debug(msg: &str) {
    log(Level::Debug, msg);
}

/// Log progress download (kalau level cukup).
pub fn progress(mb: usize, bytes: usize) {
    let Some(level) = current() else { return };
    if level < Level::Info {
        return;
    }
    eprintln!(r#"{{"level":"info","msg":"download_progress","mb":{mb},"bytes":{bytes}}}"#);
}

fn log(level: Level, msg: &str) {
    let Some(current) = current() else { return };
    if level > current {
        return;
    }
    let escaped = msg.replace('\\', "\\\\").replace('"', "\\\"");
    eprintln!(r#"{{"level":"{}","msg":"{}"}}"#, level.as_str(), escaped);
}
