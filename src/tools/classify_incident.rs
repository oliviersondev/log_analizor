use std::io;

use crate::tools::missing_raw_log_error;

#[rig::tool_macro(
    description = "Classifie grossierement la severite du log.",
    params(raw_log = "Log JSON brut a classifier"),
    required(raw_log)
)]
pub fn classify_incident(raw_log: String) -> Result<String, io::Error> {
    if raw_log.trim().is_empty() {
        return Err(missing_raw_log_error());
    }

    Ok(crate::domain::classify_incident(raw_log))
}
