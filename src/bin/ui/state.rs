use log_analizor::analyzer::AnalysisEvent;

const EMPTY_LOG_ERROR: &str = "Veuillez coller un log avant d'analyser.";

pub fn validate_raw_log(raw_log: &str) -> Result<String, String> {
    let trimmed = raw_log.trim();
    if trimmed.is_empty() {
        return Err(EMPTY_LOG_ERROR.to_string());
    }

    Ok(trimmed.to_string())
}

pub fn append_stream_event(output: &mut String, event: AnalysisEvent) {
    match event {
        AnalysisEvent::TextDelta(chunk) => output.push_str(&chunk),
        AnalysisEvent::Done { usage_line } => {
            output.push_str("\n\n");
            output.push_str(&usage_line);
        }
    }
}
