use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
}

impl HttpMethod {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GET => "GET",
            Self::POST => "POST",
            Self::PUT => "PUT",
            Self::PATCH => "PATCH",
            Self::DELETE => "DELETE",
            Self::HEAD => "HEAD",
            Self::OPTIONS => "OPTIONS",
        }
    }

    #[must_use]
    pub const fn requires_csrf(self) -> bool {
        !matches!(self, Self::GET | Self::HEAD | Self::OPTIONS)
    }

    #[must_use]
    pub const fn has_body(self) -> bool {
        !matches!(self, Self::GET | Self::HEAD | Self::OPTIONS)
    }
}

impl core::fmt::Display for HttpMethod {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum State {
    #[default]
    Idle,
    Running,
    Stopped,
    Failed,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Mode {
    #[default]
    Once,
    Loop,
    Stream,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CookieStore {
    None,
    #[default]
    Memory,
    Persistent(PathBuf),
}

impl CookieStore {
    #[must_use]
    pub const fn path(&self) -> Option<&PathBuf> {
        match self {
            Self::Persistent(p) => Some(p),
            Self::None | Self::Memory => None,
        }
    }
}
