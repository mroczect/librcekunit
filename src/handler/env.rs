use dotenvy::dotenv;
use std::env;

pub fn load_env() -> String {
    dotenv().ok();
    env::var("BASE_URL").unwrap_or_else(|_| "http://103.31.38.200".into())
}
