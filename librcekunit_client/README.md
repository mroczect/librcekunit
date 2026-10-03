# librcekunit_client

Ergonomic facade for the librcekunit workspace. Wraps `librcekunit_api` with a builder, environment loading, and tracing setup. This is the recommended entry point for applications.

The crate does not implement HTTP itself. It delegates to `librcekunit_api`, which in turn implements the trait contracts defined in `librcekunit_handler`.

The crate follows the workspace-wide strict rules:

- `#![forbid(unsafe_code)]`
- No `unwrap`, no `expect`, no `panic`, no `todo`, no `unreachable`.
- No emoji, no decorative comment, no banner comment.
- Every public item documented.
- Every function returns `Result` or `Option` explicitly.
- No wildcard enum match arm.
- No arithmetic side effect.
- No indexing slicing.
- No unused qualification.

## Table of contents

1. [Installation](#installation)
2. [Quick start](#quick-start)
3. [Crate layout](#crate-layout)
4. [Module: builder](#module-builder)
5. [Module: env](#module-env)
6. [Module: tracing](#module-tracing)
7. [Prelude](#prelude)
8. [Re-exports](#re-exports)
9. [Common workflows](#common-workflows)
10. [Error handling](#error-handling)
11. [Testing](#testing)
12. [Coverage](#coverage)
13. [Linting](#linting)
14. [Design notes](#design-notes)
15. [FAQ](#faq)
16. [License](#license)

## Installation

Add the crate to `Cargo.toml`:

```toml
[dependencies]
librcekunit_client = { git = "https://codeberg.org/mroczect/librcekunit" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Or, when published:

```toml
[dependencies]
librcekunit_client = "3.0.0"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Rust version: 1.96 or newer. Edition: 2024.

## Quick start

```rust
use librcekunit_client::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClientBuilder::new()
        .base_url("https://campus.example.edu")
        .timeout_secs(30)
        .user_agent("my-app/1.0")
        .cookie_store(CookieStore::Persistent("cookies.json".into()))
        .build()
        .await?;

    client.login("user@example.com", "secret").await?;

    let response = client.index().await?;
    println!("status: {}", response.status());

    client.logout().await?;

    Ok(())
}
```

## Crate layout

```text
librcekunit_client/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs
│   ├── builder.rs
│   ├── env.rs
│   └── tracing.rs
└── tests/
    ├── builder_tests.rs
    ├── env_tests.rs
    └── integration_tests.rs
```

Every module is public. The `prelude` module re-exports the most commonly used items.

## Module: builder

Defines the `ClientBuilder` struct and its associated methods.

### Constant

```rust
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;
```

### Struct: ClientBuilder

```rust
pub struct ClientBuilder {
    base_url: Option<String>,
    timeout_secs: u64,
    user_agent: Option<String>,
    cookie_store: CookieStore,
}
```

Derives `Debug`, `Clone`.

Fields are private. Use the chainable methods to configure the builder.

### Methods

```rust
pub const fn new() -> Self
pub fn base_url(mut self, url: impl Into<String>) -> Self
pub const fn timeout_secs(mut self, secs: u64) -> Self
pub fn user_agent(mut self, ua: impl Into<String>) -> Self
pub fn cookie_store(mut self, store: CookieStore) -> Self
pub fn build_config(self) -> Result<Config>
pub async fn build(self) -> Result<Client>
```

`new` creates a builder with default values. `base_url` sets the target server URL. `timeout_secs` sets the request timeout in seconds. `user_agent` sets the user agent header value. `cookie_store` sets the cookie persistence strategy.

`build_config` returns a `Config` without creating an HTTP client. It is synchronous and does not require a Tokio runtime.

`build` creates a `Client` by constructing the underlying HTTP client. It is asynchronous and requires a Tokio runtime.

### Default impl

```rust
impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}
```

### Defaults

| Field          | Default                              |
| -------------- | ------------------------------------ |
| `base_url`     | none, must be set before build       |
| `timeout_secs` | 30                                   |
| `user_agent`   | handler default, `"librcekunit/3.0"` |
| `cookie_store` | `CookieStore::Memory`                |

### Example

```rust
use librcekunit_client::{ClientBuilder, CookieStore};

let config = ClientBuilder::new()
    .base_url("https://campus.example.edu")
    .timeout_secs(60)
    .user_agent("my-app/1.0")
    .cookie_store(CookieStore::Memory)
    .build_config()?;

assert_eq!(config.base_url, "https://campus.example.edu");
assert_eq!(config.timeout.as_secs(), 60);
```

### Error cases

`build_config` returns `Error::Config` when `base_url` is not set.

`build` returns `Error::Config` when `base_url` is not set, and `Error::Network` when the underlying `reqwest::Client` cannot be constructed.

### Notes

Calls to `base_url`, `timeout_secs`, and `user_agent` overwrite previous values. The last value wins.

The builder is cheap to clone. Each clone is independent.

## Module: env

Defines functions for loading a client from environment variables.

### Constants

```rust
pub const ENV_BASE_URL: &str = "LIBRCEKUNIT_BASE_URL";
pub const ENV_COOKIE_FILE: &str = "LIBRCEKUNIT_COOKIE_FILE";
pub const ENV_USER_AGENT: &str = "LIBRCEKUNIT_USER_AGENT";
pub const ENV_TIMEOUT_SECS: &str = "LIBRCEKUNIT_TIMEOUT_SECS";
pub const ENV_EMAIL: &str = "LIBRCEKUNIT_EMAIL";
pub const ENV_PASSWORD: &str = "LIBRCEKUNIT_PASSWORD";
```

### Functions

```rust
pub async fn from_env() -> Result<Client>
pub async fn from_env_with<F>(lookup: F) -> Result<Client>
where
    F: Fn(&str) -> Option<String>
```

Constructs a `Client` from environment variables. The `_with` variant accepts a lookup closure, which allows tests to provide deterministic values without mutating the process environment.

Reads:

- `LIBRCEKUNIT_BASE_URL` — required.
- `LIBRCEKUNIT_COOKIE_FILE` — optional. When set, uses `CookieStore::Persistent` with the given path.
- `LIBRCEKUNIT_USER_AGENT` — optional. When set, overrides the default user agent.
- `LIBRCEKUNIT_TIMEOUT_SECS` — optional. When set, must parse to `u64`.

### Functions

```rust
pub async fn login_from_env(client: &Client) -> Result<()>
pub async fn login_from_env_with<F>(client: &Client, lookup: F) -> Result<()>
where
    F: Fn(&str) -> Option<String>
```

Logs in using credentials read from environment variables. The `_with` variant accepts a lookup closure.

Reads:

- `LIBRCEKUNIT_EMAIL` — required.
- `LIBRCEKUNIT_PASSWORD` — required.

### Example using real environment

```rust
use librcekunit_client::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("LIBRCEKUNIT_BASE_URL", "https://campus.example.edu");
    std::env::set_var("LIBRCEKUNIT_EMAIL", "user@example.com");
    std::env::set_var("LIBRCEKUNIT_PASSWORD", "secret");

    let client = from_env().await?;
    login_from_env(&client).await?;

    Ok(())
}
```

Note: `std::env::set_var` is unsafe in edition 2024 in multi-threaded contexts. Prefer the `_with` variants in tests.

### Example using injectable lookup

```rust
use librcekunit_client::prelude::*;

let entries = [
    ("LIBRCEKUNIT_BASE_URL", "https://campus.example.edu"),
    ("LIBRCEKUNIT_TIMEOUT_SECS", "20"),
];

let lookup = |key: &str| {
    entries
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| String::from(*v))
};

let client = from_env_with(lookup).await?;
```

### Error cases

`from_env` and `from_env_with` return `Error::Config` when `LIBRCEKUNIT_BASE_URL` is missing or when `LIBRCEKUNIT_TIMEOUT_SECS` cannot be parsed as `u64`.

`login_from_env` and `login_from_env_with` return `Error::Config` when `LIBRCEKUNIT_EMAIL` or `LIBRCEKUNIT_PASSWORD` is missing.

## Module: tracing

Defines functions for setting up the tracing subscriber.

### Type alias

```rust
pub type InitResult = Result<(), Box<dyn core::error::Error + Send + Sync>>;
```

### Functions

```rust
pub fn init_tracing() -> InitResult
pub fn init_tracing_with_filter(filter: &str) -> InitResult
```

`init_tracing` reads from the `RUST_LOG` environment variable and falls back to `"info"` when the variable is not set or cannot be parsed.

`init_tracing_with_filter` accepts a filter string directly.

Both functions delegate to `tracing_subscriber::fmt` with an `EnvFilter`.

### Example

```rust
use librcekunit_client::tracing::init_tracing;

fn main() -> Result<(), Box<dyn core::error::Error + Send + Sync>> {
    init_tracing()?;
    println!("tracing initialized");
    Ok(())
}
```

### Error cases

Both functions return `Err` when a global tracing subscriber is already installed.

## Prelude

The prelude module re-exports the most commonly used items. Import the prelude to bring everything into scope at once.

```rust
use librcekunit_client::prelude::*;
```

Contents:

- `ClientBuilder` — builder for creating a client.
- Environment functions and constants from the `env` module.
- `Client` — the concrete HTTP client.
- `Config` — the configuration struct.
- `CookieStore` — the cookie persistence strategy enum.
- `Error` — the error type.
- `Result` — the result type alias.
- `Form` — the form data type alias.
- Traits: `Auth`, `Crud`, `Dashboard`, `InputData`, `InputUser`, `Pic`, `Users`.

## Re-exports

The crate root re-exports the following items:

From `librcekunit_api`:

- `Client`
- `HttpClient`

From `librcekunit_handler`:

- `ApiResponse`
- `Auth`
- `CekUnitId`
- `Config`
- `CookieStore`
- `Crud`
- `Dashboard`
- `Error`
- `Form`
- `HttpMethod`
- `InputData`
- `InputUser`
- `InputUserId`
- `Mode`
- `Pic`
- `PicId`
- `Result`
- `State`
- `Transport`
- `UserId`
- `Users`

Also re-exported at the crate root:

- `ClientBuilder` from the `builder` module.

## Common workflows

### Create a client with an inline configuration

```rust
use librcekunit_client::prelude::*;

let client = ClientBuilder::new()
    .base_url("https://campus.example.edu")
    .timeout_secs(60)
    .user_agent("my-app/1.0")
    .cookie_store(CookieStore::Persistent("session.json".into()))
    .build()
    .await?;
```

### Create a client from environment variables

```rust
use librcekunit_client::prelude::*;

let client = from_env().await?;
login_from_env(&client).await?;
```

### Log in and list Cek Unit records

```rust
use librcekunit_client::prelude::*;

client.login("user@example.com", "secret").await?;

let response = client.index().await?;
let html = response.text().await?;
println!("received {} bytes", html.len());
```

### Create a Cek Unit record

```rust
use librcekunit_client::prelude::*;

let mut data = Form::new();
data.insert("no_perjanjian".into(), "SP-25-F0001843".into());
data.insert("nama_nasabah".into(), "SADIMAN".into());
data.insert("nopol".into(), "BP2927GU".into());

let response = client.store(data).await?;
```

### Update a Cek Unit record

```rust
use librcekunit_client::prelude::*;

let mut data = Form::new();
data.insert("status".into(), "LUNAS".into());

let response = client.update(1, data).await?;
```

The client injects `_method=PUT` and `_token` automatically.

### Delete a Cek Unit record

```rust
use librcekunit_client::prelude::*;

let response = client.destroy(1).await?;
```

The client injects `_method=DELETE` and `_token` automatically.

### Export CSV

```rust
use librcekunit_client::prelude::*;

let response = client.export("csv", "nomor", "asc").await?;
let body = response.bytes().await?;
std::fs::write("export.csv", &body)?;
```

### Import CSV from disk

```rust
use librcekunit_client::prelude::*;

client.import_csv(std::path::Path::new("input.csv")).await?;
```

The file is uploaded as multipart with `Content-Type: multipart/form-data` and the field name `csv_file`.

### Set up tracing

```rust
use librcekunit_client::tracing::init_tracing;

fn main() -> Result<(), Box<dyn core::error::Error + Send + Sync>> {
    init_tracing()?;
    Ok(())
}
```

Set `RUST_LOG=librcekunit_api=debug` to see raw HTTP traffic.

### Switch to a different base URL

Rebuild the client with a new `base_url`. The builder is cheap to clone.

```rust
use librcekunit_client::prelude::*;

let builder = ClientBuilder::new()
    .timeout_secs(30)
    .cookie_store(CookieStore::Memory);

let prod = builder.clone().base_url("https://prod.example.edu").build().await?;
let staging = builder.base_url("https://staging.example.edu").build().await?;
```

## Error handling

Every function returns `Result<T, librcekunit_handler::Error>`.

```rust
use librcekunit_client::prelude::*;
use librcekunit_handler::Error;

match ClientBuilder::new().build().await {
    Ok(_client) => println!("client created"),
    Err(Error::Config(msg)) => eprintln!("configuration error: {msg}"),
    Err(Error::Network(msg)) => eprintln!("network error: {msg}"),
    Err(other) => eprintln!("other error: {other}"),
}
```

The error type is re-exported from `librcekunit_handler`. All variants implement `core::error::Error`, `Send`, `Sync`, and `'static`.

### Common error scenarios

| Scenario                                  | Error variant         |
| ----------------------------------------- | --------------------- |
| Missing `base_url` in builder             | `Error::Config`       |
| Missing `LIBRCEKUNIT_BASE_URL` in env     | `Error::Config`       |
| Invalid `LIBRCEKUNIT_TIMEOUT_SECS` in env | `Error::Config`       |
| Missing `LIBRCEKUNIT_EMAIL` in env        | `Error::Config`       |
| Missing `LIBRCEKUNIT_PASSWORD` in env     | `Error::Config`       |
| Server unreachable                        | `Error::Network`      |
| Server returned 500 on login or logout    | `Error::Api`          |
| CSRF token not found in HTML              | `Error::CsrfNotFound` |
| Request timed out                         | `Error::Network`      |
| Cookie file I/O failure                   | `Error::Io`           |

## Testing

Run every test in the crate:

```sh
cargo test -p librcekunit_client
```

Run all targets including integration tests:

```sh
cargo test -p librcekunit_client --all-targets --all-features
```

The crate includes 39 tests across three files:

- `tests/builder_tests.rs` — 18 tests covering default values, chainable methods, overwrite semantics, `build_config` behavior, `build` behavior, `Debug`, and `Clone`.
- `tests/env_tests.rs` — 14 tests covering environment loading with injectable lookup, missing values, invalid timeout, persistent cookie store, custom user agent, and constants stability.
- `tests/integration_tests.rs` — 7 tests covering end-to-end scenarios such as builder to login to CRUD, environment to client to store, and prelude trait bounds.

All tests use `wiremock` for HTTP mocking. No network access is required.

## Coverage

Install `cargo-llvm-cov`:

```sh
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov
```

Generate a terminal summary:

```sh
cargo llvm-cov -p librcekunit_client --all-features --all-targets
```

Generate an HTML report:

```sh
cargo llvm-cov -p librcekunit_client --all-features --all-targets --html --open
```

Current coverage:

- Function coverage: 100 percent.
- Line coverage: 100 percent.
- Region coverage: 100 percent.

Every public function is exercised by at least one test.

## Linting

Run the full lint pipeline for this crate:

```sh
cargo fmt -p librcekunit_client
cargo clippy -p librcekunit_client --all-targets --all-features -- -D warnings
cargo test -p librcekunit_client --all-targets --all-features
```

The workspace enforces the following lint categories at `deny`:

- `clippy::all`
- `clippy::cargo`
- `clippy::complexity`
- `clippy::correctness`
- `clippy::nursery`
- `clippy::pedantic`
- `clippy::perf`
- `clippy::style`
- `clippy::suspicious`
- `clippy::unwrap_used`
- `clippy::expect_used`
- `clippy::panic`
- `clippy::arithmetic_side_effects`
- `clippy::indexing_slicing`
- `clippy::wildcard_enum_match_arm`

The crate forbids `unsafe_code`.

## Design notes

### Why a separate crate

`librcekunit_api` is the HTTP implementation. It is not intended to be the primary entry point. Applications should depend on `librcekunit_client`, which provides a builder, environment loading, and tracing. If a future backend replaces `reqwest`, only `librcekunit_client` needs to be updated.

### Why `build_config` is synchronous

`Config` does not require a Tokio runtime to be constructed. `build_config` returns a `Config` without creating an HTTP client. This lets callers inspect or log the configuration before committing to a runtime.

### Why `from_env_with` is generic

The `_with` variants accept a `Fn(&str) -> Option<String>` closure. Tests can supply deterministic values without calling `std::env::set_var`, which is unsafe in edition 2024 in multi-threaded contexts.

### Why the prelude exists

The prelude re-exports the most commonly used items. Applications import one module and get everything needed: the builder, environment functions, the client, configuration, form type, error type, and all traits.

### Why `init_tracing` returns a `Box<dyn Error>`

`tracing_subscriber` returns different error types across versions. Boxing the error makes the public signature stable across versions.

### Why no `from_file` helper

Configuration file loading is a policy decision. The crate provides environment loading as the built-in option. Applications that need file loading can read the file and pass values to `ClientBuilder` explicitly.

## FAQ

### What does this crate do

It wraps `librcekunit_api` with a builder, environment loading, and tracing setup. This is the recommended entry point for applications.

### What does this crate not do

It does not implement HTTP. It does not define the trait contracts. Those are the responsibilities of `librcekunit_api` and `librcekunit_handler`.

### Why is `base_url` required

Without a base URL, there is no target server. The builder returns `Error::Config` when the base URL is missing.

### Why does `build_config` not validate

`build_config` constructs a `Config`. It does not call `Validate::validate`. The caller decides whether to validate. This keeps construction and validation separate.

### Why does `build` not validate

`build` delegates to `build_config` and then constructs the HTTP client. Validation is left to the caller.

### How do I validate a config

Call `config.validate()` on the resulting `Config`. The method is defined by the `Validate` trait in `librcekunit_handler`.

### How do I use a custom user agent

Chain `.user_agent("custom/1.0")` on the builder. Or set `LIBRCEKUNIT_USER_AGENT` in the environment.

### How do I use a custom timeout

Chain `.timeout_secs(60)` on the builder. Or set `LIBRCEKUNIT_TIMEOUT_SECS=60` in the environment.

### How do I persist cookies

Chain `.cookie_store(CookieStore::Persistent("cookies.json".into()))` on the builder. Or set `LIBRCEKUNIT_COOKIE_FILE=cookies.json` in the environment.

### How do I disable cookie persistence

Chain `.cookie_store(CookieStore::Memory)` on the builder. Cookies are kept in memory only.

### How do I disable cookies entirely

Chain `.cookie_store(CookieStore::None)` on the builder. No cookies are kept.

### How do I log in

Call `client.login(email, password)`. The method is defined by the `Auth` trait and re-exported by this crate.

### How do I log in from environment variables

Call `login_from_env(&client)`. Credentials are read from `LIBRCEKUNIT_EMAIL` and `LIBRCEKUNIT_PASSWORD`.

### How do I log out

Call `client.logout()`. The method is defined by the `Auth` trait.

### How do I list records

Call `client.index()`. The method is defined by the `Crud` trait.

### How do I create a record

Call `client.store(data)` with a `Form`. The method is defined by the `Crud` trait.

### How do I update a record

Call `client.update(id, data)`. The client injects `_method=PUT`.

### How do I delete a record

Call `client.destroy(id)`. The client injects `_method=DELETE`.

### How do I export data

Call `client.export(format, sort, direction)`. The method is defined by the `Dashboard` trait.

### How do I import CSV data

Call `client.import_csv(path)`. The file is uploaded as multipart.

### How do I see debug logs

Call `init_tracing()` at the start of `main`. Set `RUST_LOG=librcekunit_api=debug`.

### How do I disable tracing

Do not call `init_tracing`. The client works without it.

### How do I test without a real server

Use `wiremock` to start a mock server and point the builder at its URI.

### How do I test environment loading

Use the `_with` variants with an injectable lookup closure. Do not call `std::env::set_var` in tests.

### Can I use this crate in `async` code

Yes. Every method on `Client` is asynchronous. The `build` method is asynchronous.

### Can I use this crate in blocking code

Yes. Wrap the calls in `tokio::runtime::Runtime::block_on`.

### Can I clone a client

No. `Client` is not `Clone`. Create a new client with the builder if needed.

### Can I clone a builder

Yes. `ClientBuilder` derives `Clone`. Each clone is independent.

### Can I print a client

Yes. `Client` implements `Debug`. The `HttpClient` field is printed with `base_url` and `cookie_file`.

### Why does `Client::import_csv` exist as a separate method

The `InputUser` trait defines `input_user_import` which accepts a `Form`. Multipart uploads cannot be represented as a `Form`. The separate method bypasses the trait to support file uploads.

### Why not add multipart support to the trait

Multipart adds a dependency on `reqwest::multipart`. Keeping the trait pure avoids this. The separate method keeps the trait clean.

## License

See the `LICENSE` file in the workspace root. The crate is licensed under MIT.
