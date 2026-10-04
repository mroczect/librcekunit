#![allow(unused_crate_dependencies)]
//! # Reuse session across runs
//!
//! Demonstrates persistent cookies. The first run logs in and saves
//! the session to disk. Later runs detect the saved session and skip
//! the login step entirely.
//!
//! ## What you will learn
//!
//! 1. How to enable persistent cookies via `LIBRCEKUNIT_COOKIE_FILE`.
//! 2. How to probe an existing session without sending credentials.
//! 3. How to fall back to login only when the session is invalid.
//! 4. How to interpret a redirect response as "not authenticated".
//!
//! ## Environment variables
//!
//! | Name                      | Required | Purpose                                        |
//! |---------------------------|----------|------------------------------------------------|
//! | `LIBRCEKUNIT_BASE_URL`    | yes      | Root URL of the Laravel backend.               |
//! | `LIBRCEKUNIT_COOKIE_FILE` | yes      | Path to the persisted cookie jar.              |
//! | `LIBRCEKUNIT_EMAIL`       | only on first run | Login email address.                  |
//! | `LIBRCEKUNIT_PASSWORD`    | only on first run | Login password.                       |
//! | `RUST_LOG`                | no       | Tracing filter. Defaults to `info`.            |
//!
//! ## Running
//!
//! ```sh
//! export LIBRCEKUNIT_BASE_URL=http://103.31.38.200
//! export LIBRCEKUNIT_COOKIE_FILE=session.json
//! export LIBRCEKUNIT_EMAIL=user@example.com
//! export LIBRCEKUNIT_PASSWORD=secret
//!
//! # First run: logs in and writes session.json.
//! cargo run -p librcekunit_examples --example 02_reuse_session
//!
//! # Second run: reuses session.json, no credentials sent.
//! cargo run -p librcekunit_examples --example 02_reuse_session
//! ```
//!
//! ## Forcing a fresh login
//!
//! Delete the cookie file and run again:
//!
//! ```sh
//! rm session.json
//! cargo run -p librcekunit_examples --example 02_reuse_session
//! ```
//!
//! ## What gets saved
//!
//! The cookie file contains a JSON array of raw cookie header values,
//! for example:
//!
//! ```json
//! [
//!   "XSRF-TOKEN=...",
//!   "laravel_session=..."
//! ]
//! ```
//!
//! Treat this file as sensitive. It is equivalent to a password for
//! the duration of the session.

#![allow(clippy::wildcard_imports)]

use librcekunit_examples::auth;
use librcekunit_examples::error;
use librcekunit_examples::prelude::*;

/// Boxed, thread-safe error type used as the return of `main`.
type DynError = Box<dyn core::error::Error + Send + Sync>;

/// Entry point.
///
/// The flow is intentionally linear:
///
/// 1. Build the client with a persistent cookie jar.
/// 2. Probe `/cekunit`. A redirect means no valid session.
/// 3. If unauthenticated, log in and probe again.
/// 4. Print a summary.
///
/// No logout is performed at the end. That is the point of this
/// example: leave the cookie file in place so the next run can reuse
/// it.
#[tokio::main]
async fn main() -> Result<(), DynError> {
    // Tracing is optional. Ignore the error when a subscriber is
    // already installed.
    let _ = librcekunit_client::tracing::init_tracing();

    // Step 1: build the client.
    //
    // `from_env` reads `LIBRCEKUNIT_COOKIE_FILE`. When set, the client
    // loads cookies from that path at construction and writes them
    // back after every request. When unset, cookies stay in memory and
    // the example degenerates into a plain login flow.
    tracing::info!("building client from environment");
    let client = match from_env().await {
        Ok(c) => c,
        Err(err) => {
            eprintln!("[config] {}", error::user_message(&err));
            return Err(err.into());
        }
    };

    let cookie_file = std::env::var("LIBRCEKUNIT_COOKIE_FILE").unwrap_or_else(|_| String::new());
    if cookie_file.is_empty() {
        eprintln!("[config] LIBRCEKUNIT_COOKIE_FILE is not set");
        eprintln!("[config] this example requires persistent cookies to work");
        return Err("LIBRCEKUNIT_COOKIE_FILE is not set".into());
    }
    println!("[config] cookie file: {cookie_file}");

    // Step 2: probe the session.
    //
    // `index` performs `GET /cekunit`, a protected endpoint. When the
    // client has no valid session, Laravel answers with a 302 redirect
    // to `/login`. We treat any redirect as "not authenticated".
    tracing::info!("probing session via /cekunit");
    let probe = match client.index().await {
        Ok(r) => r,
        Err(err) => {
            eprintln!("[probe] {}", error::user_message(&err));
            return Err(err.into());
        }
    };

    let probe_status = probe.status();
    println!("[probe] /cekunit status = {probe_status}");

    // Step 3: if we are not authenticated, log in.
    //
    // `is_authenticated_status` returns false for 3xx and 4xx. Any
    // other status means we already have a working session, which is
    // the whole point of this example.
    if auth::is_authenticated_status(probe_status.as_u16()) {
        println!("[session] reused existing session from {cookie_file}");
    } else {
        println!("[session] no valid session, logging in");

        let email = env_required("LIBRCEKUNIT_EMAIL")?;
        let password = env_required("LIBRCEKUNIT_PASSWORD")?;

        tracing::info!(%email, "logging in");
        if let Err(err) = client.login(&email, &password).await {
            eprintln!("[login] {}", error::user_message(&err));
            return Err(err.into());
        }
        println!("[login] ok as {email}");

        // Confirm the new session by probing once more. This double
        // check catches the rare case where `login` returns success but
        // the server did not actually establish a session.
        let after = client.index().await?;
        let after_status = after.status();
        println!("[probe] /cekunit status after login = {after_status}");

        if !auth::is_authenticated_status(after_status.as_u16()) {
            eprintln!("[session] login appeared to succeed but the probe still failed");
            return Err("session still not authenticated after login".into());
        }

        println!("[session] new session saved to {cookie_file}");
    }

    println!("done");
    println!("[hint] run this example again to reuse the saved session");
    Ok(())
}

/// Read an environment variable or return a descriptive error.
///
/// # Errors
///
/// Returns a boxed error when the variable is missing or contains
/// invalid UTF-8.
fn env_required(key: &str) -> Result<String, DynError> {
    std::env::var(key).map_err(|e| format!("{key}: {e}").into())
}
