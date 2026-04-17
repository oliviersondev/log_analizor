use log_analizor::analyzer::AnalysisEvent;
use log_analizor::app::events;

const EMPTY_LOG_ERROR: &str = "Veuillez coller un log avant d'analyser.";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UiState {
    pub raw_log: String,
    pub output: String,
    pub is_loading: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub enum UiAction {
    RawLogChanged(String),
    SubmitStarted { raw_log: String },
    StreamEvent(AnalysisEvent),
    SubmitFailed(String),
    SubmitFinished,
}

pub fn validate_raw_log(raw_log: &str) -> Result<String, String> {
    let trimmed = raw_log.trim();
    if trimmed.is_empty() {
        return Err(EMPTY_LOG_ERROR.to_string());
    }

    Ok(trimmed.to_string())
}

impl UiState {
    pub fn apply(&mut self, action: UiAction) {
        match action {
            UiAction::RawLogChanged(value) => {
                self.raw_log = value;
            }
            UiAction::SubmitStarted { raw_log } => {
                self.raw_log = raw_log;
                self.output.clear();
                self.error_message = None;
                self.is_loading = true;
            }
            UiAction::StreamEvent(event) => {
                if let Some(chunk) = events::render_ui_event(event) {
                    self.output.push_str(&chunk);
                }
            }
            UiAction::SubmitFailed(message) => {
                self.error_message = Some(message);
            }
            UiAction::SubmitFinished => {
                self.is_loading = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{UiAction, UiState};
    use log_analizor::analyzer::{AnalysisEvent, UsageStats};

    #[test]
    fn transitions_to_loading_on_submit_start() {
        let mut state = UiState {
            raw_log: "existing".to_string(),
            output: "old output".to_string(),
            is_loading: false,
            error_message: Some("old error".to_string()),
        };

        state.apply(UiAction::SubmitStarted {
            raw_log: "new log".to_string(),
        });

        assert_eq!(state.raw_log, "new log");
        assert!(state.output.is_empty());
        assert!(state.error_message.is_none());
        assert!(state.is_loading);
    }

    #[test]
    fn appends_stream_events_to_output() {
        let mut state = UiState::default();

        state.apply(UiAction::StreamEvent(AnalysisEvent::TextDelta(
            "hello".to_string(),
        )));
        state.apply(UiAction::StreamEvent(AnalysisEvent::Completed {
            usage: UsageStats {
                input_tokens: 1,
                output_tokens: 2,
                total_tokens: 3,
            },
        }));

        assert!(state.output.contains("hello"));
        assert!(state.output.contains("tokens_in=1"));
    }

    #[test]
    fn transitions_to_error_and_idle_on_failure() {
        let mut state = UiState {
            is_loading: true,
            ..UiState::default()
        };

        state.apply(UiAction::SubmitFailed("boom".to_string()));
        state.apply(UiAction::SubmitFinished);

        assert_eq!(state.error_message.as_deref(), Some("boom"));
        assert!(!state.is_loading);
    }
}
