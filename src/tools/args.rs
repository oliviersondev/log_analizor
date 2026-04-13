use std::io;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RawLogArgs {
    pub raw_log: String,
}

pub fn missing_raw_log_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        "Missing required parameter: raw_log",
    )
}
