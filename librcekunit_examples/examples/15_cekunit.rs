#![allow(unused_crate_dependencies)]
#![allow(clippy::std_instead_of_alloc)]

//! # cekunit — all-in-one CLI
//!
//! One binary, one command, every workflow from the workspace.
//! Read-only by default. Every write operation is gated behind
//! `--execute` plus an interactive terminal plus a typed
//! confirmation phrase.
//!
//! ## Subcommands
//!
//! ```text
//! cekunit auth login                     log in and verify session
//! cekunit auth logout                    safe logout
//! cekunit auth info                      probe server, show session
//!
//! cekunit dashboard list [opts]          list /dashboard pages
//! cekunit dashboard unique <col>         fetch unique values
//! cekunit dashboard delete-category C=V   delete rows (gated)
//! cekunit dashboard delete-all            delete all rows (gated)
//!
//! cekunit input-user list [opts]         list /dashboard/input-user
//! cekunit input-user export <date> [opts]   download CSV
//! cekunit input-user report [opts]       scan + group input-user
//!
//! cekunit input-data single [opts]       insert one record
//! cekunit input-data csv <file> [opts]   upload CSV via multipart
//!
//! cekunit csv check <file>               validate CSV locally
//! cekunit csv dedup <file> --by c1,c2    deduplicate CSV
//! cekunit csv pivot <file> --rows c ...  pivot CSV
//!
//! cekunit report <today|date D|range A B>   download + dedup + pivot
//! cekunit replace <file> [--execute]     validate + backup + delete + upload
//! ```
//!
//! ## Date specs
//!
//! Used by `input-user export` and `report`:
//!
//! - `today` — today's local date
//! - `date <YYYY-MM-DD>` — a single date
//! - `range <START> <END>` — inclusive range
//!
//! ## Common options
//!
//! - `--out <dir>` — output directory, defaults to `~/Downloads`
//! - `--force` — overwrite existing files
//! - `--execute` — required for every write operation
//!
//! ## Safety
//!
//! - Read-only by default.
//! - `--execute` requires an interactive terminal. Piping stdin is
//!   rejected before any network activity.
//! - Typed confirmation phrase required (`HAPUS` or `HAPUS SEMUA`).
//! - No single flag both selects and confirms a destructive action.
//!
//! ## Environment
//!
//! | Name | Required | Purpose |
//! |---|---|---|
//! | `LIBRCEKUNIT_BASE_URL` | yes | Server URL. |
//! | `LIBRCEKUNIT_EMAIL` | login | Login email. |
//! | `LIBRCEKUNIT_PASSWORD` | login | Login password. |
//! | `LIBRCEKUNIT_COOKIE_FILE` | no | Persist session. |
//! | `LIBRCEKUNIT_TIMEOUT_SECS` | no | Request timeout. |
//! | `RUST_LOG` | no | Tracing filter. |

#![allow(clippy::wildcard_imports)]

use core::iter::Peekable;
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, HashSet};
use std::io::IsTerminal as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use librcekunit_examples::auth;
use librcekunit_examples::csv_check;
use librcekunit_examples::error;
use librcekunit_examples::parse;
use librcekunit_examples::prelude::*;
use librcekunit_examples::scan::{self, ScanOptions};

/// Boxed, thread-safe error type used as the return of `main`.
type DynError = Box<dyn core::error::Error + Send + Sync>;

// =============================================================================
// Shared types
// =============================================================================

/// Which occurrence to keep when deduplicating.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Keep {
    First,
    Last,
}

impl Keep {
    fn parse(s: &str) -> Result<Self, DynError> {
        match s {
            "first" => Ok(Self::First),
            "last" => Ok(Self::Last),
            other => Err(format!("--keep must be first or last, got {other:?}").into()),
        }
    }
}

/// Aggregation for the pivot step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Agg {
    Count,
    CountDistinct,
    Sum,
}

impl Agg {
    fn parse(s: &str) -> Result<Self, DynError> {
        match s {
            "count" => Ok(Self::Count),
            "count-distinct" => Ok(Self::CountDistinct),
            "sum" => Ok(Self::Sum),
            other => {
                Err(format!("--agg must be count, count-distinct, or sum, got {other:?}").into())
            }
        }
    }

    const fn header_suffix(self) -> &'static str {
        match self {
            Self::Count => "count",
            Self::CountDistinct => "count_distinct",
            Self::Sum => "sum",
        }
    }
}

/// A resolved date range with a filename-friendly label.
#[derive(Debug, Clone)]
struct DateRange {
    start: String,
    end: String,
    label: String,
}

/// Date specification from the command line.
#[derive(Debug, Clone)]
enum DateSpec {
    Today,
    Single(String),
    Range { start: String, end: String },
}

impl DateSpec {
    fn resolve(&self) -> Result<DateRange, DynError> {
        match self {
            Self::Today => {
                let today = today_local()?;
                validate_date(&today)?;
                Ok(DateRange {
                    start: today.clone(),
                    end: today.clone(),
                    label: format!("today_{today}"),
                })
            }
            Self::Single(d) => {
                validate_date(d)?;
                Ok(DateRange {
                    start: d.clone(),
                    end: d.clone(),
                    label: d.clone(),
                })
            }
            Self::Range { start, end } => {
                validate_date(start)?;
                validate_date(end)?;
                if start > end {
                    return Err(format!("start {start} is after end {end}").into());
                }
                Ok(DateRange {
                    start: start.clone(),
                    end: end.clone(),
                    label: format!("{start}_to_{end}"),
                })
            }
        }
    }
}

/// Accumulator for one group during a pivot.
#[derive(Debug, Default)]
struct Bucket {
    count: u64,
    distinct: HashSet<String>,
    sum: f64,
}

// =============================================================================
// Command enum
// =============================================================================

#[derive(Debug)]
enum Command {
    AuthLogin,
    AuthLogout,
    AuthInfo,

    DashboardList {
        pages: u32,
        sort: String,
        direction: String,
        search: String,
    },
    DashboardUnique {
        column: String,
    },
    DashboardDeleteCategory {
        column: String,
        value: String,
        execute: bool,
    },
    DashboardDeleteAll {
        execute: bool,
    },

    InputUserList {
        pages: u32,
        sort: String,
        direction: String,
        search: String,
    },
    InputUserExport {
        date: DateSpec,
        out_dir: Option<PathBuf>,
        force: bool,
    },
    InputUserReport {
        pages: u32,
        by: Option<String>,
        csv: Option<PathBuf>,
    },

    InputDataSingle {
        fields: Vec<(String, String)>,
        execute: bool,
    },
    InputDataCsv {
        file: PathBuf,
        execute: bool,
    },

    CsvCheck {
        file: PathBuf,
    },
    CsvDedup {
        file: PathBuf,
        by: Vec<String>,
        keep: Keep,
        out: Option<PathBuf>,
        force: bool,
    },
    CsvPivot {
        file: PathBuf,
        rows: Vec<String>,
        values: String,
        agg: Agg,
        out: Option<PathBuf>,
        force: bool,
    },

    Report {
        date: DateSpec,
        dedup_by: Vec<String>,
        keep: Keep,
        pivot_rows: Vec<String>,
        pivot_values: String,
        pivot_agg: Agg,
        out_dir: Option<PathBuf>,
        force: bool,
    },

    Replace {
        file: PathBuf,
        execute: bool,
        no_backup: bool,
    },
}

// =============================================================================
// Argument parsing
// =============================================================================

#[allow(clippy::too_many_lines)]
fn parse_command() -> Result<Command, DynError> {
    let args: Vec<String> = std::env::args().collect();
    let mut iter = args.iter().skip(1).peekable();

    let top = iter.next().ok_or_else(usage)?;

    match top.as_str() {
        "auth" => parse_auth(&mut iter),
        "dashboard" => parse_dashboard(&mut iter),
        "input-user" => parse_input_user(&mut iter),
        "input-data" => parse_input_data(&mut iter),
        "csv" => parse_csv(&mut iter),
        "report" => parse_report(&mut iter),
        "replace" => parse_replace(&mut iter),
        "help" | "-h" | "--help" => {
            println!("{}", usage());
            std::process::exit(0);
        }
        other => Err(format!("unknown command: {other}\n\n{}", usage()).into()),
    }
}

fn parse_auth<'a, I>(iter: &mut Peekable<I>) -> Result<Command, DynError>
where
    I: Iterator<Item = &'a String>,
{
    let sub = iter.next().ok_or("usage: auth login|logout|info")?;
    match sub.as_str() {
        "login" => Ok(Command::AuthLogin),
        "logout" => Ok(Command::AuthLogout),
        "info" => Ok(Command::AuthInfo),
        other => Err(format!("unknown auth subcommand: {other}").into()),
    }
}

fn parse_dashboard<'a, I>(iter: &mut Peekable<I>) -> Result<Command, DynError>
where
    I: Iterator<Item = &'a String>,
{
    let sub = iter
        .next()
        .ok_or("usage: dashboard list|unique|delete-category|delete-all")?;
    match sub.as_str() {
        "list" => {
            let mut pages = 1_u32;
            let mut sort = String::new();
            let mut direction = String::from("asc");
            let mut search = String::new();
            while let Some(arg) = iter.next() {
                match arg.as_str() {
                    "--pages" => {
                        if let Some(v) = iter.next()
                            && let Ok(n) = v.parse::<u32>()
                        {
                            pages = n;
                        }
                    }
                    "--sort" => {
                        if let Some(v) = iter.next() {
                            sort.clone_from(v);
                        }
                    }
                    "--direction" => {
                        if let Some(v) = iter.next() {
                            direction.clone_from(v);
                        }
                    }
                    "--search" => {
                        if let Some(v) = iter.next() {
                            search.clone_from(v);
                        }
                    }
                    _ => {}
                }
            }
            Ok(Command::DashboardList {
                pages,
                sort,
                direction,
                search,
            })
        }
        "unique" => {
            let column = iter.next().ok_or("usage: dashboard unique <column>")?;
            Ok(Command::DashboardUnique {
                column: column.clone(),
            })
        }
        "delete-category" => {
            let pair = iter
                .next()
                .ok_or("usage: dashboard delete-category <col>=<val> [--execute]")?;
            let (col, val) = pair.split_once('=').ok_or("expected column=value")?;
            let mut execute = false;
            for arg in iter {
                if arg == "--execute" {
                    execute = true;
                }
            }
            Ok(Command::DashboardDeleteCategory {
                column: col.to_owned(),
                value: val.to_owned(),
                execute,
            })
        }
        "delete-all" => {
            let mut execute = false;
            for arg in iter {
                if arg == "--execute" {
                    execute = true;
                }
            }
            Ok(Command::DashboardDeleteAll { execute })
        }
        other => Err(format!("unknown dashboard subcommand: {other}").into()),
    }
}

fn parse_input_user<'a, I>(iter: &mut Peekable<I>) -> Result<Command, DynError>
where
    I: Iterator<Item = &'a String>,
{
    let sub = iter.next().ok_or("usage: input-user list|export|report")?;
    match sub.as_str() {
        "list" => {
            let mut pages = 1_u32;
            let mut sort = String::new();
            let mut direction = String::from("asc");
            let mut search = String::new();
            while let Some(arg) = iter.next() {
                match arg.as_str() {
                    "--pages" => {
                        if let Some(v) = iter.next()
                            && let Ok(n) = v.parse::<u32>()
                        {
                            pages = n;
                        }
                    }
                    "--sort" => {
                        if let Some(v) = iter.next() {
                            sort.clone_from(v);
                        }
                    }
                    "--direction" => {
                        if let Some(v) = iter.next() {
                            direction.clone_from(v);
                        }
                    }
                    "--search" => {
                        if let Some(v) = iter.next() {
                            search.clone_from(v);
                        }
                    }
                    _ => {}
                }
            }
            Ok(Command::InputUserList {
                pages,
                sort,
                direction,
                search,
            })
        }
        "export" => {
            let date = parse_date_spec(iter)?;
            let mut out_dir = None;
            let mut force = false;
            while let Some(arg) = iter.next() {
                match arg.as_str() {
                    "--out" => {
                        if let Some(v) = iter.next() {
                            out_dir = Some(PathBuf::from(v));
                        }
                    }
                    "--force" => force = true,
                    _ => {}
                }
            }
            Ok(Command::InputUserExport {
                date,
                out_dir,
                force,
            })
        }
        "report" => {
            let mut pages = 5_u32;
            let mut by: Option<String> = None;
            let mut csv: Option<PathBuf> = None;
            while let Some(arg) = iter.next() {
                match arg.as_str() {
                    "--pages" => {
                        if let Some(v) = iter.next()
                            && let Ok(n) = v.parse::<u32>()
                        {
                            pages = n;
                        }
                    }
                    "--by" => {
                        if let Some(v) = iter.next() {
                            by = Some(v.clone());
                        }
                    }
                    "--csv" => {
                        if let Some(v) = iter.next() {
                            csv = Some(PathBuf::from(v));
                        }
                    }
                    _ => {}
                }
            }
            Ok(Command::InputUserReport { pages, by, csv })
        }
        other => Err(format!("unknown input-user subcommand: {other}").into()),
    }
}

fn parse_input_data<'a, I>(iter: &mut Peekable<I>) -> Result<Command, DynError>
where
    I: Iterator<Item = &'a String>,
{
    let sub = iter.next().ok_or("usage: input-data single|csv")?;
    match sub.as_str() {
        "single" => {
            let mut execute = false;
            let mut fields: Vec<(String, String)> = Vec::new();
            while let Some(arg) = iter.next() {
                match arg.as_str() {
                    "--execute" => execute = true,
                    other if other.starts_with("--") => {
                        let key = other.trim_start_matches("--").to_owned();
                        if let Some(v) = iter.next() {
                            fields.push((key, v.clone()));
                        }
                    }
                    _ => {}
                }
            }
            if fields.is_empty() {
                return Err("input-data single requires at least one --<field> <value>".into());
            }
            Ok(Command::InputDataSingle { fields, execute })
        }
        "csv" => {
            let file = iter
                .next()
                .ok_or("usage: input-data csv <file> [--execute]")?;
            let mut execute = false;
            for arg in iter {
                if arg == "--execute" {
                    execute = true;
                }
            }
            Ok(Command::InputDataCsv {
                file: PathBuf::from(file),
                execute,
            })
        }
        other => Err(format!("unknown input-data subcommand: {other}").into()),
    }
}

fn parse_csv<'a, I>(iter: &mut Peekable<I>) -> Result<Command, DynError>
where
    I: Iterator<Item = &'a String>,
{
    let sub = iter.next().ok_or("usage: csv check|dedup|pivot")?;
    match sub.as_str() {
        "check" => {
            let file = iter.next().ok_or("usage: csv check <file>")?;
            Ok(Command::CsvCheck {
                file: PathBuf::from(file),
            })
        }
        "dedup" => {
            let file = iter.next().ok_or("usage: csv dedup <file> --by c1,c2")?;
            let mut by: Vec<String> = Vec::new();
            let mut keep = Keep::First;
            let mut out: Option<PathBuf> = None;
            let mut force = false;
            while let Some(arg) = iter.next() {
                match arg.as_str() {
                    "--by" => {
                        if let Some(v) = iter.next() {
                            by = split_columns(v);
                        }
                    }
                    "--keep" => {
                        if let Some(v) = iter.next() {
                            keep = Keep::parse(v)?;
                        }
                    }
                    "--out" => {
                        if let Some(v) = iter.next() {
                            out = Some(PathBuf::from(v));
                        }
                    }
                    "--force" => force = true,
                    _ => {}
                }
            }
            if by.is_empty() {
                return Err("--by must contain at least one column".into());
            }
            Ok(Command::CsvDedup {
                file: PathBuf::from(file),
                by,
                keep,
                out,
                force,
            })
        }
        "pivot" => {
            let file = iter
                .next()
                .ok_or("usage: csv pivot <file> --rows c --values c")?;
            let mut rows: Vec<String> = Vec::new();
            let mut values = String::new();
            let mut agg = Agg::CountDistinct;
            let mut out: Option<PathBuf> = None;
            let mut force = false;
            while let Some(arg) = iter.next() {
                match arg.as_str() {
                    "--rows" => {
                        if let Some(v) = iter.next() {
                            rows = split_columns(v);
                        }
                    }
                    "--values" => {
                        if let Some(v) = iter.next() {
                            values.clone_from(v);
                        }
                    }
                    "--agg" => {
                        if let Some(v) = iter.next() {
                            agg = Agg::parse(v)?;
                        }
                    }
                    "--out" => {
                        if let Some(v) = iter.next() {
                            out = Some(PathBuf::from(v));
                        }
                    }
                    "--force" => force = true,
                    _ => {}
                }
            }
            if rows.is_empty() {
                return Err("--rows must contain at least one column".into());
            }
            if values.is_empty() {
                return Err("--values is required".into());
            }
            Ok(Command::CsvPivot {
                file: PathBuf::from(file),
                rows,
                values,
                agg,
                out,
                force,
            })
        }
        other => Err(format!("unknown csv subcommand: {other}").into()),
    }
}

fn parse_report<'a, I>(iter: &mut Peekable<I>) -> Result<Command, DynError>
where
    I: Iterator<Item = &'a String>,
{
    let date = parse_date_spec(iter)?;
    let mut dedup_by = vec![String::from("userID"), String::from("nopol")];
    let mut keep = Keep::First;
    let mut pivot_rows = vec![String::from("nama")];
    let mut pivot_values = String::from("nopol");
    let mut pivot_agg = Agg::CountDistinct;
    let mut out_dir = None;
    let mut force = false;
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--dedup-by" => {
                if let Some(v) = iter.next() {
                    dedup_by = split_columns(v);
                }
            }
            "--keep" => {
                if let Some(v) = iter.next() {
                    keep = Keep::parse(v)?;
                }
            }
            "--pivot-rows" => {
                if let Some(v) = iter.next() {
                    pivot_rows = split_columns(v);
                }
            }
            "--pivot-values" => {
                if let Some(v) = iter.next() {
                    pivot_values.clone_from(v);
                }
            }
            "--pivot-agg" => {
                if let Some(v) = iter.next() {
                    pivot_agg = Agg::parse(v)?;
                }
            }
            "--out" => {
                if let Some(v) = iter.next() {
                    out_dir = Some(PathBuf::from(v));
                }
            }
            "--force" => force = true,
            _ => {}
        }
    }
    Ok(Command::Report {
        date,
        dedup_by,
        keep,
        pivot_rows,
        pivot_values,
        pivot_agg,
        out_dir,
        force,
    })
}

fn parse_replace<'a, I>(iter: &mut Peekable<I>) -> Result<Command, DynError>
where
    I: Iterator<Item = &'a String>,
{
    let file = iter.next().ok_or("usage: replace <file> [--execute]")?;
    let mut execute = false;
    let mut no_backup = false;
    for arg in iter {
        match arg.as_str() {
            "--execute" => execute = true,
            "--no-backup" => no_backup = true,
            _ => {}
        }
    }
    Ok(Command::Replace {
        file: PathBuf::from(file),
        execute,
        no_backup,
    })
}

/// Consume tokens for `today`, `date <D>`, or `range <A> <B>`.
fn parse_date_spec<'a, I>(iter: &mut Peekable<I>) -> Result<DateSpec, DynError>
where
    I: Iterator<Item = &'a String>,
{
    let sub = iter
        .next()
        .ok_or("expected today | date <YYYY-MM-DD> | range <START> <END>")?;
    match sub.as_str() {
        "today" => Ok(DateSpec::Today),
        "date" => {
            let d = iter.next().ok_or("date requires a YYYY-MM-DD argument")?;
            Ok(DateSpec::Single(d.clone()))
        }
        "range" => {
            let s = iter.next().ok_or("range requires START")?;
            let e = iter.next().ok_or("range requires END")?;
            Ok(DateSpec::Range {
                start: s.clone(),
                end: e.clone(),
            })
        }
        other => Err(format!("unknown date spec: {other}").into()),
    }
}

fn usage() -> String {
    String::from(
        "usage: cekunit <command> [options]\n\
         \n\
         auth:\n\
           auth login                       log in and verify session\n\
           auth logout                      safe logout\n\
           auth info                        probe server, show session\n\
         \n\
         dashboard (server):\n\
           dashboard list [--pages N] [--sort C] [--direction D] [--search S]\n\
           dashboard unique <column>\n\
           dashboard delete-category <col>=<val> [--execute]\n\
           dashboard delete-all [--execute]\n\
         \n\
         input-user (server):\n\
           input-user list [--pages N] [--sort C] [--direction D] [--search S]\n\
           input-user export <today|date D|range A B> [--out DIR] [--force]\n\
           input-user report [--pages N] [--by COL] [--csv FILE]\n\
         \n\
         input-data (server, write):\n\
           input-data single [--execute] --<field> <value> ...\n\
           input-data csv <file> [--execute]\n\
         \n\
         csv (local):\n\
           csv check <file>\n\
           csv dedup <file> --by c1,c2 [--keep first|last] [--out FILE] [--force]\n\
           csv pivot <file> --rows c1[,c2] --values c [--agg A] [--out FILE] [--force]\n\
         \n\
         workflows:\n\
           report <today|date D|range A B> [--dedup-by c1,c2]\n\
                                            [--pivot-rows c] [--pivot-values c]\n\
                                            [--pivot-agg A] [--out DIR] [--force]\n\
           replace <file> [--execute] [--no-backup]\n",
    )
}

fn split_columns(s: &str) -> Vec<String> {
    s.split(',')
        .map(|p| p.trim().to_owned())
        .filter(|p| !p.is_empty())
        .collect()
}

// =============================================================================
// Entry point
// =============================================================================

#[tokio::main]
async fn main() -> Result<(), DynError> {
    let _ = librcekunit_client::tracing::init_tracing();

    let cmd = parse_command()?;

    // Hard safety gate for writes.
    if is_write(&cmd) && !std::io::stdin().is_terminal() {
        return Err("destructive operation requires an interactive terminal".into());
    }

    dispatch(cmd).await
}

const fn is_write(cmd: &Command) -> bool {
    matches!(
        cmd,
        Command::DashboardDeleteCategory { execute: true, .. }
            | Command::DashboardDeleteAll { execute: true }
            | Command::InputDataSingle { execute: true, .. }
            | Command::InputDataCsv { execute: true, .. }
            | Command::Replace { execute: true, .. }
    )
}

async fn dispatch(cmd: Command) -> Result<(), DynError> {
    match cmd {
        Command::AuthLogin => cmd_auth_login().await,
        Command::AuthLogout => cmd_auth_logout().await,
        Command::AuthInfo => cmd_auth_info().await,

        Command::DashboardList {
            pages,
            sort,
            direction,
            search,
        } => cmd_dashboard_list(pages, &sort, &direction, &search).await,
        Command::DashboardUnique { column } => cmd_dashboard_unique(&column).await,
        Command::DashboardDeleteCategory {
            column,
            value,
            execute,
        } => cmd_dashboard_delete_category(&column, &value, execute).await,
        Command::DashboardDeleteAll { execute } => cmd_dashboard_delete_all(execute).await,

        Command::InputUserList {
            pages,
            sort,
            direction,
            search,
        } => cmd_input_user_list(pages, &sort, &direction, &search).await,
        Command::InputUserExport {
            date,
            out_dir,
            force,
        } => cmd_input_user_export(&date, out_dir.as_deref(), force).await,
        Command::InputUserReport { pages, by, csv } => {
            cmd_input_user_report(pages, by.as_deref(), csv.as_deref()).await
        }

        Command::InputDataSingle { fields, execute } => {
            cmd_input_data_single(&fields, execute).await
        }
        Command::InputDataCsv { file, execute } => cmd_input_data_csv(&file, execute).await,

        Command::CsvCheck { file } => cmd_csv_check(&file),
        Command::CsvDedup {
            file,
            by,
            keep,
            out,
            force,
        } => cmd_csv_dedup(&file, &by, keep, out.as_deref(), force),
        Command::CsvPivot {
            file,
            rows,
            values,
            agg,
            out,
            force,
        } => cmd_csv_pivot(&file, &rows, &values, agg, out.as_deref(), force),

        Command::Report {
            date,
            dedup_by,
            keep,
            pivot_rows,
            pivot_values,
            pivot_agg,
            out_dir,
            force,
        } => {
            cmd_report(
                &date,
                &dedup_by,
                keep,
                &pivot_rows,
                &pivot_values,
                pivot_agg,
                out_dir.as_deref(),
                force,
            )
            .await
        }

        Command::Replace {
            file,
            execute,
            no_backup,
        } => cmd_replace(&file, execute, no_backup).await,
    }
}

// =============================================================================
// Auth handlers
// =============================================================================

async fn cmd_auth_login() -> Result<(), DynError> {
    let client = connect_fresh().await?;
    println!("[auth] login ok");
    let _ = client;
    println!("done");
    Ok(())
}

async fn cmd_auth_logout() -> Result<(), DynError> {
    let client = from_env().await?;
    auth::safe_logout(&client).await?;
    println!("[auth] logged out (or was never logged in)");
    println!("done");
    Ok(())
}

async fn cmd_auth_info() -> Result<(), DynError> {
    let base_url = std::env::var("LIBRCEKUNIT_BASE_URL").unwrap_or_default();
    println!("[info] base_url: {base_url}");
    let client = match from_env().await {
        Ok(c) => c,
        Err(err) => {
            eprintln!("[info] {}", error::user_message(&err));
            return Err(err.into());
        }
    };
    let probe = client.dashboard_index().await?;
    let status = probe.status();
    println!("[info] /dashboard status: {status}");
    if auth::is_authenticated_status(status.as_u16()) {
        println!("[info] session: active");
    } else {
        println!("[info] session: not active");
    }
    Ok(())
}

// =============================================================================
// Dashboard handlers
// =============================================================================

async fn cmd_dashboard_list(
    pages: u32,
    sort: &str,
    direction: &str,
    search: &str,
) -> Result<(), DynError> {
    let client = connect().await?;
    for page in 1..=pages {
        let mut params = Form::new();
        let _ = params.insert(String::from("page"), page.to_string());
        let _ = params.insert(String::from("direction"), direction.to_owned());
        if !sort.is_empty() {
            let _ = params.insert(String::from("sort"), sort.to_owned());
        }
        if !search.is_empty() {
            let _ = params.insert(String::from("search"), search.to_owned());
        }
        let resp = client.dashboard_index_with_params(params).await?;
        let status = resp.status();
        let html = resp.text().await.unwrap_or_default();
        println!("[dashboard {page}] status={status} bytes={}", html.len());
    }
    println!("done");
    Ok(())
}

async fn cmd_dashboard_unique(column: &str) -> Result<(), DynError> {
    let client = connect().await?;
    let resp = client.get_unique_values(column).await?;
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    println!("[unique] column={column} status={status}");
    println!("[unique] body: {body}");
    println!("done");
    Ok(())
}

async fn cmd_dashboard_delete_category(
    column: &str,
    value: &str,
    execute: bool,
) -> Result<(), DynError> {
    if !execute {
        println!("[delete-category] PREVIEW: would delete rows where {column} = {value}");
        println!("[delete-category] pass --execute on a terminal to proceed");
        println!("done");
        return Ok(());
    }

    let phrase = "HAPUS";
    print!("Type {phrase:?} to delete rows where {column} = {value}: ");
    let _ = std::io::stdout().flush();
    if read_line()?.trim() != phrase {
        println!("[abort] nothing was deleted");
        return Ok(());
    }

    let client = connect().await?;
    let resp = client.delete_by_category(column, value).await?;
    println!("[delete-category] status={}", resp.status());
    println!("done");
    Ok(())
}

async fn cmd_dashboard_delete_all(execute: bool) -> Result<(), DynError> {
    if !execute {
        println!("[delete-all] PREVIEW: would delete every row");
        println!("[delete-all] pass --execute on a terminal to proceed");
        println!("done");
        return Ok(());
    }

    let phrase = "HAPUS SEMUA";
    print!("Type {phrase:?} to delete every row: ");
    let _ = std::io::stdout().flush();
    if read_line()?.trim() != phrase {
        println!("[abort] nothing was deleted");
        return Ok(());
    }

    let client = connect().await?;
    let resp = client.delete_all().await?;
    println!("[delete-all] status={}", resp.status());
    println!("done");
    Ok(())
}

// =============================================================================
// Input-user handlers
// =============================================================================

async fn cmd_input_user_list(
    pages: u32,
    sort: &str,
    direction: &str,
    search: &str,
) -> Result<(), DynError> {
    let client = connect().await?;
    for page in 1..=pages {
        let mut params = Form::new();
        let _ = params.insert(String::from("page"), page.to_string());
        let _ = params.insert(String::from("direction"), direction.to_owned());
        if !sort.is_empty() {
            let _ = params.insert(String::from("sort"), sort.to_owned());
        }
        if !search.is_empty() {
            let _ = params.insert(String::from("search"), search.to_owned());
        }
        let resp = client.input_user_index_with_params(params).await?;
        let status = resp.status();
        let html = resp.text().await.unwrap_or_default();
        println!("[input-user {page}] status={status} bytes={}", html.len());
    }
    println!("done");
    Ok(())
}

async fn cmd_input_user_export(
    date: &DateSpec,
    out_dir: Option<&Path>,
    force: bool,
) -> Result<(), DynError> {
    let range = date.resolve()?;
    println!("[export] range {} .. {}", range.start, range.end);

    let dir = resolve_out_dir(out_dir)?;
    let target = dir.join(format!("cekunit_{}.raw.csv", range.label));

    if target.exists() && !force {
        return Err(format!("file exists: {}. pass --force", target.display()).into());
    }

    let bytes = download_csv(&range.start, &range.end).await?;
    std::fs::write(&target, &bytes)?;
    println!(
        "[export] wrote {} bytes to {}",
        bytes.len(),
        target.display()
    );
    println!("done");
    Ok(())
}

async fn cmd_input_user_report(
    pages: u32,
    by: Option<&str>,
    csv: Option<&Path>,
) -> Result<(), DynError> {
    let client = connect().await?;
    let opts = ScanOptions::default()
        .max_pages(pages)
        .page_delay(core::time::Duration::from_millis(150));

    let result = scan::scan_input_user_with(&client, &opts, |p| {
        let total = p
            .total_reported
            .map_or_else(|| String::from("?"), |n| n.to_string());
        println!(
            "[scan] page {}: +{} rows ({} total, server reports {})",
            p.page, p.rows_on_page, p.rows_total, total,
        );
    })
    .await?;

    println!("[scan] fetched {} pages", result.pages_fetched);
    println!("[scan] collected {} rows", result.rows.len());
    if let Some(total) = result.total_reported {
        println!("[scan] server reported total: {total}");
    }

    if let Some(column) = by {
        let counts = match column {
            "no" => parse::count_by_input_user(&result.rows, |r| r.no.clone()),
            "created_at" => parse::count_by_input_user(&result.rows, |r| r.created_at.clone()),
            "user_id" => parse::count_by_input_user(&result.rows, |r| r.user_id.clone()),
            "nopol" => parse::count_by_input_user(&result.rows, |r| r.nopol.clone()),
            "lokasi" => parse::count_by_input_user(&result.rows, |r| r.lokasi.clone()),
            "forn" => parse::count_by_input_user(&result.rows, |r| r.forn.clone()),
            "nama" => parse::count_by_input_user(&result.rows, |r| r.nama.clone()),
            "kategori" => parse::count_by_input_user(&result.rows, |r| r.kategori.clone()),
            "nama_nasabah" => parse::count_by_input_user(&result.rows, |r| r.nama_nasabah.clone()),
            "no_perjanjian" => {
                parse::count_by_input_user(&result.rows, |r| r.no_perjanjian.clone())
            }
            other => return Err(format!("unknown column: {other}").into()),
        };
        println!("[by] column={column}");
        for (label, count) in counts.iter().take(30) {
            println!("[by]   {label}: {count}");
        }
    }

    if let Some(path) = csv {
        let text = parse::input_user_rows_to_csv(&result.rows);
        std::fs::write(path, text.as_bytes())?;
        println!(
            "[csv] wrote {} rows ({} bytes) to {}",
            result.rows.len(),
            text.len(),
            path.display()
        );
    }

    println!("done");
    Ok(())
}

// =============================================================================
// Input-data handlers
// =============================================================================

async fn cmd_input_data_single(fields: &[(String, String)], execute: bool) -> Result<(), DynError> {
    let mut form = Form::new();
    for (k, v) in fields {
        let _ = form.insert(k.clone(), v.clone());
    }

    println!("[single] payload ({} field(s)):", form.len());
    let mut keys: Vec<&String> = form.keys().collect();
    keys.sort();
    for k in keys {
        if let Some(v) = form.get(k) {
            println!("[single]   {k} = {v}");
        }
    }

    if !execute {
        println!("[single] PREVIEW only; pass --execute on a terminal to proceed");
        println!("done");
        return Ok(());
    }

    let phrase = "INSERT-1";
    print!("Type {phrase:?} to insert: ");
    let _ = std::io::stdout().flush();
    if read_line()?.trim() != phrase {
        println!("[abort] nothing was inserted");
        return Ok(());
    }

    let client = connect().await?;
    let resp = client.input_data_store(form).await?;
    println!("[single] status={}", resp.status());
    println!("done");
    Ok(())
}

async fn cmd_input_data_csv(file: &Path, execute: bool) -> Result<(), DynError> {
    if !file.exists() {
        return Err(format!("file not found: {}", file.display()).into());
    }

    let text = std::fs::read_to_string(file)?;
    let rows = text.lines().filter(|l| !l.trim().is_empty()).count();
    let data_rows = rows.saturating_sub(1);

    println!("[csv] file: {}", file.display());
    println!("[csv] data rows: {data_rows}");

    if !execute {
        println!("[csv] PREVIEW only; pass --execute on a terminal to proceed");
        println!("done");
        return Ok(());
    }

    let phrase = format!("INSERT-{data_rows}");
    print!("Type {phrase:?} to upload: ");
    let _ = std::io::stdout().flush();
    if read_line()?.trim() != phrase {
        println!("[abort] nothing was uploaded");
        return Ok(());
    }

    let client = connect().await?;
    let resp = client.import_csv(file).await?;
    println!("[csv] status={}", resp.status());
    println!("done");
    Ok(())
}

// =============================================================================
// CSV handlers (local, no network)
// =============================================================================

fn cmd_csv_check(file: &Path) -> Result<(), DynError> {
    let text = std::fs::read_to_string(file)?;
    let summary = csv_check::validate(&text)
        .map_err(|e| -> DynError { format!("validation failed: {e}").into() })?;

    println!("[check] header ({} columns):", summary.header.len());
    for (i, col) in summary.header.iter().enumerate() {
        println!("[check]   {i}: {col}");
    }
    println!("[check] data rows: {}", summary.data_rows);
    println!("[check] file is well-formed");
    println!("done");
    Ok(())
}

fn cmd_csv_dedup(
    file: &Path,
    by: &[String],
    keep: Keep,
    out: Option<&Path>,
    force: bool,
) -> Result<(), DynError> {
    let text = std::fs::read_to_string(file)?;
    let result = dedupe_csv(&text, by, keep)?;

    let target = out.map_or_else(|| default_suffixed_path(file, "deduped"), Path::to_path_buf);

    if target.exists() && !force {
        return Err(format!("file exists: {}. pass --force", target.display()).into());
    }

    std::fs::write(&target, result.as_bytes())?;
    println!(
        "[dedup] wrote {} bytes to {}",
        result.len(),
        target.display()
    );
    println!("done");
    Ok(())
}

fn cmd_csv_pivot(
    file: &Path,
    rows: &[String],
    values: &str,
    agg: Agg,
    out: Option<&Path>,
    force: bool,
) -> Result<(), DynError> {
    let text = std::fs::read_to_string(file)?;
    let result = pivot_csv(&text, rows, values, agg)?;

    let target = out.map_or_else(|| default_suffixed_path(file, "pivot"), Path::to_path_buf);

    if target.exists() && !force {
        return Err(format!("file exists: {}. pass --force", target.display()).into());
    }

    std::fs::write(&target, result.as_bytes())?;
    println!(
        "[pivot] wrote {} bytes to {}",
        result.len(),
        target.display()
    );
    for line in result.lines().take(11) {
        println!("[pivot]   {line}");
    }
    println!("done");
    Ok(())
}

// =============================================================================
// Report (download + dedup + pivot)
// =============================================================================

#[allow(clippy::too_many_arguments)]
async fn cmd_report(
    date: &DateSpec,
    dedup_by: &[String],
    keep: Keep,
    pivot_rows: &[String],
    pivot_values: &str,
    pivot_agg: Agg,
    out_dir: Option<&Path>,
    force: bool,
) -> Result<(), DynError> {
    let range = date.resolve()?;
    println!("[report] range {} .. {}", range.start, range.end);

    let dir = resolve_out_dir(out_dir)?;
    let raw_path = dir.join(format!("cekunit_{}.raw.csv", range.label));
    let deduped_path = dir.join(format!("cekunit_{}.deduped.csv", range.label));
    let report_path = dir.join(format!("cekunit_{}.report.csv", range.label));

    for path in [&raw_path, &deduped_path, &report_path] {
        if path.exists() && !force {
            return Err(format!("file exists: {}. pass --force", path.display()).into());
        }
    }

    println!("[report 1/3] downloading");
    let bytes = download_csv(&range.start, &range.end).await?;
    std::fs::write(&raw_path, &bytes)?;
    println!("[report 1/3] wrote {} bytes", bytes.len());

    println!("[report 2/3] deduping");
    let text = String::from_utf8_lossy(&bytes);
    let deduped = dedupe_csv(&text, dedup_by, keep)?;
    std::fs::write(&deduped_path, deduped.as_bytes())?;
    println!("[report 2/3] wrote {} bytes", deduped.len());

    println!("[report 3/3] pivoting");
    let report = pivot_csv(&deduped, pivot_rows, pivot_values, pivot_agg)?;
    std::fs::write(&report_path, report.as_bytes())?;
    println!("[report 3/3] wrote {} bytes", report.len());

    println!();
    println!("raw:      {}", raw_path.display());
    println!("deduped:  {}", deduped_path.display());
    println!("report:   {}", report_path.display());

    for line in report.lines().take(11) {
        println!("[preview] {line}");
    }
    println!("done");
    Ok(())
}

// =============================================================================
// Replace (validate + backup + delete + upload)
// =============================================================================

async fn cmd_replace(file: &Path, execute: bool, no_backup: bool) -> Result<(), DynError> {
    if !file.exists() {
        return Err(format!("file not found: {}", file.display()).into());
    }

    let text = std::fs::read_to_string(file)?;
    let summary = csv_check::validate(&text)
        .map_err(|e| -> DynError { format!("validation failed: {e}").into() })?;

    println!("[replace] file: {}", file.display());
    println!("[replace] data rows: {}", summary.data_rows);
    println!("[replace] header cols: {}", summary.header.len());

    if !execute {
        println!("[replace] PREVIEW only; pass --execute on a terminal to proceed");
        println!("done");
        return Ok(());
    }

    // Backup.
    let backup_path = if no_backup {
        println!("[replace] backup SKIPPED (--no-backup)");
        None
    } else {
        let path = default_backup_path();
        let client = connect().await?;
        println!("[replace] backing up first 5 pages to {}", path.display());
        let scan_opts = ScanOptions::default().max_pages(5);
        let scan = scan::scan_all(&client, &scan_opts).await?;
        let csv = parse::rows_to_csv(&scan.rows);
        std::fs::write(&path, csv.as_bytes())?;
        println!("[replace] backup wrote {} rows", scan.rows.len());
        Some(path)
    };

    // Confirm.
    let phrase = format!("REPLACE-{}", summary.data_rows);
    print!("Type {phrase:?} to proceed: ");
    let _ = std::io::stdout().flush();
    if read_line()?.trim() != phrase {
        println!("[abort] nothing was changed");
        if let Some(p) = backup_path {
            println!("[note] backup is at {}", p.display());
        }
        return Ok(());
    }

    // Execute.
    let client = connect().await?;
    println!("[replace] delete-all");
    let resp = client.delete_all().await?;
    println!("[replace] delete status={}", resp.status());

    println!("[replace] uploading {}", file.display());
    let resp = client.import_csv(file).await?;
    println!("[replace] upload status={}", resp.status());

    if let Some(p) = backup_path {
        println!("[replace] previous data backed up at {}", p.display());
    }
    println!("done");
    Ok(())
}

// =============================================================================
// Shared helpers
// =============================================================================

/// Build a client and ensure an authenticated session.
async fn connect() -> Result<Client, DynError> {
    let client = from_env().await?;
    let probe = client.dashboard_index().await?;
    if auth::is_authenticated_status(probe.status().as_u16()) {
        println!("[session] reused existing session");
        return Ok(client);
    }
    println!("[session] logging in");
    login_from_env(&client).await?;
    println!("[session] login ok");
    Ok(client)
}

/// Force a fresh login.
async fn connect_fresh() -> Result<Client, DynError> {
    let client = from_env().await?;
    login_from_env(&client).await?;
    Ok(client)
}

/// Download a CSV range, streaming the body.
async fn download_csv(start_date: &str, end_date: &str) -> Result<Vec<u8>, DynError> {
    let client = connect().await?;

    let mut params = Form::new();
    let _ = params.insert(String::from("format"), String::from("csv"));
    let _ = params.insert(String::from("sort"), String::from("created_at"));
    let _ = params.insert(String::from("direction"), String::from("asc"));
    let _ = params.insert(String::from("start_date"), start_date.to_owned());
    let _ = params.insert(String::from("end_date"), end_date.to_owned());
    let _ = params.insert(String::from("search"), String::new());

    let mut resp = client.input_user_export(params).await?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        let n = body.len().min(300);
        let preview = body.get(..n).unwrap_or(&body);
        return Err(format!("server returned {status}; preview: {preview}").into());
    }

    let mut bytes: Vec<u8> = Vec::new();
    let mut chunks: u64 = 0;
    let mut last_mb: usize = 0;

    while let Some(chunk) = resp
        .chunk()
        .await
        .map_err(|e| -> DynError { format!("read chunk: {e}").into() })?
    {
        bytes.extend_from_slice(&chunk);
        chunks = chunks.saturating_add(1);
        let mb = bytes.len() / (1024 * 1024);
        if mb > last_mb {
            last_mb = mb;
            println!("[download] {mb} MiB ({} bytes)", bytes.len());
        }
    }

    println!(
        "[download] done: {} bytes in {} chunks",
        bytes.len(),
        chunks
    );

    if bytes.is_empty() {
        return Err("server returned an empty body".into());
    }
    Ok(bytes)
}

/// Dedupe a CSV text, returning CSV text.
fn dedupe_csv(text: &str, by: &[String], keep: Keep) -> Result<String, DynError> {
    let mut records = csv_check::parse_records(text)?;
    if records.is_empty() {
        return Err("input CSV is empty".into());
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

        match keep {
            Keep::First => {
                if let Entry::Vacant(e) = index_of.entry(key) {
                    let _ = e.insert(out.len());
                    out.push(row);
                }
            }
            Keep::Last => {
                if let Some(&prev) = index_of.get(&key) {
                    if let Some(slot) = out.get_mut(prev) {
                        slot.clone_from(&row);
                    }
                } else {
                    let _ = index_of.insert(key, out.len());
                    out.push(row);
                }
            }
        }
    }

    println!(
        "[dedup] {input_rows} input rows -> {} unique rows",
        out.len()
    );

    let mut result = Vec::with_capacity(out.len().saturating_add(1));
    result.push(header);
    result.extend(out);
    Ok(records_to_csv(&result))
}

/// Pivot a CSV text, returning CSV text.
fn pivot_csv(text: &str, rows: &[String], values: &str, agg: Agg) -> Result<String, DynError> {
    let mut records = csv_check::parse_records(text)?;
    if records.is_empty() {
        return Err("input CSV is empty".into());
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
            Agg::Count => {}
            Agg::CountDistinct => {
                let _ = bucket.distinct.insert(value);
            }
            Agg::Sum => {
                if let Ok(n) = value.trim().parse::<f64>() {
                    bucket.sum += n;
                }
            }
        }
    }

    let mut rows_out: Vec<(Vec<String>, f64)> = buckets
        .into_iter()
        .map(|(key, b)| {
            let value = match agg {
                Agg::Count => {
                    #[allow(clippy::cast_precision_loss)]
                    {
                        b.count as f64
                    }
                }
                Agg::CountDistinct => {
                    #[allow(clippy::cast_precision_loss)]
                    {
                        b.distinct.len() as f64
                    }
                }
                Agg::Sum => b.sum,
            };
            (key, value)
        })
        .collect();

    rows_out.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(core::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });

    let mut out = String::new();
    let mut header_parts: Vec<String> = rows.to_vec();
    header_parts.push(format!("{}_{}", agg.header_suffix(), values));
    out.push_str(
        &header_parts
            .iter()
            .map(|h| escape_field(h))
            .collect::<Vec<_>>()
            .join(","),
    );
    out.push('\n');

    for (key, value) in rows_out {
        let mut parts = key;
        parts.push(fmt_value(value));
        out.push_str(
            &parts
                .iter()
                .map(|f| escape_field(f))
                .collect::<Vec<_>>()
                .join(","),
        );
        out.push('\n');
    }

    Ok(out)
}

fn resolve_indices(header: &[String], names: &[String]) -> Result<Vec<usize>, DynError> {
    let mut indices = Vec::with_capacity(names.len());
    for name in names {
        if let Some(i) = header.iter().position(|h| h == name) {
            indices.push(i);
        } else {
            eprintln!("[error] column not found: {name}");
            eprintln!("[error] available: {}", header.join(", "));
            return Err(format!("column not found: {name}").into());
        }
    }
    Ok(indices)
}

fn resolve_single_index(header: &[String], name: &str) -> Result<usize, DynError> {
    header.iter().position(|h| h == name).ok_or_else(|| {
        eprintln!("[error] column not found: {name}");
        eprintln!("[error] available: {}", header.join(", "));
        format!("column not found: {name}").into()
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

fn default_suffixed_path(input: &Path, suffix: &str) -> PathBuf {
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let filename = format!("{stem}.{suffix}.csv");
    match input.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir.join(filename),
        _ => PathBuf::from(filename),
    }
}

fn resolve_out_dir(out_dir: Option<&Path>) -> Result<PathBuf, DynError> {
    let dir = match out_dir {
        Some(d) => d.to_path_buf(),
        None => default_downloads_dir()?,
    };
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
        println!("[fs] created directory {}", dir.display());
    }
    Ok(dir)
}

fn default_downloads_dir() -> Result<PathBuf, DynError> {
    let home = std::env::var("HOME").map_err(|e| -> DynError { format!("$HOME: {e}").into() })?;
    Ok(Path::new(&home).join("Downloads"))
}

fn default_backup_path() -> PathBuf {
    let epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    PathBuf::from(format!("pre_replace_backup_{epoch}.csv"))
}

fn read_line() -> Result<String, DynError> {
    let mut buffer = String::new();
    let _: usize = std::io::stdin()
        .read_line(&mut buffer)
        .map_err(|e| -> DynError { format!("read stdin: {e}").into() })?;
    Ok(buffer)
}

fn today_local() -> Result<String, DynError> {
    let output = std::process::Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .map_err(|e| -> DynError { format!("run date: {e}").into() })?;
    if !output.status.success() {
        return Err("date command failed".into());
    }
    let raw = String::from_utf8(output.stdout)
        .map_err(|e| -> DynError { format!("date output: {e}").into() })?;
    Ok(raw.trim().to_string())
}

fn validate_date(date: &str) -> Result<(), DynError> {
    let bytes = date.as_bytes();
    if bytes.len() != 10 {
        return Err(format!("date must be YYYY-MM-DD, got {date:?}").into());
    }
    if bytes.get(4) != Some(&b'-') || bytes.get(7) != Some(&b'-') {
        return Err(format!("date must be YYYY-MM-DD, got {date:?}").into());
    }
    for (i, b) in bytes.iter().enumerate() {
        if i == 4 || i == 7 {
            continue;
        }
        if !b.is_ascii_digit() {
            return Err(format!("date must be YYYY-MM-DD, got {date:?}").into());
        }
    }
    Ok(())
}
