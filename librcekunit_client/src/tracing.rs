use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;

pub type InitResult = Result<(), Box<dyn core::error::Error + Send + Sync>>;

pub fn init_tracing() -> InitResult {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).try_init()
}

pub fn init_tracing_with_filter(filter: &str) -> InitResult {
    fmt().with_env_filter(EnvFilter::new(filter)).try_init()
}
