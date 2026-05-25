use crate::handler::Error;
use std::fs;
use std::path::PathBuf;

pub fn save_cookies_to_file(content: &str, path: &PathBuf) -> Result<(), Error> {
    fs::write(path, content).map_err(Error::Io)
}

pub fn load_cookies_from_file(path: &PathBuf) -> Result<String, Error> {
    fs::read_to_string(path).map_err(Error::Io)
}
