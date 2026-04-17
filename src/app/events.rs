use crate::analyzer::{AnalysisEvent, UsageStats};

pub struct RenderedEvent {
    pub text: String,
    pub flush: bool,
}

fn usage_line(usage: &UsageStats) -> String {
    format!(
        "[final] tokens_in={} tokens_out={} total={}",
        usage.input_tokens, usage.output_tokens, usage.total_tokens
    )
}

pub fn render_terminal_event(event: AnalysisEvent) -> Option<RenderedEvent> {
    match event {
        AnalysisEvent::TextDelta(text) => Some(RenderedEvent { text, flush: true }),
        AnalysisEvent::DebugConfig(line) => Some(RenderedEvent {
            text: format!("{line}\n"),
            flush: false,
        }),
        AnalysisEvent::Reasoning(reasoning) => Some(RenderedEvent {
            text: format!("\n[thinking] {reasoning}\n"),
            flush: false,
        }),
        AnalysisEvent::ReasoningDelta(reasoning) => Some(RenderedEvent {
            text: format!("\n[thinking-delta] {reasoning}\n"),
            flush: false,
        }),
        AnalysisEvent::ToolCall {
            internal_call_id,
            name,
            arguments,
        } => Some(RenderedEvent {
            text: format!(
                "\n[tool-call] internal_id={} name={} args={}\n",
                internal_call_id, name, arguments
            ),
            flush: false,
        }),
        AnalysisEvent::ToolCallDelta { id, content } => Some(RenderedEvent {
            text: format!("\n[tool-call-delta] id={} content={}\n", id, content),
            flush: false,
        }),
        AnalysisEvent::ToolResult {
            internal_call_id,
            id,
            content,
        } => Some(RenderedEvent {
            text: format!(
                "\n[tool-result] internal_id={} id={} content={}\n",
                internal_call_id, id, content
            ),
            flush: false,
        }),
        AnalysisEvent::Completed { usage } => Some(RenderedEvent {
            text: format!("\n\n{}\n", usage_line(&usage)),
            flush: false,
        }),
    }
}

pub fn render_ui_event(event: AnalysisEvent) -> Option<String> {
    match event {
        AnalysisEvent::TextDelta(text) => Some(text),
        AnalysisEvent::Completed { usage } => Some(format!("\n\n{}", usage_line(&usage))),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::render_terminal_event;
    use crate::analyzer::{AnalysisEvent, UsageStats};

    #[test]
    fn renders_text_delta_with_flush() {
        let rendered = render_terminal_event(AnalysisEvent::TextDelta("chunk".to_string()))
            .expect("text delta should render");

        assert_eq!(rendered.text, "chunk");
        assert!(rendered.flush);
    }

    #[test]
    fn renders_completed_usage_line() {
        let rendered = render_terminal_event(AnalysisEvent::Completed {
            usage: UsageStats {
                input_tokens: 10,
                output_tokens: 20,
                total_tokens: 30,
            },
        })
        .expect("completed event should render");

        assert!(rendered.text.contains("tokens_in=10"));
        assert!(rendered.text.contains("tokens_out=20"));
        assert!(rendered.text.contains("total=30"));
        assert!(!rendered.flush);
    }
}
