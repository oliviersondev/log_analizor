use std::io;

use crate::tools::missing_raw_log_error;

#[rig::tool_macro(
    description = "Parse un log JSON brut et retourne un resume exploitable.",
    params(raw_log = "Log JSON brut a parser"),
    required(raw_log)
)]
pub fn parse_log(raw_log: String) -> Result<String, io::Error> {
    if raw_log.trim().is_empty() {
        return Err(missing_raw_log_error());
    }

    Ok(crate::domain::parse_log(raw_log))
}
