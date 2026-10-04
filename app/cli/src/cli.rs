//! Definisi argumen CLI.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

/// Top-level CLI.
#[derive(Debug, Parser)]
#[command(
    name = "cekunit",
    version,
    about = "Cek Unit CLI (machine-oriented)",
    long_about = None,
    disable_help_subcommand = true,
)]
pub struct Cli {
    /// Subcommand.
    #[command(subcommand)]
    pub command: Command,
}

/// Semua subcommand.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Probe server, tampilkan status session.
    Info,

    /// Login, logout, cek session.
    Auth {
        #[command(subcommand)]
        cmd: AuthCmd,
    },

    /// Operasi dashboard.
    Dashboard {
        #[command(subcommand)]
        cmd: DashboardCmd,
    },

    /// Operasi input-user (server).
    #[command(name = "input-user")]
    InputUser {
        #[command(subcommand)]
        cmd: InputUserCmd,
    },

    /// Operasi input-data.
    #[command(name = "input-data")]
    InputData {
        #[command(subcommand)]
        cmd: InputDataCmd,
    },

    /// Operasi CSV lokal.
    Csv {
        #[command(subcommand)]
        cmd: CsvCmd,
    },

    /// Download + dedup + pivot.
    Report(ReportArgs),

    /// Validasi + backup + delete-all + upload.
    Replace(ReplaceArgs),
}

impl Command {
    /// Nama command untuk field `command` di JSON.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Auth { .. } => "auth",
            Self::Dashboard { .. } => "dashboard",
            Self::InputUser { .. } => "input-user",
            Self::InputData { .. } => "input-data",
            Self::Csv { .. } => "csv",
            Self::Report(_) => "report",
            Self::Replace(_) => "replace",
        }
    }
}

// =============================================================================
// Auth
// =============================================================================

/// Subcommand `auth`.
#[derive(Debug, Subcommand)]
pub enum AuthCmd {
    /// Login dan verifikasi session.
    Login,
    /// Logout dengan aman.
    Logout,
    /// Tampilkan status session.
    Info,
}

// =============================================================================
// Dashboard
// =============================================================================

/// Subcommand `dashboard`.
#[derive(Debug, Subcommand)]
pub enum DashboardCmd {
    /// List halaman dashboard.
    List(ListArgs),
    /// Ambil nilai unik dari sebuah kolom.
    Unique {
        /// Nama kolom.
        column: String,
    },
    /// Hapus baris dengan `column=value`.
    DeleteCategory(DeleteCategoryArgs),
    /// Hapus semua baris.
    DeleteAll(ExecuteArgs),
}

/// Argumen `dashboard delete-category`.
#[derive(Debug, Args)]
pub struct DeleteCategoryArgs {
    /// `column=value`.
    #[arg(value_name = "COLUMN=VALUE")]
    pub column_value: String,
    /// Minta eksekusi (butuh `CEKUNIT_ALLOW_WRITE=1`).
    #[arg(long)]
    pub execute: bool,
}

/// Argumen write generik.
#[derive(Debug, Args)]
pub struct ExecuteArgs {
    /// Minta eksekusi.
    #[arg(long)]
    pub execute: bool,
}

// =============================================================================
// List
// =============================================================================

/// Argumen list.
#[derive(Debug, Args)]
pub struct ListArgs {
    /// Jumlah halaman.
    #[arg(long, default_value_t = 1)]
    pub pages: u32,
    /// Kolom sort.
    #[arg(long, default_value = "")]
    pub sort: String,
    /// `asc` atau `desc`.
    #[arg(long, default_value = "asc")]
    pub direction: String,
    /// Kata kunci.
    #[arg(long, default_value = "")]
    pub search: String,
}

// =============================================================================
// Input-user
// =============================================================================

/// Subcommand `input-user`.
#[derive(Debug, Subcommand)]
pub enum InputUserCmd {
    /// List halaman input-user.
    List(ListArgs),
    /// Download CSV.
    Export(ExportArgs),
    /// Scan + group.
    Report(InputUserReportArgs),
}

/// Argumen `input-user export`.
#[derive(Debug, Args)]
pub struct ExportArgs {
    /// Tanggal: `today`, `YYYY-MM-DD`, atau `START:END`.
    #[arg(value_name = "DATE")]
    pub date: DateArg,
    /// Folder output.
    #[arg(long, value_name = "DIR")]
    pub out: Option<PathBuf>,
    /// Timpa file yang ada.
    #[arg(long)]
    pub force: bool,
}

/// Argumen `input-user report`.
#[derive(Debug, Args)]
pub struct InputUserReportArgs {
    /// Jumlah halaman.
    #[arg(long, default_value_t = 5)]
    pub pages: u32,
    /// Group by kolom.
    #[arg(long)]
    pub by: Option<String>,
    /// Tulis ke CSV.
    #[arg(long, value_name = "FILE")]
    pub csv: Option<PathBuf>,
}

// =============================================================================
// Date
// =============================================================================

/// Spesifikasi tanggal.
///
/// Format:
/// - `today` вЂ” hari ini
/// - `YYYY-MM-DD` вЂ” tanggal tertentu
/// - `START:END` вЂ” rentang inklusif, mis. `2026-10-01:2026-10-05`
#[derive(Debug, Clone)]
pub enum DateArg {
    /// Hari ini.
    Today,
    /// Tanggal tertentu.
    Date(String),
    /// Rentang inklusif.
    Range { start: String, end: String },
}

impl DateArg {
    /// Parse dari string.
    ///
    /// # Errors
    ///
    /// Return error string kalau format tidak dikenal.
    pub fn parse(s: &str) -> Result<Self, String> {
        let s = s.trim();
        if s.is_empty() {
            return Err(String::from("tanggal kosong"));
        }
        if s.eq_ignore_ascii_case("today") {
            return Ok(Self::Today);
        }
        if let Some((start, end)) = s.split_once(':') {
            let start = start.trim();
            let end = end.trim();
            if start.is_empty() || end.is_empty() {
                return Err(String::from("rentang butuh START:END"));
            }
            return Ok(Self::Range {
                start: start.to_owned(),
                end: end.to_owned(),
            });
        }
        Ok(Self::Date(s.to_owned()))
    }
}

impl std::str::FromStr for DateArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

// =============================================================================
// Input-data
// =============================================================================

/// Subcommand `input-data`.
#[derive(Debug, Subcommand)]
pub enum InputDataCmd {
    /// Insert satu record.
    Single(SingleArgs),
    /// Upload CSV.
    Csv(CsvUploadArgs),
}

/// Argumen `input-data single`.
#[derive(Debug, Args)]
pub struct SingleArgs {
    /// Minta eksekusi.
    #[arg(long)]
    pub execute: bool,

    /// Field `--key value` (diulang).
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub fields: Vec<String>,
}

/// Argumen `input-data csv`.
#[derive(Debug, Args)]
pub struct CsvUploadArgs {
    /// Path file CSV.
    pub file: PathBuf,
    /// Minta eksekusi.
    #[arg(long)]
    pub execute: bool,
}

// =============================================================================
// CSV
// =============================================================================

/// Subcommand `csv`.
#[derive(Debug, Subcommand)]
pub enum CsvCmd {
    /// Validasi CSV.
    Check {
        /// Path file.
        file: PathBuf,
    },
    /// Hapus duplikat.
    Dedup(DedupArgs),
    /// Pivot.
    Pivot(PivotArgs),
}

/// Argumen `csv dedup`.
#[derive(Debug, Args)]
pub struct DedupArgs {
    /// Path input.
    pub file: PathBuf,
    /// Kolom kunci, pisah koma.
    #[arg(long, value_name = "COL1,COL2")]
    pub by: String,
    /// `first` atau `last`.
    #[arg(long, default_value = "first", value_enum)]
    pub keep: KeepArg,
    /// Path output.
    #[arg(long, value_name = "FILE")]
    pub out: Option<PathBuf>,
    /// Timpa file.
    #[arg(long)]
    pub force: bool,
}

/// Argumen `csv pivot`.
#[derive(Debug, Args)]
pub struct PivotArgs {
    /// Path input.
    pub file: PathBuf,
    /// Kolom baris, pisah koma.
    #[arg(long, value_name = "COL1,COL2")]
    pub rows: String,
    /// Kolom nilai.
    #[arg(long)]
    pub values: String,
    /// Agregasi.
    #[arg(long, default_value = "count-distinct", value_enum)]
    pub agg: AggArg,
    /// Path output.
    #[arg(long, value_name = "FILE")]
    pub out: Option<PathBuf>,
    /// Timpa file.
    #[arg(long)]
    pub force: bool,
}

/// Nilai `--keep`.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum KeepArg {
    /// Kemunculan pertama.
    First,
    /// Kemunculan terakhir.
    Last,
}

impl KeepArg {
    /// Apakah keep last.
    #[must_use]
    pub const fn is_last(self) -> bool {
        matches!(self, Self::Last)
    }
}

/// Nilai `--agg`.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum AggArg {
    /// Hitung baris.
    Count,
    /// Hitung nilai unik.
    CountDistinct,
    /// Jumlahkan.
    Sum,
}

impl AggArg {
    /// String yang diterima `csv_ops`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Count => "count",
            Self::CountDistinct => "count-distinct",
            Self::Sum => "sum",
        }
    }
}

// =============================================================================
// Report
// =============================================================================

/// Argumen `report`.
#[derive(Debug, Args)]
pub struct ReportArgs {
    /// Tanggal: `today`, `YYYY-MM-DD`, atau `START:END`.
    #[arg(value_name = "DATE")]
    pub date: DateArg,
    /// Kunci dedupe.
    #[arg(long, default_value = "userID,nopol")]
    pub dedup_by: String,
    /// `first` atau `last`.
    #[arg(long, default_value = "first", value_enum)]
    pub keep: KeepArg,
    /// Kolom baris pivot.
    #[arg(long, default_value = "nama")]
    pub pivot_rows: String,
    /// Kolom nilai pivot.
    #[arg(long, default_value = "nopol")]
    pub pivot_values: String,
    /// Agregasi pivot.
    #[arg(long, default_value = "count-distinct", value_enum)]
    pub pivot_agg: AggArg,
    /// Folder output.
    #[arg(long, value_name = "DIR")]
    pub out: Option<PathBuf>,
    /// Timpa file.
    #[arg(long)]
    pub force: bool,
}

// =============================================================================
// Replace
// =============================================================================

/// Argumen `replace`.
#[derive(Debug, Args)]
pub struct ReplaceArgs {
    /// Path CSV untuk upload.
    pub file: PathBuf,
    /// Minta eksekusi.
    #[arg(long)]
    pub execute: bool,
    /// Skip backup.
    #[arg(long)]
    pub no_backup: bool,
}
