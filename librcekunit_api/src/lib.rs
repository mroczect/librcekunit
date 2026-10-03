#![forbid(unsafe_code)]
#![deny(
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub,
    unused_qualifications
)]
#![allow(clippy::multiple_crate_versions)]

#[cfg(test)]
use wiremock as _;

pub mod client;
pub mod controller;
pub mod cookies;
pub mod csrf;
pub mod endpoints;
pub mod http_client;

pub use client::Client;
pub use http_client::HttpClient;
