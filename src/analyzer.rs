use futures::StreamExt;
use rig::agent::MultiTurnStreamItem;
use rig::client::{CompletionClient, Nothing};
use rig::providers::ollama;
use rig::streaming::{StreamedAssistantContent, StreamedUserContent, StreamingPrompt};

use crate::config::AppConfig;
use crate::domain::prompt_header_for_raw_log;
use crate::tools::{ClassifyIncidentTool, ParseLogTool, SuggestFixTool};

#[derive(Debug, Clone)]
pub enum AnalysisEvent {
    TextDelta(String),
    Done { usage_line: String },
}

pub fn analysis_prompt(raw_log: &str) -> String {
    let prompt_header = prompt_header_for_raw_log(raw_log);
    format!(
        "{}\n\n{}\n\nSi necessaire, appelle les outils puis reponds uniquement avec l'analyse finale.",
        prompt_header, raw_log,
    )
}

pub async fn analyze_raw_log_stream<F>(
    raw_log: String,
    mut on_event: F,
) -> Result<(), std::io::Error>
where
    F: FnMut(AnalysisEvent),
{
    if raw_log.trim().is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Le log brut est vide.",
        ));
    }

    let config = AppConfig::from_env()?;

    if config.should_print_debug_config() {
        println!("{}", config.debug_config_line());
    }

    let client = ollama::Client::builder()
        .api_key(Nothing)
        .base_url(&config.ollama_host)
        .build()
        .map_err(|err| std::io::Error::other(err.to_string()))?;

    let agent = client
        .agent(&config.ollama_model)
        .preamble(
            "Tu es un assistant SRE concis. \
            Analyse le log fourni en utilisant les outils disponibles. \
            N'affiche jamais une definition d'outil, ni JSON de schema. \
            Donne uniquement la reponse finale en francais avec: \
            1) resume, 2) niveau de severite, 3) action recommandee.",
        )
        .tool(ParseLogTool)
        .tool(ClassifyIncidentTool)
        .tool(SuggestFixTool::new(
            config.context7_enabled,
            config.context7_api_key,
            config.context7_debug,
        ))
        .build();

    let mut response_stream = agent
        .stream_prompt(analysis_prompt(&raw_log))
        .multi_turn(3)
        .await;

    while let Some(chunk) = response_stream.next().await {
        match chunk {
            Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(text))) => {
                on_event(AnalysisEvent::TextDelta(text.text));
            }
            Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Reasoning(
                reasoning,
            ))) => {
                if config.stream_debug {
                    println!("\n[thinking] {}", reasoning.display_text());
                }
            }
            Ok(MultiTurnStreamItem::StreamAssistantItem(
                StreamedAssistantContent::ReasoningDelta { reasoning, .. },
            )) => {
                if config.stream_debug {
                    println!("\n[thinking-delta] {reasoning}");
                }
            }
            Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::ToolCall {
                tool_call,
                internal_call_id,
            })) => {
                if config.stream_debug {
                    println!(
                        "\n[tool-call] internal_id={} name={} args={}",
                        internal_call_id, tool_call.function.name, tool_call.function.arguments
                    );
                }
            }
            Ok(MultiTurnStreamItem::StreamAssistantItem(
                StreamedAssistantContent::ToolCallDelta { id, content, .. },
            )) => {
                if config.stream_debug {
                    let delta = serde_json::to_string(&content)
                        .map_err(|err| std::io::Error::other(err.to_string()))?;
                    println!("\n[tool-call-delta] id={} content={delta}", id);
                }
            }
            Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Final(_))) => {}
            Ok(MultiTurnStreamItem::StreamUserItem(StreamedUserContent::ToolResult {
                tool_result,
                internal_call_id,
            })) => {
                if config.stream_debug {
                    let tool_content = serde_json::to_string(&tool_result.content)
                        .map_err(|err| std::io::Error::other(err.to_string()))?;
                    println!(
                        "\n[tool-result] internal_id={} id={} content={tool_content}",
                        internal_call_id, tool_result.id
                    );
                }
            }
            Ok(MultiTurnStreamItem::FinalResponse(final_response)) => {
                let usage = final_response.usage();
                on_event(AnalysisEvent::Done {
                    usage_line: format!(
                        "[final] tokens_in={} tokens_out={} total={}",
                        usage.input_tokens, usage.output_tokens, usage.total_tokens
                    ),
                });
                break;
            }
            Ok(_) => {}
            Err(err) => return Err(std::io::Error::other(err.to_string())),
        }
    }

    Ok(())
}
