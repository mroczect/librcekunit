#![allow(unused_crate_dependencies)]
//! # Full login workflow
//!
//! This example walks through every step of an authenticated session
//! against a Cek Unit Laravel backend, from building the HTTP client
//! to logging out safely.
//!
//! ## What you will learn
//!
//! 1. How to initialize tracing so debug logs are visible.
//! 2. How to build a `Client` from environment variables.
//! 3. How to read credentials from the environment without panicking.
//! 4. How to log in and confirm the session is actually active.
//! 5. How to classify errors and produce user-facing messages.
//! 6. How to log out safely, tolerating sessions that were never
//!    established.
//!
//! ## Environment variables
//!
//! | Name                      | Required | Purpose                                       |
//! |---------------------------|----------|-----------------------------------------------|
//! | `LIBRCEKUNIT_BASE_URL`    | yes      | Root URL of the Laravel backend.              |
//! | `LIBRCEKUNIT_EMAIL`       | yes      | Login email address.                          |
//! | `LIBRCEKUNIT_PASSWORD`    | yes      | Login password.                               |
//! | `LIBRCEKUNIT_COOKIE_FILE` | no       | When set, cookies persist to this file.       |
//! | `LIBRCEKUNIT_USER_AGENT`  | no       | Overrides the default user agent string.      |
//! | `LIBRCEKUNIT_TIMEOUT_SECS`| no       | Overrides the default 30 second timeout.      |
//! | `RUST_LOG`                | no       | Tracing filter. Defaults to `info` when unset.|
//!
//! ## Running
//!
//! ```sh
//! export LIBRCEKUNIT_BASE_URL=http://your_server
//! export LIBRCEKUNIT_EMAIL=user@example.com
//! export LIBRCEKUNIT_PASSWORD=secret
//! cargo run -p librcekunit_examples --example 01_full_login
//! ```
//!
//! ## Seeing raw HTTP traffic
//!
//! ```sh
//! export RUST_LOG=librcekunit_api=debug,librcekunit_examples=info
//! ```
//!
//! With this filter, the example prints the same user-facing lines but
//! `librcekunit_api` additionally logs every request and response.

// The example uses `use librcekunit_examples::prelude::*;` which brings
// many items into scope. Some of them are not referenced directly in
// this file, so the wildcard import lint is silenced here rather than
// in every example.
#![allow(clippy::wildcard_imports)]

// `auth` provides higher-level helpers such as `safe_logout` and the
// status classification used to detect an unauthenticated redirect.
use librcekunit_examples::auth;
// `error` provides `classify`, `is_retryable`, and `user_message`.
// This example only uses `user_message`, but the module is imported
// whole so the reader can see where the helpers live.
use librcekunit_examples::error;
// The prelude re-exports `Client`, `ClientBuilder`, `CookieStore`,
// `Error`, `Result`, `Form`, and every handler trait. It also
// re-exports the environment helpers `from_env` and `login_from_env`.
use librcekunit_examples::prelude::*;

/// Boxed, thread-safe error type used as the return of `main`.
///
/// Using a boxed trait object lets the example mix several error
/// sources: `librcekunit_client::Error`, `std::env::VarError`, and
/// ad-hoc string errors such as "session not authenticated".
type DynError = Box<dyn core::error::Error + Send + Sync>;

/// Entry point.
///
/// Every step prints a bracketed tag such as `[login]` or `[verify]` so
/// the output can be scanned quickly. Fatal errors are printed before
/// returning so the caller sees a friendly message even when the
/// process exits with a non-zero status.
#[tokio::main]
async fn main() -> Result<(), DynError> {
    // Step 0: tracing.
    //
    // `init_tracing` installs a global subscriber. Calling it more than
    // once in the same process returns an error, which we ignore with
    // `let _ =` because tracing is optional for this example. When
    // `RUST_LOG` is unset, the subscriber falls back to the `info`
    // level, which is why the `tracing::info!` calls below are visible
    // by default.
    let _ = librcekunit_client::tracing::init_tracing();

    // Step 1: build the client.
    //
    // `from_env` reads `LIBRCEKUNIT_BASE_URL`, `LIBRCEKUNIT_TIMEOUT_SECS`,
    // `LIBRCEKUNIT_USER_AGENT`, and `LIBRCEKUNIT_COOKIE_FILE`. It builds
    // a `Client` whose base URL points at the backend. When the cookie
    // file variable is set, the client persists cookies to that file
    // after every request and reloads them at construction, so a second
    // run of this example can skip the login step entirely.
    tracing::info!("building client from environment");
    let client = match from_env().await {
        Ok(c) => c,
        Err(err) => {
            eprintln!("[config] {}", error::user_message(&err));
            return Err(err.into());
        }
    };
    tracing::info!("client ready");

    // Step 2: read credentials.
    //
    // `env_required` wraps `std::env::var` and converts both a missing
    // variable and a non-UTF-8 value into `DynError`. This keeps the
    // example free of `unwrap` and `expect`, matching the workspace
    // lint policy.
    let email = env_required("LIBRCEKUNIT_EMAIL")?;
    let password = env_required("LIBRCEKUNIT_PASSWORD")?;

    // Step 3: log in.
    //
    // Under the hood, `login` performs three HTTP requests:
    //
    // 1. `GET /login` to fetch a CSRF token from the login form.
    // 2. `POST /login` with `_token`, `email`, and `password`.
    // 3. A redirect follow on success. The cached CSRF token is
    //    cleared because the session token changes after login.
    //
    // If the server returns a non-2xx and non-3xx status, `login`
    // returns `Error::Api(status, body)`.
    tracing::info!(%email, "logging in");
    if let Err(err) = client.login(&email, &password).await {
        eprintln!("[login] {}", error::user_message(&err));
        return Err(err.into());
    }
    println!("[login] ok as {email}");

    // Step 4: verify the session.
    //
    // A 200 response from `login` only means the server accepted the
    // credentials. To confirm the session is actually active, we hit a
    // protected endpoint. Laravel redirects unauthenticated requests
    // to `/login`, which we detect with `is_authenticated_status`.
    tracing::info!("verifying session via /dashboard");
    let resp = match client.dashboard_index().await {
        Ok(r) => r,
        Err(err) => {
            eprintln!("[verify] {}", error::user_message(&err));
            return Err(err.into());
        }
    };

    let status = resp.status();
    println!("[verify] dashboard status = {status}");

    if !auth::is_authenticated_status(status.as_u16()) {
        eprintln!("[verify] session not authenticated, status = {status}");
        return Err("session not authenticated".into());
    }
    println!("[verify] session confirmed");

    // Step 5: a representative read call.
    //
    // `index` performs `GET /cekunit` and returns the raw
    // `reqwest::Response`. We read the body as text and print its
    // length. This is the same pattern every other read endpoint
    // follows.
    tracing::info!("fetching cek unit list");
    match client.index().await {
        Ok(list) => {
            let html = list.text().await.unwrap_or_default();
            println!("[list] received {} bytes of HTML", html.len());
        }
        Err(err) => {
            // The list call is not fatal for this example; we print and
            // continue so the logout step still runs.
            eprintln!("[list] {}", error::user_message(&err));
        }
    }

    // Step 6: log out.
    //
    // `safe_logout` calls `client.logout()` and treats
    // `Error::NotLoggedIn` as success, so it is safe to call even when
    // the session was never established. The CSRF cache is cleared and
    // the cookie file, if any, is deleted.
    tracing::info!("logging out");
    match auth::safe_logout(&client).await {
        Ok(()) => println!("[logout] ok"),
        Err(err) => {
            eprintln!("[logout] {}", error::user_message(&err));
            return Err(err.into());
        }
    }

    println!("done");
    Ok(())
}

/// Read an environment variable or return a descriptive error.
///
/// # Errors
///
/// Returns a boxed error when the variable is missing or contains
/// invalid UTF-8.
///
/// # Why not `unwrap`
///
/// The workspace forbids `unwrap` and `expect`. Returning a `Result`
/// keeps the example compliant and gives the reader a model for their
/// own code.
fn env_required(key: &str) -> Result<String, DynError> {
    std::env::var(key).map_err(|e| format!("{key}: {e}").into())
}
