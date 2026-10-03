use serde::{Deserialize, Serialize};

macro_rules! newtype_id {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(u64);

        impl $name {
            #[must_use]
            pub const fn new(raw: u64) -> Option<Self> {
                if raw == 0 { None } else { Some(Self(raw)) }
            }

            #[must_use]
            pub const fn get(self) -> u64 {
                self.0
            }

            #[must_use]
            pub const fn is_valid(self) -> bool {
                self.0 > 0
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl TryFrom<u64> for $name {
            type Error = crate::error::Error;

            fn try_from(value: u64) -> Result<Self, Self::Error> {
                Self::new(value).ok_or_else(|| {
                    crate::error::Error::Config(format!(
                        "invalid {}: must be > 0",
                        stringify!($name)
                    ))
                })
            }
        }
    };
}

newtype_id!(CekUnitId);
newtype_id!(InputUserId);
newtype_id!(PicId);
newtype_id!(UserId);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ApiResponse<T> {
    pub status: String,
    pub data: Option<T>,
    pub message: Option<String>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct JoinUrl;

impl JoinUrl {
    #[must_use]
    pub fn join(base: &str, path: &str) -> String {
        if path.starts_with("http://") || path.starts_with("https://") {
            return path.to_string();
        }
        let base = base.trim_end_matches('/');
        let path = path.trim_start_matches('/');
        format!("{base}/{path}")
    }

    #[must_use]
    pub fn is_absolute(url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }
}
