#![forbid(unsafe_code)]
#![deny(
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub,
    unused_qualifications
)]
#![allow(clippy::multiple_crate_versions)]

use tracing as _;

pub mod auth;
pub mod config;
pub mod csv_check;
pub mod error;
pub mod form;
pub mod input_user;
pub mod logging;
pub mod parse;
pub mod prelude;
pub mod scan;
