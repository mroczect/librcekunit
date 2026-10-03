#![forbid(unsafe_code)]
#![deny(
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub,
    unused_qualifications
)]
#![allow(clippy::multiple_crate_versions)]

#[cfg(test)]
use tokio as _;

#[cfg(test)]
use wiremock as _;

pub mod builder;
pub mod env;
pub mod tracing;

pub use crate::builder::ClientBuilder;
pub use librcekunit_api::Client;
pub use librcekunit_handler::{
    ApiResponse, Auth, CekUnitId, Config, CookieStore, Crud, Dashboard, Error, Form, HttpMethod,
    InputData, InputUser, InputUserId, Mode, Pic, PicId, Result, State, Transport, UserId, Users,
};

pub mod prelude {
    pub use crate::builder::ClientBuilder;
    pub use crate::env::{
        ENV_BASE_URL, ENV_COOKIE_FILE, ENV_EMAIL, ENV_PASSWORD, ENV_TIMEOUT_SECS, ENV_USER_AGENT,
        from_env, from_env_with, login_from_env, login_from_env_with,
    };
    pub use crate::{
        Auth, Client, Config, CookieStore, Crud, Dashboard, Error, Form, InputData, InputUser, Pic,
        Result, Users,
    };
}
