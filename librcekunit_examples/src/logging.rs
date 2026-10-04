use librcekunit_client::tracing as client_tracing;

pub fn init() -> client_tracing::InitResult {
    client_tracing::init_tracing()
}

pub fn init_with_filter(filter: &str) -> client_tracing::InitResult {
    client_tracing::init_tracing_with_filter(filter)
}
