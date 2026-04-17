use futures::StreamExt;
use rig::agent::MultiTurnStreamItem;
use rig::client::{CompletionClient, Nothing};
use rig::providers::ollama;
use rig::streaming::{StreamedAssistantContent, StreamedUserContent, StreamingPrompt};

use crate::config::AppConfig;
use crate::domain::prompt_header_for_raw_log;
use crate::tools::{ClassifyIncidentTool, ParseLogTool, SuggestFixTool};

#[derive(Debug, Clone)]
pub struct UsageStats {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Debug, Clone)]
pub enum AnalysisEvent {
    TextDelta(String),
    DebugConfig(String),
    Reasoning(String),
    ReasoningDelta(String),
    ToolCall {
        internal_call_id: String,
        name: String,
        arguments: String,
    },
    ToolCallDelta {
        id: String,
        content: String,
    },
    ToolResult {
        internal_call_id: String,
        id: String,
        content: String,
    },
    Completed {
        usage: UsageStats,
    },
}

#[derive(Clone)]
pub struct Analyzer {
    config: AppConfig,
    client: ollama::Client,
}

pub fn analysis_prompt(raw_log: &str) -> String {
    let prompt_header = prompt_header_for_raw_log(raw_log);
    format!(
        "{}\n\n{}\n\nSi necessaire, appelle les outils puis reponds uniquement avec l'analyse finale.",
        prompt_header, raw_log,
    )
}

impl Analyzer {
    pub fn new(config: AppConfig) -> Result<Self, std::io::Error> {
        let client = ollama::Client::builder()
            .api_key(Nothing)
            .base_url(&config.ollama_host)
            .build()
            .map_err(|err| std::io::Error::other(err.to_string()))?;

        Ok(Self { config, client })
    }

    pub fn from_env() -> Result<Self, std::io::Error> {
        let config = AppConfig::from_env()?;
        Self::new(config)
    }

    pub async fn analyze_raw_log_stream<F>(
        &self,
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

        if self.config.should_print_debug_config() {
            on_event(AnalysisEvent::DebugConfig(self.config.debug_config_line()));
        }

        let agent = self
            .client
            .agent(&self.config.ollama_model)
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
                self.config.context7_enabled,
                self.config.context7_api_key.clone(),
                self.config.context7_debug,
            ))
            .build();

        let mut response_stream = agent
            .stream_prompt(analysis_prompt(&raw_log))
            .multi_turn(3)
            .await;

        while let Some(chunk) = response_stream.next().await {
            match chunk {
                Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(
                    text,
                ))) => {
                    on_event(AnalysisEvent::TextDelta(text.text));
                }
                Ok(MultiTurnStreamItem::StreamAssistantItem(
                    StreamedAssistantContent::Reasoning(reasoning),
                )) => {
                    if self.config.stream_debug {
                        on_event(AnalysisEvent::Reasoning(reasoning.display_text()));
                    }
                }
                Ok(MultiTurnStreamItem::StreamAssistantItem(
                    StreamedAssistantContent::ReasoningDelta { reasoning, .. },
                )) => {
                    if self.config.stream_debug {
                        on_event(AnalysisEvent::ReasoningDelta(reasoning));
                    }
                }
                Ok(MultiTurnStreamItem::StreamAssistantItem(
                    StreamedAssistantContent::ToolCall {
                        tool_call,
                        internal_call_id,
                    },
                )) => {
                    if self.config.stream_debug {
                        on_event(AnalysisEvent::ToolCall {
                            internal_call_id,
                            name: tool_call.function.name,
                            arguments: tool_call.function.arguments.to_string(),
                        });
                    }
                }
                Ok(MultiTurnStreamItem::StreamAssistantItem(
                    StreamedAssistantContent::ToolCallDelta { id, content, .. },
                )) => {
                    if self.config.stream_debug {
                        let delta = serde_json::to_string(&content)
                            .map_err(|err| std::io::Error::other(err.to_string()))?;
                        on_event(AnalysisEvent::ToolCallDelta { id, content: delta });
                    }
                }
                Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Final(
                    _,
                ))) => {}
                Ok(MultiTurnStreamItem::StreamUserItem(StreamedUserContent::ToolResult {
                    tool_result,
                    internal_call_id,
                })) => {
                    if self.config.stream_debug {
                        let tool_content = serde_json::to_string(&tool_result.content)
                            .map_err(|err| std::io::Error::other(err.to_string()))?;
                        on_event(AnalysisEvent::ToolResult {
                            internal_call_id,
                            id: tool_result.id,
                            content: tool_content,
                        });
                    }
                }
                Ok(MultiTurnStreamItem::FinalResponse(final_response)) => {
                    let usage = final_response.usage();
                    on_event(AnalysisEvent::Completed {
                        usage: UsageStats {
                            input_tokens: usage.input_tokens,
                            output_tokens: usage.output_tokens,
                            total_tokens: usage.total_tokens,
                        },
                    });
                    break;
                }
                Ok(_) => {}
                Err(err) => return Err(std::io::Error::other(err.to_string())),
            }
        }

        Ok(())
    }
}

pub async fn analyze_raw_log_stream<F>(raw_log: String, on_event: F) -> Result<(), std::io::Error>
where
    F: FnMut(AnalysisEvent),
{
    let analyzer = Analyzer::from_env()?;
    analyzer.analyze_raw_log_stream(raw_log, on_event).await
}
