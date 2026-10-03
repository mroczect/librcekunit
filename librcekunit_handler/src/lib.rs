#![forbid(unsafe_code)]
#![deny(
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub,
    unused_qualifications
)]
#![warn(clippy::all, clippy::pedantic)]

pub mod config;
pub mod enums;
pub mod error;
pub mod traits;
pub mod types;

pub use config::{Config, DEFAULT_COOKIE_FILE, DEFAULT_TIMEOUT_SECS, DEFAULT_USER_AGENT, Validate};
pub use enums::{CookieStore, HttpMethod, Mode, State};
pub use error::{Error, Result};
pub use traits::{Auth, Crud, Dashboard, Form, InputData, InputUser, Pic, Transport, Users};
pub use types::{ApiResponse, CekUnitId, InputUserId, JoinUrl, PicId, UserId};
