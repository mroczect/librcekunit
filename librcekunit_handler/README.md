# librcekunit_handler

Pure contracts for the librcekunit workspace. Types, traits, errors, and configuration only. No I/O. No network. No runtime dependency beyond `serde` and `thiserror`.

This crate is the foundation of the workspace. It defines the interfaces and data models that other crates implement against. Consumers that only need the trait contracts can depend on this crate alone, without pulling in `reqwest`, `scraper`, or `tokio`.

The crate follows the workspace-wide strict rules:

- `#![forbid(unsafe_code)]`
- No `unwrap`, no `expect`, no `panic`, no `todo`, no `unreachable`.
- No emoji, no decorative comment, no banner comment.
- Every public item documented.
- Every enum marked `#[non_exhaustive]`.
- Every error variant marked non-exhaustive.
- No wildcard enum match arm.
- No arithmetic side effect.
- No indexing slicing.
- No unused qualification.

## Table of contents

1. [Installation](#installation)
2. [Quick start](#quick-start)
3. [Crate layout](#crate-layout)
4. [Module: config](#module-config)
5. [Module: enums](#module-enums)
6. [Module: error](#module-error)
7. [Module: traits](#module-traits)
8. [Module: types](#module-types)
9. [Common workflows](#common-workflows)
10. [Error handling](#error-handling)
11. [Implementing a trait](#implementing-a-trait)
12. [Testing](#testing)
13. [Coverage](#coverage)
14. [Linting](#linting)
15. [Design notes](#design-notes)
16. [FAQ](#faq)
17. [License](#license)

## Installation

Add the crate to `Cargo.toml`:

```toml
[dependencies]
librcekunit_handler = { git = "https://codeberg.org/mroczect/librcekunit" }
```

    let err: Error = Error::NotLoggedIn;
    assert!(matches!(err, Error::NotLoggedIn));

Or, when published:

```toml
[dependencies]
librcekunit_handler = "3.0.0"
```

Rust version: 1.96 or newer. Edition: 2024.

## Quick start

```rust
use librcekunit_handler::{Config, CookieStore, Validate, CekUnitId, JoinUrl};

fn main() -> Result<(), librcekunit_handler::Error> {
    let config = Config::new("https://campus.example.edu")
        .with_timeout(30)
        .with_user_agent("my-app/1.0")
        .with_cookie_store(CookieStore::Memory);

    config.validate()?;

    let id = CekUnitId::new(42).ok_or_else(|| {
        librcekunit_handler::Error::Config(String::from("invalid id"))
    })?;

    let url = JoinUrl::join(&config.base_url, &format!("/cekunit/{}", id));
    println!("URL: {}", url);

    Ok(())
}
```

## Crate layout

```text
librcekunit_handler/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs
│   ├── config.rs
│   ├── enums.rs
│   ├── error.rs
│   ├── traits.rs
│   └── types.rs
└── tests/
    ├── error_handling_tests.rs
    ├── integration_tests.rs
    ├── property_tests.rs
    └── use_case_tests.rs
```

Every module is public. Every public item is re-exported at the crate root. Consumers can write `use librcekunit_handler::Config;` instead of `use librcekunit_handler::config::Config;`.

## Module: config

Defines the `Config` struct and the `Validate` trait.

### Constants

```rust
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;
pub const DEFAULT_USER_AGENT: &str = "librcekunit/3.0";
pub const DEFAULT_COOKIE_FILE: &str = "librcekunit_cookies.json";
```

### Struct: Config

```rust
pub struct Config {
    pub base_url: String,
    pub timeout: Duration,
    pub user_agent: String,
    pub cookie_store: CookieStore,
}
```

Derives `Debug` and `Clone`. Marked `#[non_exhaustive]`.

Fields:

- `base_url: String` — base URL of the target server, without trailing slashes.
- `timeout: Duration` — request timeout.
- `user_agent: String` — user agent header value.
- `cookie_store: CookieStore` — cookie persistence strategy.

Methods:

```rust
pub fn new(base_url: &str) -> Self
pub const fn with_timeout(mut self, secs: u64) -> Self
pub fn with_user_agent(mut self, ua: &str) -> Self
pub fn with_cookie_store(mut self, store: CookieStore) -> Self
```

`new` trims trailing slashes from `base_url`. An empty result becomes `"/"`. Timeout defaults to `DEFAULT_TIMEOUT_SECS`. User agent defaults to `DEFAULT_USER_AGENT`. Cookie store defaults to `CookieStore::Persistent(DEFAULT_COOKIE_FILE)`.

Example:

```rust
use librcekunit_handler::{Config, CookieStore};
use core::time::Duration;

let config = Config::new("https://campus.example.edu")
    .with_timeout(60)
    .with_user_agent("custom/1.0")
    .with_cookie_store(CookieStore::Memory);

assert_eq!(config.base_url, "https://campus.example.edu");
assert_eq!(config.timeout, Duration::from_secs(60));
assert_eq!(config.user_agent, "custom/1.0");
```

### Trait: Validate

```rust
pub trait Validate {
    fn validate(&self) -> Result<()>;
}
```

Implemented for `Config`.

Validation rules:

- `base_url` is not empty after trimming.
- `base_url` starts with `"http://"` or `"https://"`.
- `timeout` is greater than zero.
- `user_agent` is not empty after trimming.

Returns `Error::Config` with a descriptive message on failure.

Example:

```rust
use librcekunit_handler::{Config, Validate};

let config = Config::new("ftp://bad.example.com");
assert!(config.validate().is_err());
```

## Module: enums

Defines `HttpMethod`, `State`, `Mode`, and `CookieStore`.

### Enum: HttpMethod

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

- `as_str(self) -> &'static str` — wire representation.
- `requires_csrf(self) -> bool` — `true` for POST, PUT, PATCH, DELETE.
- `has_body(self) -> bool` — `true` for POST, PUT, PATCH, DELETE.

Implements `Display`, which writes the same string as `as_str`.

Example:

```rust
use librcekunit_handler::HttpMethod;

assert_eq!(HttpMethod::GET.as_str(), "GET");
assert!(!HttpMethod::GET.requires_csrf());
assert!(HttpMethod::POST.requires_csrf());
```

### Enum: State

```rust
pub enum State {
    Idle,
    Running,
    Stopped,
    Failed,
}
```

Derives `Debug`, `Default`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Serialize`, `Deserialize`. Marked `#[non_exhaustive]`. Default is `Idle`.

### Enum: Mode

```rust
pub enum Mode {
    Once,
    Loop,
    Stream,
}
```

Derives `Debug`, `Default`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Serialize`, `Deserialize`. Marked `#[non_exhaustive]`. Default is `Once`.

### Enum: CookieStore

```rust
pub enum CookieStore {
    None,
    Memory,
    Persistent(PathBuf),
}
```

Derives `Debug`, `Default`, `Clone`, `PartialEq`, `Eq`, `Hash`. Marked `#[non_exhaustive]`. Default is `Memory`.

Method:

```rust
pub const fn path(&self) -> Option<&PathBuf>
```

Returns `Some(path)` for the `Persistent` variant, `None` otherwise.

Example:

```rust
use librcekunit_handler::CookieStore;
use std::path::PathBuf;

assert!(CookieStore::None.path().is_none());
assert!(CookieStore::Memory.path().is_none());

let store = CookieStore::Persistent(PathBuf::from("cookies.json"));
assert!(store.path().is_some());
```

## Module: error

Defines the `Error` enum and the `Result` type alias.

### Enum: Error

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

Example:

```rust
use librcekunit_handler::Error;

let err = Error::Api(404, String::from("not found"));
assert_eq!(err.to_string(), "API error (404): not found");
```

### Type alias: Result

```rust
pub type Result<T, E = Error> = core::result::Result<T, E>;
```

## Module: traits

Defines the `Form` type alias and eight traits.

### Type alias: Form

```rust
pub type Form = HashMap<String, String>;
```

Used for form bodies and query parameters.

### Trait: Transport

```rust
pub trait Transport {
    type Response;

    fn request(
        &self,
        method: HttpMethod,
        path: &str,
        body: Option<Form>,
    ) -> impl Future<Output = Result<Self::Response>> + Send;

    fn fetch_csrf_token(&self, url: &str) -> impl Future<Output = Result<String>> + Send;

    fn reset_csrf_token(&self) -> impl Future<Output = ()> + Send;

    fn clear_session(&self) -> impl Future<Output = ()> + Send;
}
```

Responsible for HTTP dispatch, CSRF token management, and session cleanup.

### Trait: Auth

```rust
pub trait Auth {
    fn login(&self, email: &str, password: &str) -> impl Future<Output = Result<()>> + Send;
    fn logout(&self) -> impl Future<Output = Result<()>> + Send;
}
```

### Trait: Crud

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

### Trait: Dashboard

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

### Trait: InputData

```rust
pub trait InputData {
    type Response;

    fn input_data_create(&self) -> impl Future<Output = Result<Self::Response>> + Send;
    fn input_data_store(&self, data: Form) -> impl Future<Output = Result<Self::Response>> + Send;
}
```

### Trait: InputUser

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

### Trait: Pic

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

### Trait: Users

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

## Module: types

Defines newtype IDs, `ApiResponse`, and `JoinUrl`.

### Newtype IDs

```rust
pub struct CekUnitId(u64);
pub struct InputUserId(u64);
pub struct PicId(u64);
pub struct UserId(u64);
```

Each newtype enforces that the inner value is greater than zero. Derives `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Serialize`, `Deserialize`. Serialized as transparent.

Methods:

- `new(raw: u64) -> Option<Self>` — returns `None` when `raw == 0`.
- `get(self) -> u64` — returns the inner value.
- `is_valid(self) -> bool` — returns `true` when the inner value is positive.

Implements `Display`. Implements `TryFrom<u64>` with `Error = crate::error::Error`. Returns `Error::Config` when the input is zero.

Example:

```rust
use librcekunit_handler::CekUnitId;

let id = CekUnitId::new(42);
assert_eq!(id.map(CekUnitId::get), Some(42));

assert!(CekUnitId::new(0).is_none());
assert!(CekUnitId::try_from(5_u64).is_ok());
assert!(CekUnitId::try_from(0_u64).is_err());
```

### Struct: ApiResponse

```rust
pub struct ApiResponse<T> {
    pub status: String,
    pub data: Option<T>,
    pub message: Option<String>,
}
```

Derives `Debug`, `Clone`, `PartialEq`, `Eq`, `Serialize`, `Deserialize`. Marked `#[non_exhaustive]`.

Represents a generic response envelope. Deserialization from JSON is supported. Construction from outside the crate is not, by design.

### Struct: JoinUrl

```rust
pub struct JoinUrl;
```

Derives `Debug`, `Default`, `Clone`, `Copy`, `PartialEq`, `Eq`. Marked `#[non_exhaustive]`.

Methods:

```rust
pub fn join(base: &str, path: &str) -> String
pub fn is_absolute(url: &str) -> bool
```

`join` returns `path` unchanged when it starts with `"http://"` or `"https://"`. Otherwise it trims trailing slashes from `base`, leading slashes from `path`, and joins with a single slash.

Example:

```rust
use librcekunit_handler::JoinUrl;

assert_eq!(JoinUrl::join("https://a.com", "/x"), "https://a.com/x");
assert_eq!(JoinUrl::join("https://a.com/", "/x"), "https://a.com/x");
assert_eq!(
    JoinUrl::join("https://a.com", "https://b.com/y"),
    "https://b.com/y"
);
assert!(JoinUrl::is_absolute("https://a.com"));
```

## Common workflows

### Build a validated configuration

```rust
use librcekunit_handler::{Config, CookieStore, Validate};

let config = Config::new("https://campus.example.edu")
    .with_timeout(60)
    .with_user_agent("my-app/1.0")
    .with_cookie_store(CookieStore::Memory);

config.validate()?;
```

### Classify an HTTP method

```rust
use librcekunit_handler::HttpMethod;

let method = HttpMethod::POST;
assert!(method.requires_csrf());
assert!(method.has_body());
assert_eq!(method.as_str(), "POST");
```

### Create and read IDs

```rust
use librcekunit_handler::{CekUnitId, InputUserId, PicId, UserId};

let a = CekUnitId::try_from(1_u64)?;
let b = InputUserId::try_from(2_u64)?;
let c = PicId::try_from(3_u64)?;
let d = UserId::try_from(4_u64)?;

assert_eq!(a.get(), 1);
assert_eq!(b.get(), 2);
assert_eq!(c.get(), 3);
assert_eq!(d.get(), 4);
```

### Check cookie store configuration

```rust
use librcekunit_handler::CookieStore;
use std::path::PathBuf;

let store = CookieStore::Persistent(PathBuf::from("/tmp/cookies.json"));
match store.path() {
    Some(path) => println!("cookies persisted at {}", path.display()),
    None => println!("no persistence"),
}
```

## Error handling

Every public function that can fail returns `Result<T, librcekunit_handler::Error>`.

```rust
use librcekunit_handler::{Config, Error, Validate};

match Config::new("ftp://bad.example.com").validate() {
    Ok(()) => println!("valid"),
    Err(Error::Config(msg)) => eprintln!("config error: {msg}"),
    Err(other) => eprintln!("other error: {other}"),
}
```

All error variants implement `core::error::Error`, `Send`, `Sync`, and `'static`. They can be boxed as `Box<dyn core::error::Error + Send + Sync>`.

Example:

```rust
use librcekunit_handler::Error;
use core::error::Error as StdError;

let err: Box<dyn StdError + Send + Sync> = Box::new(Error::NotLoggedIn);
assert_eq!(err.to_string(), "Not logged in");
```

## Implementing a trait

Any type can implement the traits in this crate. The traits are designed for pure contracts and do not assume any particular transport.

Example implementing `Auth`:

```rust
use librcekunit_handler::{Auth, Result};
use core::future::Future;

struct MyClient;

impl Auth for MyClient {
    fn login(&self, email: &str, password: &str) -> impl Future<Output = Result<()>> + Send {
        let _ = (email, password);
        async move { Ok(()) }
    }

    fn logout(&self) -> impl Future<Output = Result<()>> + Send {
        async move { Ok(()) }
    }
}
```

Example implementing `Transport` with a stub:

```rust
use librcekunit_handler::{Form, HttpMethod, Result, Transport};
use core::future::Future;

struct Stub;

impl Transport for Stub {
    type Response = String;

    fn request(
        &self,
        method: HttpMethod,
        path: &str,
        body: Option<Form>,
    ) -> impl Future<Output = Result<Self::Response>> + Send {
        let _ = (method, path, body);
        async move { Ok(String::from("ok")) }
    }

    fn fetch_csrf_token(&self, url: &str) -> impl Future<Output = Result<String>> + Send {
        let _ = url;
        async move { Ok(String::from("token")) }
    }

    fn reset_csrf_token(&self) -> impl Future<Output = ()> + Send {
        async {}
    }

    fn clear_session(&self) -> impl Future<Output = ()> + Send {
        async {}
    }
}
```

## Testing

Run every test in the crate:

```sh
cargo test -p librcekunit_handler
```

Run all targets including integration tests:

```sh
cargo test -p librcekunit_handler --all-targets --all-features
```

The crate includes 166 tests across four files:

- `tests/error_handling_tests.rs` — 41 tests covering every error variant, Display output, Send/Sync bounds, and core error conversion.
- `tests/integration_tests.rs` — 80 tests covering config behavior, ID construction, URL joining, HTTP method classification, and cookie store variants.
- `tests/property_tests.rs` — 21 tests covering invariants that must hold for any input.
- `tests/use_case_tests.rs` — 24 tests covering realistic scenarios such as setting up a client configuration, validating input, and joining request URLs.

## Coverage

Install `cargo-llvm-cov`:

```sh
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov
```

Generate a terminal summary:

```sh
cargo llvm-cov -p librcekunit_handler --all-features --all-targets
```

Generate an HTML report:

```sh
cargo llvm-cov -p librcekunit_handler --all-features --all-targets --html --open
```

Current coverage:

- Function coverage: 100 percent.
- Line coverage: 100 percent.
- Region coverage: 100 percent.

Every public function is exercised by at least one test.

## Linting

Run the full lint pipeline for this crate:

```sh
cargo fmt -p librcekunit_handler
cargo clippy -p librcekunit_handler --all-targets --all-features -- -D warnings
cargo test -p librcekunit_handler --all-targets --all-features
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

### Why no I/O

The crate defines contracts. Any I/O lives in the implementing crate. This keeps the dependency tree minimal, makes the crate usable in `no_std`-adjacent contexts, and allows tests to run without a runtime.

### Why non-exhaustive everywhere

Adding new variants to `Error`, `HttpMethod`, `State`, `Mode`, and `CookieStore` must not break downstream code. Every enum is marked `#[non_exhaustive]`. Matches on these types require a wildcard arm.

### Why `Option` for IDs

A record ID must be greater than zero. Creating an ID from a `u64` returns `Option<Self>`. The `TryFrom<u64>` implementation converts a zero into `Error::Config`.

### Why `saturating_add` for capacity

`clippy::arithmetic_side_effects` forbids the `+` operator on `usize`. `saturating_add` makes the intent explicit and avoids overflow.

### Why `Duration` for timeout

Using `Duration` prevents unit mistakes. The type encodes the unit. Consumers pass seconds via `with_timeout`, which internally uses `Duration::from_secs`.

### Why `Config::new` does not validate

Construction and validation are separate. `Config::new` builds a value; `Validate::validate` checks it. This lets callers construct a config, mutate it, and validate only when ready.

### Why `CookieStore` is an enum

The persistence strategy is a discrete choice. An enum encodes it precisely. An `Option<PathBuf>` would allow an invalid state such as `None` meaning both "no persistence" and "in-memory".

## FAQ

### What does this crate do

It defines the traits, types, errors, and configuration for the librcekunit workspace. No I/O. No network.

### What does this crate not do

It does not perform HTTP requests. It does not parse HTML. It does not manage cookies. Those are the responsibility of the implementing crate.

### Why not include I/O here

Minimal dependency tree. Testing without runtime. Reuse in non-HTTP contexts.

### How do I implement the traits

Define a struct and implement the trait for it. See the implementing a trait section for examples.

### How do I construct an ID

Use `Id::new(raw)` for an `Option<Self>`, or `Id::try_from(raw)` for a `Result<Self, Error>`. Zero is rejected in both cases.

### How do I validate a config

Call `config.validate()`. The method returns `Result<()>`. See the Validate trait section for the exact rules.

### What happens when a config is invalid

`validate` returns `Error::Config` with a descriptive message. The config is not mutated.

### Why does `Config::new("")` not error

Construction and validation are separate. The empty string is normalized to `"/"`, and `validate` rejects it because it does not start with `http://` or `https://`.

### Why are enums non-exhaustive

To allow new variants without breaking downstream code. Matches on these types require a wildcard arm.

### Why are error variants non-exhaustive

Same reason. New error variants can be added without a breaking change.

### Can I serialize a config to JSON

No. `Config` does not derive `Serialize` or `Deserialize`. Serialization is a transport concern and belongs in the implementing crate.

### Can I compare two configs

No. `Config` does not derive `PartialEq`. Two configs with the same fields may still represent different runtime states. Use field-by-field comparison instead.

### Can I clone a config

Yes. `Config` derives `Clone`.

### Can I print a config

Yes. `Config` derives `Debug`.

### Can I print an error

Yes. `Error` derives `Debug` and implements `Display`.

### Can I box an error

Yes. `Error` implements `core::error::Error`, `Send`, `Sync`, and `'static`. It can be boxed as `Box<dyn core::error::Error + Send + Sync>`.

### Does this crate support no_std

Not currently. The crate uses `std::path::PathBuf` in `CookieStore` and `std::collections::HashMap` in the `Form` alias. A `no_std` variant would require `alloc` and a replacement for `PathBuf`.

### Why `Form` and not `serde_json::Value`

Form data is flat. Every field is a string. A `HashMap<String, String>` is the accurate representation for both form bodies and query parameters.

### Why not use `url::Url` for `base_url`

`Config::new` accepts a `&str` for ergonomics. Validation checks the prefix. The implementing crate can parse it as `Url` if needed.

### How do I add a new trait

Add it to `src/traits.rs`, re-export from `src/lib.rs`, and add tests. Follow the same strict rules: no `unwrap`, no `expect`, all public items documented.

### How do I add a new error variant

Add it to `src/error.rs`, update the `#[error(...)]` format string, re-export is automatic, and add tests in `error_handling_tests.rs`.

### How do I add a new ID type

Add a call to the `newtype_id!` macro in `src/types.rs`, re-export from `src/lib.rs`, and add tests in `integration_tests.rs` and `property_tests.rs`.

### How do I run a single test

```sh
cargo test -p librcekunit_handler --test error_handling_tests -- display_config_variant
```

## License

See the `LICENSE` file in the workspace root. The crate is licensed under MIT.
let err: Error = Error::NotLoggedIn;
assert!(matches!(err, Error::NotLoggedIn));
