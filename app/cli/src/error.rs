//! Error terstruktur + exit code.

/// Exit code yang disepakati antara CLI dan caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExitCode {
    /// Sukses.
    Ok,
    /// Error umum.
    Internal,
    /// Argumen CLI salah.
    Usage,
    /// Konfigurasi salah (env var hilang, URL invalid).
    Config,
    /// Login gagal atau session hilang.
    Auth,
    /// Gagal konek ke server (DNS, TCP, TLS, timeout).
    Network,
    /// Server kembalikan non-2xx.
    Api,
    /// I/O file gagal.
    Io,
    /// CSV tidak valid.
    Csv,
    /// Operasi write ditolak oleh safety gate.
    WriteRefused,
}

impl ExitCode {
    /// Nilai numerik untuk `std::process::exit`.
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        match self {
            Self::Ok => 0,
            Self::Internal => 1,
            Self::Usage => 2,
            Self::Config => 3,
            Self::Auth => 4,
            Self::Network => 5,
            Self::Api => 6,
            Self::Io => 7,
            Self::Csv => 8,
            Self::WriteRefused => 9,
        }
    }
}

impl From<ExitCode> for std::process::ExitCode {
    fn from(code: ExitCode) -> Self {
        Self::from(code.as_u8())
    }
}

/// Error terstruktur.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Argumen CLI salah atau kombinasi tidak valid.
    Usage(String),
    /// Konfigurasi salah.
    Config(String),
    /// Auth atau session.
    Auth(String),
    /// Jaringan.
    Network(String),
    /// Server balas non-2xx.
    Api { status: u16, body: String },
    /// I/O file.
    Io(String),
    /// CSV tidak valid.
    Csv(String),
    /// Safety gate menolak write.
    WriteRefused(String),
    /// Error internal yang tidak diklasifikasi.
    Internal(String),
}

impl Error {
    /// Klasifikasi singkat untuk field `kind` di JSON.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Usage(_) => "usage",
            Self::Config(_) => "config",
            Self::Auth(_) => "auth",
            Self::Network(_) => "network",
            Self::Api { .. } => "api",
            Self::Io(_) => "io",
            Self::Csv(_) => "csv",
            Self::WriteRefused(_) => "write_refused",
            Self::Internal(_) => "internal",
        }
    }

    /// Pesan untuk field `message`.
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Self::Usage(m)
            | Self::Config(m)
            | Self::Auth(m)
            | Self::Network(m)
            | Self::Io(m)
            | Self::Csv(m)
            | Self::WriteRefused(m)
            | Self::Internal(m) => m.clone(),
            Self::Api { status, body } => format!("HTTP {status}: {body}"),
        }
    }

    /// Apakah caller boleh retry.
    #[must_use]
    pub const fn retryable(&self) -> bool {
        matches!(self, Self::Network(_))
    }

    /// Exit code terkait.
    #[must_use]
    pub const fn exit_code(&self) -> ExitCode {
        match self {
            Self::Usage(_) => ExitCode::Usage,
            Self::Config(_) => ExitCode::Config,
            Self::Auth(_) => ExitCode::Auth,
            Self::Network(_) => ExitCode::Network,
            Self::Api { .. } => ExitCode::Api,
            Self::Io(_) => ExitCode::Io,
            Self::Csv(_) => ExitCode::Csv,
            Self::WriteRefused(_) => ExitCode::WriteRefused,
            Self::Internal(_) => ExitCode::Internal,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message())
    }
}

impl std::error::Error for Error {}

/// Alias hasil.
pub type Result<T, E = Error> = core::result::Result<T, E>;

// -----------------------------------------------------------------------------
// From impls
// -----------------------------------------------------------------------------

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

impl From<librcekunit_handler::Error> for Error {
    fn from(e: librcekunit_handler::Error) -> Self {
        match e {
            librcekunit_handler::Error::Config(m) => Self::Config(m),
            librcekunit_handler::Error::Auth(m) => Self::Auth(m),
            librcekunit_handler::Error::NotLoggedIn => Self::Auth(String::from("not logged in")),
            librcekunit_handler::Error::CsrfNotFound => {
                Self::Auth(String::from("csrf token not found"))
            }
            librcekunit_handler::Error::Network(m) => Self::Network(m),
            librcekunit_handler::Error::Io(m) => Self::Io(m),
            librcekunit_handler::Error::Json(m) => Self::Csv(m),
            librcekunit_handler::Error::CookieStore(m) => Self::Io(m),
            librcekunit_handler::Error::Api(status, body) => Self::Api { status, body },
            // Handler Error non-exhaustive.
            _ => Self::Internal(format!("{e}")),
        }
    }
}

impl From<librcekunit_examples::csv_check::CsvError> for Error {
    fn from(e: librcekunit_examples::csv_check::CsvError) -> Self {
        Self::Csv(e.to_string())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Self::Internal(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Self::Internal(s.to_owned())
    }
}
