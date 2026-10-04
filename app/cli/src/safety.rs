//! Safety gate untuk operasi write.
//!
//! Bukan TTY check. Butuh dua sinyal independen:
//!
//! 1. Flag `--execute` (per perintah)
//! 2. Env var `CEKUNIT_ALLOW_WRITE=1` (per environment)
//!
//! Kalau salah satu tidak ada, caller bisa menjalankan dalam mode plan.

use crate::error::{Error, Result};

/// Nama env var yang mengizinkan write.
pub const ENV_ALLOW_WRITE: &str = "CEKUNIT_ALLOW_WRITE";

#[must_use]
pub fn write_allowed_by_env() -> bool {
    std::env::var(ENV_ALLOW_WRITE).is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}
/// Cek apakah write boleh dieksekusi.
///
/// # Errors
///
/// Return `Error::WriteRefused` kalau caller tidak memberi sinyal.
pub fn gate(execute_flag: bool) -> Result<bool> {
    if !execute_flag {
        // Caller tidak minta execute. Ini bukan error, cuma plan.
        return Ok(false);
    }
    if !write_allowed_by_env() {
        return Err(Error::WriteRefused(format!(
            "flag --execute diberikan tapi {ENV_ALLOW_WRITE}=1 tidak di-set"
        )));
    }
    Ok(true)
}
