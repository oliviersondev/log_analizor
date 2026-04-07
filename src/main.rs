use log_analizor::config::AppConfig;
use log_analizor::tools::{ClassifyIncidentTool, ParseLogTool, SuggestFixTool};
use strands_agents::Agent;
use strands_agents::models::OllamaModel;

const SAMPLE_LOG: &str = r#"{
    "level": "ERROR",
    "service": "invoice-sync",
    "message": "Database connection timeout while syncing invoice #48291",
    "timestamp": "2026-04-05T10:12:34Z",
    "error_code": "DB_TIMEOUT",
    "response_time_ms": 3120
}"#;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::from_env()?;

    let ollama_model = config.ollama_model;
    let ollama_host = config.ollama_host;
    let context7_api_key = config.context7_api_key;

    let mut agent = Agent::builder()
        .model(OllamaModel::new(ollama_model).with_host(ollama_host))
        .system_prompt(
            "Tu es un assistant SRE concis. \
            Analyse le log fourni en utilisant les outils disponibles. \
            Donne en sortie : 1) resume, 2) niveau de severite, 3) action recommandee.",
        )
        .tool(ParseLogTool)?
        .tool(ClassifyIncidentTool)?
        .tool(SuggestFixTool::new(context7_api_key))?
        .build()?;

    let suggest_fix_result = agent
        .tool()
        .invoke("suggest_fix", serde_json::json!({ "raw_log": SAMPLE_LOG }))
        .await?;

    let forced_suggest_fix = suggest_fix_result
        .content
        .iter()
        .filter_map(|c| c.text.as_deref())
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        "Analyse ce log JSON et reponds en francais de maniere structuree:\n\n{}\n\nSuggestion forcee (tool suggest_fix):\n{}", // TODO c'est pas forcement du JSON
        SAMPLE_LOG,
        if forced_suggest_fix.is_empty() {
            "<suggest_fix returned no text>"
        } else {
            &forced_suggest_fix
        }
    );

    let response = agent.invoke_async(prompt).await?;
    println!("{response}");

    Ok(())
}
