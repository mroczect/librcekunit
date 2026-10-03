# librcekunit

Rust workspace for communicating with the Cek Unit Laravel backend.

The workspace is split into three crates:

- `librcekunit_handler` — pure contracts. Types, traits, errors, configuration. No I/O.
- `librcekunit_api` — HTTP implementation built on `reqwest`. Handles cookies, CSRF tokens, endpoint dispatch.
- `librcekunit_client` — thin facade over `api`. Provides a builder, environment loading, and tracing setup.

Every crate in this workspace enforces the same strict rules:

- `#![forbid(unsafe_code)]`
- No `unwrap`, no `expect`, no `panic`, no `todo`, no `unreachable`.
- No emoji, no decorative comment, no banner comment.
- All public items documented.
- Every function returns `Result` or `Option` explicitly.
- All `thiserror` variants are non-exhaustive.
- All enums are non-exhaustive.
- No `wildcard_enum_match_arm`.
- No `arithmetic_side_effects`.
- No `indexing_slicing`.
- No `unwrap_used`, no `expect_used`.

## Table of contents

1. [Installation](#installation)
2. [Quick start](#quick-start)
3. [Workspace layout](#workspace-layout)
4. [librcekunit_handler](#librcekunit_handler)
5. [librcekunit_api](#librcekunit_api)
6. [librcekunit_client](#librcekunit_client)
7. [Common workflows](#common-workflows)
8. [Error handling](#error-handling)
9. [Testing](#testing)
10. [Coverage](#coverage)
11. [Linting](#linting)
12. [Design notes](#design-notes)
13. [FAQ](#faq)
14. [License](#license)

## Installation

Add the client crate to `Cargo.toml`:

```toml
[dependencies]
librcekunit_client = { git = "https://codeberg.org/mroczect/librcekunit" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

If only the contracts are needed, without HTTP:

```toml
[dependencies]
librcekunit_handler = { git = "https://codeberg.org/mroczect/librcekunit" }
```

If the HTTP layer is needed directly, without the facade:

```toml
[dependencies]
librcekunit_api = { git = "https://codeberg.org/mroczect/librcekunit" }
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

## Workspace layout

```text
librcekunit/
├── Cargo.toml
├── README.md
├── LICENSE
├── librcekunit_handler/
│   ├── Cargo.toml
│   ├── README.md
│   ├── src/
│   │   ├── lib.rs
│   │   ├── config.rs
│   │   ├── enums.rs
│   │   ├── error.rs
│   │   ├── traits.rs
│   │   └── types.rs
│   └── tests/
│       ├── error_handling_tests.rs
│       ├── integration_tests.rs
│       ├── property_tests.rs
│       └── use_case_tests.rs
├── librcekunit_api/
│   ├── Cargo.toml
│   ├── README.md
│   ├── src/
│   │   ├── lib.rs
│   │   ├── client.rs
│   │   ├── controller.rs
│   │   ├── cookies.rs
│   │   ├── csrf.rs
│   │   ├── http_client.rs
│   │   └── endpoints/
│   │       ├── mod.rs
│   │       ├── auth.rs
│   │       ├── cekunit.rs
│   │       ├── dashboard.rs
│   │       ├── input_data.rs
│   │       ├── input_user.rs
│   │       ├── pic.rs
│   │       └── users.rs
│   └── tests/
│       ├── controller_dispatch_tests.rs
│       ├── cookies_tests.rs
│       ├── coverage_boost_tests.rs
│       ├── endpoint_tests.rs
│       ├── integration_tests.rs
│       └── unit_tests.rs
└── librcekunit_client/
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

## librcekunit_handler

Pure contracts. No I/O. Depends only on `serde` and `thiserror`.

### Module: config

Constants:

- `DEFAULT_TIMEOUT_SECS: u64 = 30`
- `DEFAULT_USER_AGENT: &str = "librcekunit/3.0"`
- `DEFAULT_COOKIE_FILE: &str = "librcekunit_cookies.json"`

Struct `Config`:

```rust
pub struct Config {
    pub base_url: String,
    pub timeout: Duration,
    pub user_agent: String,
    pub cookie_store: CookieStore,
}
```

Derives `Debug`, `Clone`. Marked `#[non_exhaustive]`.

Methods:

```rust
pub fn new(base_url: &str) -> Self
pub const fn with_timeout(mut self, secs: u64) -> Self
pub fn with_user_agent(mut self, ua: &str) -> Self
pub fn with_cookie_store(mut self, store: CookieStore) -> Self
```

`new` trims trailing slashes from `base_url`. An empty result becomes `"/"`. Timeout defaults to 30 seconds. User agent defaults to `"librcekunit/3.0"`. Cookie store defaults to `CookieStore::Persistent("librcekunit_cookies.json")`.

Trait `Validate`:

```rust
pub trait Validate {
    fn validate(&self) -> Result<()>;
}
```

Checks:

- `base_url` is not empty after trimming.
- `base_url` starts with `"http://"` or `"https://"`.
- `timeout` is greater than zero.
- `user_agent` is not empty after trimming.

Returns `Error::Config` with a descriptive message on failure.

### Module: enums

`HttpMethod`:

```rust
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
}
```

Derives `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Serialize`, `Deserialize`. Marked `#[non_exhaustive]`.

Methods:

- `as_str(self) -> &'static str` returns the wire representation.
- `requires_csrf(self) -> bool` returns `true` for write methods.
- `has_body(self) -> bool` returns `true` for methods that carry a body.

`State`:

```rust
pub enum State { Idle, Running, Stopped, Failed }
```

Default is `Idle`.

`Mode`:

```rust
pub enum Mode { Once, Loop, Stream }
```

Default is `Once`.

`CookieStore`:

```rust
pub enum CookieStore {
    None,
    Memory,
    Persistent(PathBuf),
}
```

Default is `Memory`. Method `path(&self) -> Option<&PathBuf>` returns the path when the variant is `Persistent`.

### Module: error

```rust
pub enum Error {
    Config(String),
    Network(String),
    Io(String),
    Json(String),
    Auth(String),
    Api(u16, String),
    CsrfNotFound,
    NotLoggedIn,
    CookieStore(String),
}
```

Derives `Debug` and `thiserror::Error`. Marked `#[non_exhaustive]`.

Display formats:

| Variant        | Display                           |
| -------------- | --------------------------------- |
| `Config`       | `Configuration error: {0}`        |
| `Network`      | `Network error: {0}`              |
| `Io`           | `IO error: {0}`                   |
| `Json`         | `JSON error: {0}`                 |
| `Auth`         | `Authentication failed: {0}`      |
| `Api`          | `API error ({status}): {message}` |
| `CsrfNotFound` | `CSRF token not found in HTML`    |
| `NotLoggedIn`  | `Not logged in`                   |
| `CookieStore`  | `Cookie store error: {0}`         |

Type alias:

```rust
pub type Result<T, E = Error> = core::result::Result<T, E>;
```

### Module: traits

Type alias:

```rust
pub type Form = HashMap<String, String>;
```

Transport:

```rust
pub trait Transport {
    type Response;
    fn request(&self, method: HttpMethod, path: &str, body: Option<Form>) -> impl Future<Output = Result<Self::Response>> + Send;
    fn fetch_csrf_token(&self, url: &str) -> impl Future<Output = Result<String>> + Send;
    fn reset_csrf_token(&self) -> impl Future<Output = ()> + Send;
    fn clear_session(&self) -> impl Future<Output = ()> + Send;
}
```

Auth:

```rust
pub trait Auth {
    fn login(&self, email: &str, password: &str) -> impl Future<Output = Result<()>> + Send;
    fn logout(&self) -> impl Future<Output = Result<()>> + Send;
}
```

Crud:

```rust
pub trait Crud {
    type Response;
    fn index(&self) -> impl Future<Output = Result<Self::Response>> + Send;
    fn index_with_params(&self, params: Form) -> impl Future<Output = Result<Self::Response>> + Send;
    fn create(&self) -> impl Future<Output = Result<Self::Response>> + Send;
    fn store(&self, data: Form) -> impl Future<Output = Result<Self::Response>> + Send;
    fn show(&self, id: u64) -> impl Future<Output = Result<Self::Response>> + Send;
    fn edit(&self, id: u64) -> impl Future<Output = Result<Self::Response>> + Send;
    fn update(&self, id: u64, data: Form) -> impl Future<Output = Result<Self::Response>> + Send;
    fn destroy(&self, id: u64) -> impl Future<Output = Result<Self::Response>> + Send;
}
```

Dashboard:

```rust
pub trait Dashboard {
    type Response;
    fn dashboard_index(&self) -> impl Future<Output = Result<Self::Response>> + Send;
    fn dashboard_index_with_params(&self, params: Form) -> impl Future<Output = Result<Self::Response>> + Send;
    fn delete_all(&self) -> impl Future<Output = Result<Self::Response>> + Send;
    fn delete_by_category(&self, column: &str, value: &str) -> impl Future<Output = Result<Self::Response>> + Send;
    fn export(&self, format: &str, sort: &str, direction: &str) -> impl Future<Output = Result<Self::Response>> + Send;
    fn get_unique_values(&self, column: &str) -> impl Future<Output = Result<Self::Response>> + Send;
}
```

InputData:

```rust
pub trait InputData {
    type Response;
    fn input_data_create(&self) -> impl Future<Output = Result<Self::Response>> + Send;
    fn input_data_store(&self, data: Form) -> impl Future<Output = Result<Self::Response>> + Send;
}
```

InputUser:

```rust
pub trait InputUser {
    type Response;
    fn input_user_index(&self) -> impl Future<Output = Result<Self::Response>> + Send;
    fn input_user_index_with_params(&self, params: Form) -> impl Future<Output = Result<Self::Response>> + Send;
    fn input_user_create(&self) -> impl Future<Output = Result<Self::Response>> + Send;
    fn input_user_store(&self, data: Form) -> impl Future<Output = Result<Self::Response>> + Send;
    fn input_user_show(&self, id: InputUserId) -> impl Future<Output = Result<Self::Response>> + Send;
    fn input_user_edit(&self, id: InputUserId) -> impl Future<Output = Result<Self::Response>> + Send;
    fn input_user_update(&self, id: InputUserId, data: Form) -> impl Future<Output = Result<Self::Response>> + Send;
    fn input_user_destroy(&self, id: InputUserId) -> impl Future<Output = Result<Self::Response>> + Send;
    fn input_user_export(&self, params: Form) -> impl Future<Output = Result<Self::Response>> + Send;
    fn input_user_import(&self, data: Form) -> impl Future<Output = Result<Self::Response>> + Send;
}
```

Pic:

```rust
pub trait Pic {
    type Response;
    fn pic_index(&self) -> impl Future<Output = Result<Self::Response>> + Send;
    fn pic_index_with_params(&self, params: Form) -> impl Future<Output = Result<Self::Response>> + Send;
    fn pic_create(&self) -> impl Future<Output = Result<Self::Response>> + Send;
    fn pic_store(&self, data: Form) -> impl Future<Output = Result<Self::Response>> + Send;
    fn pic_show(&self, id: PicId) -> impl Future<Output = Result<Self::Response>> + Send;
    fn pic_edit(&self, id: PicId) -> impl Future<Output = Result<Self::Response>> + Send;
    fn pic_update(&self, id: PicId, data: Form) -> impl Future<Output = Result<Self::Response>> + Send;
    fn pic_destroy(&self, id: PicId) -> impl Future<Output = Result<Self::Response>> + Send;
    fn dashboard_pic_index(&self) -> impl Future<Output = Result<Self::Response>> + Send;
    fn dashboard_pic_index_with_params(&self, params: Form) -> impl Future<Output = Result<Self::Response>> + Send;
    fn input_pic_create(&self) -> impl Future<Output = Result<Self::Response>> + Send;
    fn input_pic_store(&self, data: Form) -> impl Future<Output = Result<Self::Response>> + Send;
}
```

Users:

```rust
pub trait Users {
    type Response;
    fn users_index(&self) -> impl Future<Output = Result<Self::Response>> + Send;
    fn users_index_with_params(&self, params: Form) -> impl Future<Output = Result<Self::Response>> + Send;
    fn users_create(&self) -> impl Future<Output = Result<Self::Response>> + Send;
    fn users_store(&self, data: Form) -> impl Future<Output = Result<Self::Response>> + Send;
    fn users_show(&self, id: UserId) -> impl Future<Output = Result<Self::Response>> + Send;
    fn users_edit(&self, id: UserId) -> impl Future<Output = Result<Self::Response>> + Send;
    fn users_update(&self, id: UserId, data: Form) -> impl Future<Output = Result<Self::Response>> + Send;
    fn users_destroy(&self, id: UserId) -> impl Future<Output = Result<Self::Response>> + Send;
    fn dashboard_users_index(&self) -> impl Future<Output = Result<Self::Response>> + Send;
    fn dashboard_users_index_with_params(&self, params: Form) -> impl Future<Output = Result<Self::Response>> + Send;
}
```

### Module: types

Newtype IDs:

- `CekUnitId`
- `InputUserId`
- `PicId`
- `UserId`

Each wraps `u64` and enforces value greater than zero. Derives `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Serialize`, `Deserialize`. Serialized as transparent.

Methods:

- `new(raw: u64) -> Option<Self>` returns `None` when `raw == 0`.
- `get(self) -> u64` returns the inner value.
- `is_valid(self) -> bool` returns `true` when the inner value is positive.

Implements `Display`. Implements `TryFrom<u64>` with `Error = crate::error::Error`.

ApiResponse:

```rust
pub struct ApiResponse<T> {
    pub status: String,
    pub data: Option<T>,
    pub message: Option<String>,
}
```

JoinUrl:

```rust
pub struct JoinUrl;

impl JoinUrl {
    pub fn join(base: &str, path: &str) -> String;
    pub fn is_absolute(url: &str) -> bool;
}
```

`join` returns `path` unchanged when it is absolute. Otherwise trims trailing slashes from `base`, leading slashes from `path`, and joins with a single slash.

## librcekunit_api

HTTP implementation. Depends on `reqwest`, `scraper`, `tokio`, `url`, `serde_json`, and `librcekunit_handler`.

### Modules

`client` — struct `Client` wrapping `HttpClient`.

`http_client` — struct `HttpClient` implementing `Transport`.

`cookies` — cookie persistence helpers.

`csrf` — CSRF token extraction from HTML.

`endpoints` — one module per resource: `auth`, `cekunit`, `dashboard`, `input_data`, `input_user`, `pic`, `users`.

`controller` — implements every trait from `librcekunit_handler` for `Client`.

### Struct: HttpClient

```rust
pub struct HttpClient {
    client: reqwest::Client,
    base_url: String,
    cookie_jar: Arc<Jar>,
    csrf_token: RwLock<Option<String>>,
    cookie_file: Option<PathBuf>,
}
```

Methods:

- `new(config: &Config) -> Result<Self>`
- `fetch_csrf(&self, path: &str) -> Result<String>`
- `ensure_csrf(&self) -> Result<()>`
- `set_csrf(&self, token: Option<String>)`
- `get_csrf(&self) -> Option<String>`
- `request_impl(&self, method: HttpMethod, path: &str, body: Option<Form>) -> Result<reqwest::Response>`
- `request_multipart(&self, path: &str, field: &str, filename: String, bytes: Vec<u8>) -> Result<reqwest::Response>`
- `clear_session(&self)`

CSRF token is fetched lazily on the first write request. All write methods inject `_token` into the form body. All responses trigger cookie persistence when `CookieStore::Persistent` is configured.

### Struct: Client

```rust
pub struct Client {
    http: HttpClient,
}
```

Methods:

- `new(config: Config) -> Result<Self>`
- `http(&self) -> &HttpClient`
- `import_csv(&self, path: &Path) -> Result<reqwest::Response>`

Every method from every handler trait is implemented for `Client`.

### CSRF handling

CSRF tokens are extracted from HTML using the selector `input[name='_token']`. When a write request is issued and no token is cached, the client fetches the token from `/dashboard` (or `/login` during login flow). After a successful login, the cached token is cleared because the session token changes.

### Cookie handling

When `CookieStore::Persistent` is configured, cookies are saved to disk after every request and loaded at client construction. Cookies are serialized as a JSON array of strings. Empty cookie jars delete the persistence file.

### Multipart upload

`request_multipart` sends a `multipart/form-data` request with the CSRF token included as a form field. Used by `input_user::import_file` for CSV uploads.

## librcekunit_client

Facade over `librcekunit_api`. Depends on `librcekunit_api`, `librcekunit_handler`, and `tracing-subscriber`.

### Struct: ClientBuilder

```rust
pub struct ClientBuilder {
    base_url: Option<String>,
    timeout_secs: u64,
    user_agent: Option<String>,
    cookie_store: CookieStore,
}
```

Methods:

- `new() -> Self`
- `base_url(self, url: impl Into<String>) -> Self`
- `timeout_secs(self, secs: u64) -> Self`
- `user_agent(self, ua: impl Into<String>) -> Self`
- `cookie_store(self, store: CookieStore) -> Self`
- `build_config(self) -> Result<Config>` — synchronous, does not require a Tokio runtime.
- `build(self) -> Result<Client>` — asynchronous, creates the underlying HTTP client.

Defaults:

- Timeout: 30 seconds.
- User agent: handler default (`"librcekunit/3.0"`).
- Cookie store: `CookieStore::Memory`.

### Environment loading

Constants:

```rust
pub const ENV_BASE_URL: &str = "LIBRCEKUNIT_BASE_URL";
pub const ENV_COOKIE_FILE: &str = "LIBRCEKUNIT_COOKIE_FILE";
pub const ENV_USER_AGENT: &str = "LIBRCEKUNIT_USER_AGENT";
pub const ENV_TIMEOUT_SECS: &str = "LIBRCEKUNIT_TIMEOUT_SECS";
pub const ENV_EMAIL: &str = "LIBRCEKUNIT_EMAIL";
pub const ENV_PASSWORD: &str = "LIBRCEKUNIT_PASSWORD";
```

Functions:

```rust
pub async fn from_env() -> Result<Client>
pub async fn from_env_with<F>(lookup: F) -> Result<Client> where F: Fn(&str) -> Option<String>
pub async fn login_from_env(client: &Client) -> Result<()>
pub async fn login_from_env_with<F>(client: &Client, lookup: F) -> Result<()> where F: Fn(&str) -> Option<String>
```

The `_with` variants accept an injectable lookup function, which allows tests to provide environment values without mutating the process environment.

### Tracing

```rust
pub type InitResult = Result<(), Box<dyn core::error::Error + Send + Sync>>;

pub fn init_tracing() -> InitResult
pub fn init_tracing_with_filter(filter: &str) -> InitResult
```

`init_tracing` reads from `RUST_LOG` and falls back to `"info"`. `init_tracing_with_filter` accepts a filter string directly.

### Prelude

```rust
use librcekunit_client::prelude::*;
```

Brings into scope: `Client`, `ClientBuilder`, `Config`, `CookieStore`, `Error`, `Result`, `Form`, all environment functions and constants, and the traits `Auth`, `Crud`, `Dashboard`, `InputData`, `InputUser`, `Pic`, `Users`.

## Common workflows

### Log in and list Cek Unit records

```rust
use librcekunit_client::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClientBuilder::new()
        .base_url("https://campus.example.edu")
        .cookie_store(CookieStore::Memory)
        .build()
        .await?;

    client.login("user@example.com", "secret").await?;
    let response = client.index().await?;
    let html = response.text().await?;
    println!("received {} bytes", html.len());
    Ok(())
}
```

### Create a Cek Unit record

```rust
let mut data = Form::new();
data.insert("no_perjanjian".into(), "SP-25-F0001843".into());
data.insert("nama_nasabah".into(), "SADIMAN".into());
data.insert("nopol".into(), "BP2927GU".into());

let response = client.store(data).await?;
```

### Update a Cek Unit record

```rust
let mut data = Form::new();
data.insert("status".into(), "LUNAS".into());

let response = client.update(1, data).await?;
```

The client injects `_method=PUT` and `_token`.

### Delete a Cek Unit record

```rust
let response = client.destroy(1).await?;
```

The client injects `_method=DELETE` and `_token`.

### Export CSV

```rust
let response = client.export("csv", "nomor", "asc").await?;
let body = response.bytes().await?;
std::fs::write("export.csv", &body)?;
```

### Import CSV

```rust
client.import_csv(std::path::Path::new("input.csv")).await?;
```

The file is uploaded as multipart with `Content-Type: multipart/form-data` and the field name `csv_file`.

### Persistent cookies

```rust
let client = ClientBuilder::new()
    .base_url("https://campus.example.edu")
    .cookie_store(CookieStore::Persistent("session.json".into()))
    .build()
    .await?;
```

Cookies are persisted after each request and reloaded at construction.

### Load configuration from environment

```rust
std::env::set_var("LIBRCEKUNIT_BASE_URL", "https://campus.example.edu");
std::env::set_var("LIBRCEKUNIT_EMAIL", "user@example.com");
std::env::set_var("LIBRCEKUNIT_PASSWORD", "secret");

let client = from_env().await?;
login_from_env(&client).await?;
```

## Error handling

Every function returns `Result<T, librcekunit_handler::Error>`. Match on the variant to react precisely.

```rust
use librcekunit_handler::Error;

match client.login("user@example.com", "bad").await {
    Ok(()) => println!("logged in"),
    Err(Error::Api(status, body)) => eprintln!("server returned {status}: {body}"),
    Err(Error::Network(msg)) => eprintln!("network error: {msg}"),
    Err(Error::CsrfNotFound) => eprintln!("csrf token missing"),
    Err(other) => eprintln!("other error: {other}"),
}
```

All error variants implement `core::error::Error`, `Send`, `Sync`, and `'static`. They can be boxed as `Box<dyn core::error::Error + Send + Sync>`.

## Testing

Run every test in the workspace:

```sh
cargo test --workspace --all-targets --all-features
```

Run tests for a single crate:

```sh
cargo test -p librcekunit_handler
cargo test -p librcekunit_api
cargo test -p librcekunit_client
```

The handler crate includes 166 tests covering error display, config validation, newtype ID behavior, URL joining, HTTP method classification, and use case scenarios.

The API crate includes 169 tests covering every endpoint, CSRF lifecycle, cookie persistence, error propagation, and controller dispatch.

The client crate includes 39 tests covering the builder, environment loading, and end-to-end workflows.

The total test count is 374.

## Coverage

Install `cargo-llvm-cov`:

```sh
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov
```

Generate a terminal summary:

```sh
cargo llvm-cov --workspace --all-features --all-targets
```

Generate an HTML report:

```sh
cargo llvm-cov --workspace --all-features --all-targets --html --open
```

The HTML report is written to `target/llvm-cov/html/index.html`.

Current coverage:

- `librcekunit_handler`: 100 percent function, 100 percent line.
- `librcekunit_api`: 94 percent function, 95 percent line.
- `librcekunit_client`: 100 percent function, 100 percent line.

The remaining uncovered lines are defensive branches that cannot be triggered from tests, such as errors during `reqwest::Client` construction and OS-level I/O failures.

## Linting

Run the full lint pipeline:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
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

The workspace forbids `unsafe_code`.

## Design notes

### Why three crates

The handler crate is pure and has no I/O dependencies. Consumers that only need to define types or implement the trait contracts can depend on it without pulling in `reqwest`, `scraper`, or `tokio`.

The API crate implements the contracts against an HTTP backend. It depends on `reqwest` and `scraper`. Swapping the transport later, for example to gRPC or WebSocket, only requires changing this crate.

The client crate is a thin facade. It provides ergonomic helpers such as `ClientBuilder`, environment loading, and tracing setup. It is the recommended entry point for applications.

### Why non-exhaustive everywhere

Adding new variants to `Error`, `HttpMethod`, `State`, `Mode`, and `CookieStore` must not break downstream code. Every enum is marked `#[non_exhaustive]`. Matches on these types require a wildcard arm.

### Why `Option` for IDs

A record ID must be greater than zero. Creating an ID from a `u64` returns `Option<Self>`. The `TryFrom<u64>` implementation converts a zero into `Error::Config`.

### Why `saturating_add` for capacity

`clippy::arithmetic_side_effects` forbids the `+` operator on `usize`. `saturating_add` makes the intent explicit and avoids overflow.

### Why `#[allow(clippy::multiple_crate_versions)]`

The `reqwest`, `rustls`, and `aws-lc-rs` dependency tree pulls in multiple versions of `getrandom`, `rand`, `syn`, and `windows-sys`. The workspace cannot resolve these without forking upstream crates. The allow is scoped to the crate root of `librcekunit_api` and `librcekunit_client`.

## FAQ

### How do I log in with a custom token

Log in with the standard `Auth::login` method. The CSRF token is fetched automatically.

### How do I refresh a CSRF token

Call `http().reset_csrf_token().await`. The next write request fetches a fresh token.

### How do I bypass the CSRF token

Do not. Laravel rejects writes without a valid token.

### How do I disable redirect following

Modify `HttpClient::new` to use `reqwest::redirect::Policy::none()`. This is not exposed by the current API.

### How do I see the raw HTTP traffic

Enable `tracing` at the `debug` level. Set `RUST_LOG=librcekunit_api=debug` before running.

### How do I mock the server in tests

Use `wiremock`. See `librcekunit_api/tests/endpoint_tests.rs` for examples.

### How do I test the client without a real server

Use `ClientBuilder` with a `wiremock` server URI, or use the `_with` variants of the environment functions with an injectable lookup closure.

### Why is `from_env_with` generic

The `_with` variants accept a `Fn(&str) -> Option<String>` closure. Tests can supply deterministic values without mutating the process environment, which is unsafe in edition 2024.

### Why does `Config::new("")` not error

`Config::new` constructs a value; it does not validate. Call `config.validate()` to check. The empty string is normalized to `"/"`.

### What happens when the server returns 500

The `reqwest::Response` is returned as-is. The caller decides whether to inspect the status. Endpoint functions return `Error::Api` only for `Auth::login` and `Auth::logout`, where success is determined by redirect status.

### What happens when the CSRF token is missing

The API returns `Error::CsrfNotFound`. Ensure the target page contains an `<input name="_token">` element.

### What happens when cookies are missing

Requests proceed without cookies. The server may return a redirect to `/login`.

### Can I use the client with a proxy

Set the `HTTP_PROXY` or `HTTPS_PROXY` environment variable. `reqwest` reads these automatically.

### Can I use the client with custom TLS settings

Yes. Modify `HttpClient::new` to add a custom TLS configuration. This is not exposed by the current API.

### Can I stream large responses

Yes. `reqwest::Response` implements `Stream`. Use `response.bytes_stream()`.

### Can I cancel an in-flight request

Yes. Wrap the future in a `tokio::select!` and drop it. `reqwest` aborts the request.

## License

See the `LICENSE` file in the workspace root. The workspace is licensed under MIT.
