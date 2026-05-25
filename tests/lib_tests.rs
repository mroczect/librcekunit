#[test]
fn test_init_tracing_does_not_panic() {
    librcekunit::init_tracing();
}

#[test]
fn test_public_api_types_exist() {
    let _ = librcekunit::HttpMethod::GET;
    let _ = librcekunit::Config::new("http://x");
    let _ = librcekunit::CookieStore::None;

    let _ = librcekunit::Error::NotLoggedIn;

    fn _check_client(_c: librcekunit::Client) {}
}
