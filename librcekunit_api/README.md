# librcekunit_api

HTTP implementation for the librcekunit workspace. Built on `reqwest`. Handles cookies, CSRF tokens, session state, and endpoint dispatch.

This crate implements the trait contracts defined in `librcekunit_handler`. It is the backend engine that the `librcekunit_client` facade delegates to.

The crate follows the workspace-wide strict rules:

- `#![forbid(unsafe_code)]`
- No `unwrap`, no `expect`, no `panic`, no `todo`, no `unreachable`.
- No emoji, no decorative comment, no banner comment.
- Every public item documented.
- No wildcard enum match arm.
- No arithmetic side effect.
- No indexing slicing.
- No unused qualification.

## Table of contents

1. [Installation](#installation)
2. [Quick start](#quick-start)
3. [Crate layout](#crate-layout)
4. [Module: client](#module-client)
5. [Module: http_client](#module-http_client)
6. [Module: cookies](#module-cookies)
7. [Module: csrf](#module-csrf)
8. [Module: endpoints](#module-endpoints)
9. [Module: controller](#module-controller)
10. [CSRF lifecycle](#csrf-lifecycle)
11. [Cookie persistence](#cookie-persistence)
12. [Error handling](#error-handling)
13. [Testing](#testing)
14. [Coverage](#coverage)
15. [Linting](#linting)
16. [Design notes](#design-notes)
17. [FAQ](#faq)
18. [License](#license)

## Installation

Add the crate to `Cargo.toml`:

```toml
[dependencies]
librcekunit_api = { git = "https://codeberg.org/mroczect/librcekunit" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Or, when published:

```toml
[dependencies]
librcekunit_api = "3.0.0"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Rust version: 1.96 or newer. Edition: 2024.

## Quick start

```rust
use librcekunit_api::Client;
use librcekunit_handler::{Auth, Config, CookieStore, Crud, Form};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new("https://campus.example.edu")
        .with_timeout(30)
        .with_user_agent("my-app/1.0")
        .with_cookie_store(CookieStore::Persistent("cookies.json".into()));

    let client = Client::new(config).await?;

    client.login("user@example.com", "secret").await?;

    let response = client.index().await?;
    println!("status: {}", response.status());

    let mut data = Form::new();
    data.insert("no_perjanjian".into(), "SP-25-F0001843".into());
    data.insert("nama_nasabah".into(), "SADIMAN".into());
    let _ = client.store(data).await?;

    Ok(())
}
```

## Crate layout

```text
librcekunit_api/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs
│   ├── client.rs
│   ├── controller.rs
│   ├── cookies.rs
│   ├── csrf.rs
│   ├── http_client.rs
│   └── endpoints/
│       ├── mod.rs
│       ├── auth.rs
│       ├── cekunit.rs
│       ├── dashboard.rs
│       ├── input_data.rs
│       ├── input_user.rs
│       ├── pic.rs
│       └── users.rs
└── tests/
    ├── controller_dispatch_tests.rs
    ├── cookies_tests.rs
    ├── coverage_boost_tests.rs
    ├── endpoint_tests.rs
    ├── integration_tests.rs
    └── unit_tests.rs
```

Every module is public. The crate root re-exports `Client` and `HttpClient`.

## Module: client

Defines the `Client` struct, the primary entry point.

### Struct: Client

```rust
pub struct Client {
    http: HttpClient,
}
```

Methods:

```rust
pub async fn new(config: Config) -> Result<Self>
pub const fn http(&self) -> &HttpClient
pub async fn import_csv(&self, path: &Path) -> Result<reqwest::Response>
```

`new` constructs an `HttpClient` from the given configuration and wraps it.

`http` returns a shared reference to the inner `HttpClient`.

`import_csv` reads a CSV file from disk and uploads it as multipart. The file must exist; missing files return `Error::Io`.

### Debug impl

`Client` implements `Debug` manually. The inner `HttpClient` is not printed in full; only its type name appears. This avoids leaking session state in logs.

### Example

```rust
use librcekunit_api::Client;
use librcekunit_handler::{Config, CookieStore};

let config = Config::new("https://campus.example.edu")
    .with_cookie_store(CookieStore::Memory);

let client = Client::new(config).await?;
let _http = client.http();
```

## Module: http_client

Defines the `HttpClient` struct, the low-level transport.

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

Fields are private. Use the public methods to interact with the client.

### Debug impl

`HttpClient` implements `Debug` manually. The printed fields are `base_url` and `cookie_file`. The `reqwest::Client` field is omitted because it contains internal state that is not useful for debugging.

### Methods

```rust
pub async fn new(config: &Config) -> Result<Self>
pub async fn fetch_csrf(&self, path: &str) -> Result<String>
pub async fn ensure_csrf(&self) -> Result<()>
pub async fn set_csrf(&self, token: Option<String>)
pub async fn get_csrf(&self) -> Option<String>
pub async fn request_impl(
    &self,
    method: HttpMethod,
    path: &str,
    body: Option<Form>,
) -> Result<reqwest::Response>
pub async fn request_multipart(
    &self,
    path: &str,
    field: &str,
    filename: String,
    bytes: Vec<u8>,
) -> Result<reqwest::Response>
pub async fn clear_session(&self)
```

`new` constructs the underlying `reqwest::Client`. Redirects are followed up to five times. The cookie jar is shared across all requests. When `CookieStore::Persistent` is configured, cookies are loaded from disk during construction.

`fetch_csrf` issues a GET request to the given path, extracts the `_token` from the response HTML, caches it, and returns it.

`ensure_csrf` checks whether a token is already cached. If not, it fetches one from `/dashboard`.

`set_csrf` overwrites the cached token. Passing `None` clears it.

`get_csrf` reads the cached token.

`request_impl` issues an HTTP request. For write methods, it ensures a CSRF token is present and injects `_token` into the form body. For read methods with a body, the body is sent as query parameters. Cookies are persisted to disk after each response when `CookieStore::Persistent` is configured.

`request_multipart` sends a `multipart/form-data` request. The CSRF token is included as a form field. Used for CSV uploads.

`clear_session` clears the CSRF cache and deletes the cookie file if it exists.

### Transport impl

`HttpClient` implements `Transport` from `librcekunit_handler`. The associated `Response` type is `reqwest::Response`.

### Example

```rust
use librcekunit_api::HttpClient;
use librcekunit_handler::{Config, CookieStore, Form, HttpMethod, Transport};

let config = Config::new("https://campus.example.edu")
    .with_cookie_store(CookieStore::Memory);

let http = HttpClient::new(&config).await?;

let token = http.fetch_csrf("/dashboard").await?;
println!("csrf: {}", token);

let response = http.request(HttpMethod::GET, "/cekunit", None).await?;
println!("status: {}", response.status());
```

## Module: cookies

Cookie persistence helpers. Exposes two functions.

### Function: save

```rust
pub async fn save(jar: &Jar, base_url: &str, path: &Path) -> Result<()>
```

Serializes the cookies from the jar as a JSON array of strings and writes them to disk. If the jar is empty, the file is deleted if it exists.

### Function: load

```rust
pub async fn load(jar: &Jar, base_url: &str, path: &Path) -> Result<()>
```

Reads a JSON array of cookie strings from disk and adds them to the jar. If the file does not exist, the function is a no-op.

### Error cases

- `Error::Config` when `base_url` cannot be parsed as a URL.
- `Error::Io` when reading or writing the file fails.
- `Error::Json` when the file content is not valid JSON.

### Example

```rust
use librcekunit_api::cookies;
use reqwest::cookie::Jar;
use std::path::Path;

let jar = Jar::default();
let path = Path::new("/tmp/cookies.json");

cookies::save(&jar, "https://campus.example.edu", path).await?;
cookies::load(&jar, "https://campus.example.edu", path).await?;
```

## Module: csrf

CSRF token extraction from HTML.

### Function: extract

```rust
pub fn extract(html: &str) -> Result<String>
```

Parses the HTML and returns the value of the first `input[name='_token']` element.

### Error cases

`Error::CsrfNotFound` when the document does not contain a matching input.

### Example

```rust
use librcekunit_api::csrf;

let html = r#"<form><input name="_token" value="abc123"></form>"#;
let token = csrf::extract(html)?;
assert_eq!(token, "abc123");
```

## Module: endpoints

One module per resource. Each module exposes free functions that take a reference to `HttpClient` and return `Result<reqwest::Response>`.

### auth

```rust
pub async fn login(http: &HttpClient, email: &str, password: &str) -> Result<()>
pub async fn logout(http: &HttpClient) -> Result<()>
```

`login` fetches a CSRF token from `/login`, submits the form with the credentials, and clears the CSRF cache on success. Success is defined as a 2xx or 3xx response.

`logout` clears the CSRF cache, submits an empty form to `/logout`, and clears the session on success.

Both return `Error::Api` with the status code and body when the server responds with a 4xx or 5xx status.

### cekunit

```rust
pub async fn index(http: &HttpClient) -> Result<reqwest::Response>
pub async fn index_with_params(http: &HttpClient, params: Form) -> Result<reqwest::Response>
pub async fn create(http: &HttpClient) -> Result<reqwest::Response>
pub async fn store(http: &HttpClient, data: Form) -> Result<reqwest::Response>
pub async fn show(http: &HttpClient, no: u64) -> Result<reqwest::Response>
pub async fn edit(http: &HttpClient, no: u64) -> Result<reqwest::Response>
pub async fn update(http: &HttpClient, no: u64, data: Form) -> Result<reqwest::Response>
pub async fn destroy(http: &HttpClient, no: u64) -> Result<reqwest::Response>
```

Paths:

| Function            | Method | Path                               |
| ------------------- | ------ | ---------------------------------- |
| `index`             | GET    | `/cekunit`                         |
| `index_with_params` | GET    | `/cekunit?…`                       |
| `create`            | GET    | `/cekunit/create`                  |
| `store`             | POST   | `/cekunit`                         |
| `show`              | GET    | `/cekunit/{no}`                    |
| `edit`              | GET    | `/cekunit/{no}/edit`               |
| `update`            | POST   | `/cekunit/{no}` + `_method=PUT`    |
| `destroy`           | POST   | `/cekunit/{no}` + `_method=DELETE` |

`update` and `destroy` use form method spoofing. The `_method` field is injected automatically.

### dashboard

```rust
pub async fn index(http: &HttpClient) -> Result<reqwest::Response>
pub async fn index_with_params(http: &HttpClient, params: Form) -> Result<reqwest::Response>
pub async fn delete_all(http: &HttpClient) -> Result<reqwest::Response>
pub async fn delete_by_category(http: &HttpClient, column: &str, value: &str) -> Result<reqwest::Response>
pub async fn export(http: &HttpClient, format: &str, sort: &str, direction: &str) -> Result<reqwest::Response>
pub async fn get_unique_values(http: &HttpClient, column: &str) -> Result<reqwest::Response>
```

Paths:

| Function             | Method | Path                                                       |
| -------------------- | ------ | ---------------------------------------------------------- |
| `index`              | GET    | `/dashboard`                                               |
| `index_with_params`  | GET    | `/dashboard?…`                                             |
| `delete_all`         | POST   | `/dashboard/delete-all` + `_method=DELETE`                 |
| `delete_by_category` | POST   | `/dashboard/cekunit/delete-by-category` + `_method=DELETE` |
| `export`             | GET    | `/dashboard/cekunit/export?format=…&sort=…&direction=…`    |
| `get_unique_values`  | GET    | `/dashboard/cekunit/get-unique-values?column=…`            |

`delete_by_category` validates that `column` and `value` are not empty. Returns `Error::Api(400, …)` when either is empty.

`export` accepts `format`, `sort`, and `direction` as query parameters. Common formats include `csv`.

### input_data

```rust
pub async fn create(http: &HttpClient) -> Result<reqwest::Response>
pub async fn store(http: &HttpClient, data: Form) -> Result<reqwest::Response>
```

Paths:

| Function | Method | Path                    |
| -------- | ------ | ----------------------- |
| `create` | GET    | `/dashboard/input-data` |
| `store`  | POST   | `/dashboard/input-data` |

### input_user

```rust
pub async fn index(http: &HttpClient) -> Result<reqwest::Response>
pub async fn index_with_params(http: &HttpClient, params: Form) -> Result<reqwest::Response>
pub async fn create(http: &HttpClient) -> Result<reqwest::Response>
pub async fn store(http: &HttpClient, data: Form) -> Result<reqwest::Response>
pub async fn show(http: &HttpClient, id: u64) -> Result<reqwest::Response>
pub async fn edit(http: &HttpClient, id: u64) -> Result<reqwest::Response>
pub async fn update(http: &HttpClient, id: u64, data: Form) -> Result<reqwest::Response>
pub async fn destroy(http: &HttpClient, id: u64) -> Result<reqwest::Response>
pub async fn export(http: &HttpClient, params: Form) -> Result<reqwest::Response>
pub async fn import(http: &HttpClient, data: Form) -> Result<reqwest::Response>
pub async fn import_file(http: &HttpClient, path: &Path) -> Result<reqwest::Response>
```

Paths:

| Function            | Method | Path                                            |
| ------------------- | ------ | ----------------------------------------------- |
| `index`             | GET    | `/dashboard/input-user`                         |
| `index_with_params` | GET    | `/dashboard/input-user?…`                       |
| `create`            | GET    | `/dashboard/input_user/create`                  |
| `store`             | POST   | `/dashboard/input_user`                         |
| `show`              | GET    | `/dashboard/input_user/{id}`                    |
| `edit`              | GET    | `/dashboard/input_user/{id}/edit`               |
| `update`            | POST   | `/dashboard/input_user/{id}` + `_method=PUT`    |
| `destroy`           | POST   | `/dashboard/input_user/{id}` + `_method=DELETE` |
| `export`            | GET    | `/dashboard/input_user/export?…`                |
| `import`            | POST   | `/dashboard/input_user/insert`                  |
| `import_file`       | POST   | `/dashboard/input_user/insert` (multipart)      |

`import_file` reads the file from disk, uploads it as multipart with the field name `csv_file`. Missing files return `Error::Io`.

### pic

```rust
pub async fn index(http: &HttpClient) -> Result<reqwest::Response>
pub async fn index_with_params(http: &HttpClient, params: Form) -> Result<reqwest::Response>
pub async fn create(http: &HttpClient) -> Result<reqwest::Response>
pub async fn store(http: &HttpClient, data: Form) -> Result<reqwest::Response>
pub async fn show(http: &HttpClient, nomor: u64) -> Result<reqwest::Response>
pub async fn edit(http: &HttpClient, nomor: u64) -> Result<reqwest::Response>
pub async fn update(http: &HttpClient, nomor: u64, data: Form) -> Result<reqwest::Response>
pub async fn destroy(http: &HttpClient, nomor: u64) -> Result<reqwest::Response>
pub async fn dashboard_index(http: &HttpClient) -> Result<reqwest::Response>
pub async fn dashboard_index_with_params(http: &HttpClient, params: Form) -> Result<reqwest::Response>
pub async fn input_create(http: &HttpClient) -> Result<reqwest::Response>
pub async fn input_store(http: &HttpClient, data: Form) -> Result<reqwest::Response>
```

Paths:

| Function                      | Method | Path                              |
| ----------------------------- | ------ | --------------------------------- |
| `index`                       | GET    | `/pic`                            |
| `index_with_params`           | GET    | `/pic?…`                          |
| `create`                      | GET    | `/pic/create`                     |
| `store`                       | POST   | `/pic`                            |
| `show`                        | GET    | `/pic/{nomor}`                    |
| `edit`                        | GET    | `/pic/{nomor}/edit`               |
| `update`                      | POST   | `/pic/{nomor}` + `_method=PUT`    |
| `destroy`                     | POST   | `/pic/{nomor}` + `_method=DELETE` |
| `dashboard_index`             | GET    | `/dashboard/pic`                  |
| `dashboard_index_with_params` | GET    | `/dashboard/pic?…`                |
| `input_create`                | GET    | `/dashboard/input-PIC`            |
| `input_store`                 | POST   | `/dashboard/input-PIC`            |

Note the uppercase `PIC` in the input paths. This matches the server routes exactly.

### users

```rust
pub async fn index(http: &HttpClient) -> Result<reqwest::Response>
pub async fn index_with_params(http: &HttpClient, params: Form) -> Result<reqwest::Response>
pub async fn create(http: &HttpClient) -> Result<reqwest::Response>
pub async fn store(http: &HttpClient, data: Form) -> Result<reqwest::Response>
pub async fn show(http: &HttpClient, nomor: u64) -> Result<reqwest::Response>
pub async fn edit(http: &HttpClient, nomor: u64) -> Result<reqwest::Response>
pub async fn update(http: &HttpClient, nomor: u64, data: Form) -> Result<reqwest::Response>
pub async fn destroy(http: &HttpClient, nomor: u64) -> Result<reqwest::Response>
pub async fn dashboard_index(http: &HttpClient) -> Result<reqwest::Response>
pub async fn dashboard_index_with_params(http: &HttpClient, params: Form) -> Result<reqwest::Response>
```

Paths:

| Function                      | Method | Path                                |
| ----------------------------- | ------ | ----------------------------------- |
| `index`                       | GET    | `/dashboard/users`                  |
| `index_with_params`           | GET    | `/dashboard/users?…`                |
| `create`                      | GET    | `/users/create`                     |
| `store`                       | POST   | `/dashboard/users`                  |
| `show`                        | GET    | `/users/{nomor}`                    |
| `edit`                        | GET    | `/users/{nomor}/edit`               |
| `update`                      | POST   | `/dashboard/users` + `_method=PUT`  |
| `destroy`                     | POST   | `/users/{nomor}` + `_method=DELETE` |
| `dashboard_index`             | GET    | `/dashboard/users`                  |
| `dashboard_index_with_params` | GET    | `/dashboard/users?…`                |

The `update` function ignores its `nomor` parameter because the server route does not accept a path segment.

## Module: controller

Implements every trait from `librcekunit_handler` for `Client`.

Traits implemented:

- `Auth`
- `Crud`
- `Dashboard`
- `InputData`
- `InputUser`
- `Pic`
- `Users`

Each implementation delegates to the corresponding endpoint function.

## CSRF lifecycle

Laravel rejects write requests without a valid CSRF token. The `HttpClient` handles this automatically.

### Fetch

When a write request is issued and no token is cached, `ensure_csrf` fetches a token from `/dashboard`. If the client has not logged in yet, `/dashboard` redirects to `/login`. The client follows the redirect and extracts the token from the login form.

### Cache

After the token is extracted, it is stored in `RwLock<Option<String>>` and reused for all subsequent write requests. The cache is per-client instance.

### Inject

For every write request, `request_impl` reads the cached token and injects it into the form body as `_token`. If the form already contains `_token`, the existing value is preserved.

### Invalidate

The token is cleared in three situations:

1. After a successful login. The session token changes; the old CSRF token is no longer valid.
2. Before a logout request. The token is fetched fresh if the client reuses the same instance.
3. When `reset_csrf_token` or `clear_session` is called.

### Example

```rust
let http = HttpClient::new(&config).await?;
assert!(http.get_csrf().await.is_none());

let _ = http.request(HttpMethod::GET, "/dashboard", None).await?;
let token = http.fetch_csrf("/dashboard").await?;
assert_eq!(http.get_csrf().await.as_deref(), Some(token.as_str()));

http.reset_csrf_token().await;
assert!(http.get_csrf().await.is_none());
```

## Cookie persistence

When `CookieStore::Persistent(path)` is configured, cookies are saved to disk after every request and loaded during construction.

The format is a JSON array of strings, where each string is a raw cookie header value.

### Example

```rust
let config = Config::new("https://campus.example.edu")
    .with_cookie_store(CookieStore::Persistent("cookies.json".into()));

let http = HttpClient::new(&config).await?;

let _ = http.request(HttpMethod::GET, "/dashboard", None).await?;

assert!(std::path::Path::new("cookies.json").exists());
```

### Empty jar

When the jar is empty, `save` deletes the file if it exists. This prevents stale cookie files from being loaded on the next run.

### Failure handling

I/O and JSON errors during save or load are ignored. The client continues to function without persistence. Errors are logged only if tracing is enabled.

## Error handling

Every function returns `Result<T, librcekunit_handler::Error>`.

| Scenario                                      | Error variant         |
| --------------------------------------------- | --------------------- |
| Invalid base URL in config                    | `Error::Config`       |
| Missing CSRF token in HTML                    | `Error::CsrfNotFound` |
| Network failure                               | `Error::Network`      |
| Server returned 4xx or 5xx on login or logout | `Error::Api`          |
| File read or write failure                    | `Error::Io`           |
| JSON parse or serialize failure               | `Error::Json`         |
| Invalid URL when saving cookies               | `Error::Config`       |

### Example

```rust
use librcekunit_handler::Error;

match librcekunit_api::endpoints::auth::login(client.http(), "u@e.com", "bad").await {
    Ok(()) => println!("logged in"),
    Err(Error::Api(status, body)) => eprintln!("api error {status}: {body}"),
    Err(Error::Network(msg)) => eprintln!("network error: {msg}"),
    Err(other) => eprintln!("other error: {other}"),
}
```

## Testing

Run every test in the crate:

```sh
cargo test -p librcekunit_api
```

Run all targets including integration tests:

```sh
cargo test -p librcekunit_api --all-targets --all-features
```

The crate includes 169 tests across six files:

- `tests/controller_dispatch_tests.rs` — 13 tests exercising every trait method through the `Client` facade.
- `tests/cookies_tests.rs` — 9 tests covering cookie save, load, and round-trip behavior.
- `tests/coverage_boost_tests.rs` — 20 tests targeting error branches for `auth`, multipart upload, and session clearing.
- `tests/endpoint_tests.rs` — 41 tests verifying method, path, CSRF injection, and form method spoofing for every endpoint.
- `tests/integration_tests.rs` — 12 tests covering login flow, CSRF lifecycle, cookie persistence, and error propagation.
- `tests/unit_tests.rs` — 116 tests covering CSRF extraction, HTTP method classification, `HttpClient` state, and `Client` construction.

All HTTP tests use `wiremock`. No network access is required.

## Coverage

Install `cargo-llvm-cov`:

```sh
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov
```

Generate a terminal summary:

```sh
cargo llvm-cov -p librcekunit_api --all-features --all-targets
```

Generate an HTML report:

```sh
cargo llvm-cov -p librcekunit_api --all-features --all-targets --html --open
```

Current coverage:

- Function coverage: 94 percent.
- Line coverage: 95 percent.
- Region coverage: 92 percent.

The remaining uncovered lines are defensive branches that cannot be triggered from tests:

- `reqwest::Client` construction failures (OOM, TLS provider errors).
- File I/O errors from the operating system.
- The `_ =>` wildcard arm in the HTTP method match. Since `HttpMethod` is non-exhaustive, this arm exists for future variants.

## Linting

Run the full lint pipeline for this crate:

```sh
cargo fmt -p librcekunit_api
cargo clippy -p librcekunit_api --all-targets --all-features -- -D warnings
cargo test -p librcekunit_api --all-targets --all-features
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

`clippy::multiple_crate_versions` is allowed at the crate root. The `reqwest`, `rustls`, and `aws-lc-rs` dependency tree pulls in multiple versions of `getrandom`, `rand`, `syn`, and `windows-sys`. The workspace cannot resolve these without forking upstream crates.

## Design notes

### Why the CSRF cache is `RwLock<Option<String>>`

Reads happen on every write request. Writes happen only when the token is fetched or cleared. `RwLock` allows concurrent reads while still allowing exclusive writes.

### Why the cookie jar is `Arc<Jar>`

`reqwest::cookie::Jar` is not cloneable. To share it across requests and persistence code, the client wraps it in `Arc`. The `reqwest::Client` builder also takes an `Arc<Jar>` via `cookie_provider`.

### Why the client follows redirects

Laravel returns 302 responses on successful login and logout. The client must follow these to reach the actual destination. Redirects are limited to five to prevent infinite loops.

### Why login and logout return `Error::Api`

For most endpoints, the raw `reqwest::Response` is returned. The caller inspects the status. For login and logout, the outcome is binary: success or failure. Returning a typed error simplifies the caller's logic.

### Why `request_impl` is public

The `Transport` trait is defined in `librcekunit_handler`. The `Client` facade exposes `http()` to give callers direct access to the `HttpClient`. Endpoint functions call `request_impl` directly. Making it public allows advanced callers to issue custom requests.

### Why `import_file` is a free function

The `InputUser::input_user_import` trait method accepts a `Form`, which cannot represent a multipart upload. The free function `import_file` bypasses the trait to support file uploads.

### Why cookie persistence silently ignores errors

Session continuity is a convenience, not a correctness requirement. If persistence fails, the client continues working with in-memory cookies. Silently ignoring the error avoids forcing callers to handle a low-value failure.

### Why `join_url` is a method instead of using `url::Url`

`url::Url` requires a full parse and validation. The endpoint paths are known to be well-formed. The method concatenates strings and normalizes slashes, which is sufficient and faster.

## FAQ

### What does this crate do

It implements the trait contracts from `librcekunit_handler` against an HTTP backend. Handles CSRF tokens, cookies, and endpoint dispatch.

### What does this crate not do

It does not define the trait contracts. It does not provide a builder or environment loading. Those are the responsibilities of `librcekunit_handler` and `librcekunit_client`.

### How do I create a client

Call `Client::new(config)`. The `config` is a `Config` from `librcekunit_handler`.

### How do I log in

Call `client.login(email, password)`. The method fetches a CSRF token from `/login`, submits the form, and clears the cache on success.

### How do I log out

Call `client.logout()`. The method clears the CSRF cache, submits an empty form to `/logout`, and clears the session on success.

### How do I fetch a CSRF token manually

Call `http.fetch_csrf("/dashboard")`. The token is cached and returned.

### How do I reset a CSRF token

Call `http.reset_csrf_token()`. The next write request fetches a fresh token.

### How do I clear a session

Call `http.clear_session()`. The CSRF cache is cleared and the cookie file is deleted if it exists.

### How do I persist cookies to disk

Configure `CookieStore::Persistent(path)` in the config. Cookies are saved after every request and loaded at construction.

### How do I disable cookie persistence

Configure `CookieStore::Memory` or `CookieStore::None`.

### How do I upload a CSV file

Call `client.import_csv(path)`. The file is uploaded as multipart with the field name `csv_file`.

### How do I export data

Call `client.export(format, sort, direction)`. The server returns the file content in the response body.

### Why do I get a 500 error on `/cekunit/1`

The server may have a bug in its `show` or `edit` view, or the resource ID may be invalid. The client returns the raw response, which contains a `Server Error` page.

### Why do I get a 405 error on `/dashboard/input_user/1`

The route only accepts POST, PUT, PATCH, and DELETE. There is no GET handler for that path.

### Why is `_method` injected

Laravel form method spoofing. HTML forms cannot send PUT or DELETE. The `_method` field is inspected by the server and dispatched to the correct handler.

### Why is `_token` injected

Laravel CSRF protection. Every write request must include a valid CSRF token in the form body or in the `X-CSRF-TOKEN` header.

### How do I test a specific endpoint

Use `wiremock` to start a mock server. Point the config at its URI. See `tests/endpoint_tests.rs` for examples.

### How do I test error propagation

Use `wiremock` to configure a 4xx or 5xx response. Assert the returned error variant. See `tests/coverage_boost_tests.rs` for examples.

### How do I debug HTTP traffic

Enable `tracing` at the `debug` level. Set `RUST_LOG=librcekunit_api=debug`. The client emits debug logs for requests and responses.

### Why does `Client::import_csv` exist as a separate method

The `InputUser` trait defines `input_user_import` which accepts a `Form`. Multipart uploads cannot be represented as a `Form`. The separate method bypasses the trait to support file uploads.

### Why does `users::update` ignore its `nomor` parameter

The server route does not accept a path segment. The handler only reads the form body.

### Why are there three different route patterns for input user

The server exposes `/dashboard/input-user` for listing, `/dashboard/input_user` for resource operations, and `/dashboard/input_user/insert` for CSV import. The client maps each function to the correct path.

### Why is the CSRF token fetched from `/dashboard` and not `/login`

`/dashboard` requires authentication. If the client is not authenticated, the server redirects to `/login`, and the client extracts the token from the login form. This works for both authenticated and unauthenticated scenarios.

### Why are cookies loaded at construction

Persistent cookies restore the session across process restarts. This avoids a re-login on every run.

### What happens when the cookie file is corrupted

`load` returns `Error::Json`. The client ignores the error and continues without cookies. The file is overwritten on the next successful request.

### What happens when the CSRF token changes server-side

The client detects a 419 response (Laravel CSRF mismatch) only if the caller inspects the status. Automatic retry on 419 is not implemented. The caller can call `reset_csrf_token` and retry.

### What happens when the server is behind a proxy

`reqwest` reads `HTTP_PROXY` and `HTTPS_PROXY` environment variables automatically. No additional configuration is needed.

### Can I use a self-signed TLS certificate

Not currently. The client uses the system trust store. Custom certificate handling requires modifying `HttpClient::new`.

### Can I stream large responses

Yes. `reqwest::Response` implements `Stream`. Use `response.bytes_stream()`.

### Can I cancel an in-flight request

Yes. Wrap the future in `tokio::select!` and drop it. `reqwest` aborts the request.

## License

See the `LICENSE` file in the workspace root. The crate is licensed under MIT.
