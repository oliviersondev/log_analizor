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

    let mut agent = Agent::builder()
        .model(OllamaModel::new(config.ollama_model).with_host(config.ollama_host))
        .system_prompt(
            "Tu es un assistant SRE concis. \
            Analyse le log fourni en utilisant les outils disponibles. \
            Donne en sortie : 1) resume, 2) niveau de severite, 3) action recommandee.",
        )
        .tool(ParseLogTool)?
        .tool(ClassifyIncidentTool)?
        .tool(SuggestFixTool)?
        .build()?;

    let prompt = format!(
        "Analyse ce log JSON et reponds en francais de maniere structuree:\n\n{}",
        SAMPLE_LOG
    );

    let response = agent.invoke_async(prompt).await?;
    println!("{response}");

    Ok(())
}
