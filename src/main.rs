use futures::StreamExt;
use log_analizor::config::AppConfig;
use log_analizor::domain::prompt_header_for_raw_log;
use log_analizor::sample_logs::pick_random_sample;
use log_analizor::tools::{ClassifyIncidentTool, ParseLogTool, SuggestFixTool};
use rig::agent::MultiTurnStreamItem;
use rig::client::{CompletionClient, Nothing};
use rig::providers::ollama;
use rig::streaming::{StreamedAssistantContent, StreamedUserContent, StreamingPrompt};
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::from_env()?;
    let sample = pick_random_sample();

    if config.context7_debug {
        println!(
            "Context7 config => enabled={}, debug={}, api_key_present={}",
            config.context7_enabled,
            config.context7_debug,
            config.context7_api_key.is_some()
        );
    }

    let client = ollama::Client::builder()
        .api_key(Nothing)
        .base_url(&config.ollama_host)
        .build()?;

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

    let prompt_header = prompt_header_for_raw_log(sample.raw);

    let prompt = format!(
        "{}\n\n{}\n\nSi necessaire, appelle les outils puis reponds uniquement avec l'analyse finale.",
        prompt_header, sample.raw,
    );
    println!("Cas de test: {}\n{}", sample.name, prompt);

    println!("Agent response :");
    let mut response_stream = agent.stream_prompt(prompt).multi_turn(3).await;

    while let Some(chunk) = response_stream.next().await {
        match chunk {
            Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(text))) => {
                print!("{}", text.text);
                std::io::stdout().flush()?;
            }
            Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Reasoning(
                reasoning,
            ))) => {
                println!("\n[thinking] {}", reasoning.display_text());
            }
            Ok(MultiTurnStreamItem::StreamAssistantItem(
                StreamedAssistantContent::ReasoningDelta { reasoning, .. },
            )) => {
                println!("\n[thinking-delta] {reasoning}");
            }
            Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::ToolCall {
                tool_call,
                internal_call_id,
            })) => {
                println!(
                    "\n[tool-call] internal_id={} name={} args={}",
                    internal_call_id, tool_call.function.name, tool_call.function.arguments
                );
            }
            Ok(MultiTurnStreamItem::StreamAssistantItem(
                StreamedAssistantContent::ToolCallDelta { id, content, .. },
            )) => {
                let delta = serde_json::to_string(&content)?;
                println!("\n[tool-call-delta] id={} content={delta}", id);
            }
            Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Final(_))) => {}
            Ok(MultiTurnStreamItem::StreamUserItem(StreamedUserContent::ToolResult {
                tool_result,
                internal_call_id,
            })) => {
                let tool_content = serde_json::to_string(&tool_result.content)?;
                println!(
                    "\n[tool-result] internal_id={} id={} content={tool_content}",
                    internal_call_id, tool_result.id
                );
            }
            Ok(MultiTurnStreamItem::FinalResponse(final_response)) => {
                let usage = final_response.usage();
                println!(
                    "\n\n[final] tokens_in={} tokens_out={} total={}",
                    usage.input_tokens, usage.output_tokens, usage.total_tokens
                );
                break;
            }
            Ok(_) => {}
            Err(err) => {
                return Err(std::io::Error::other(err.to_string()).into());
            }
        }
    }

    Ok(())
}
