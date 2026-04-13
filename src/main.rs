use log_analizor::config::AppConfig;
use log_analizor::domain::prompt_header_for_raw_log;
use log_analizor::sample_logs::pick_random_sample;
use log_analizor::tools::{ClassifyIncidentTool, ParseLogTool, SuggestFixTool};
use rig::client::{CompletionClient, Nothing};
use rig::completion::Prompt;
use rig::providers::ollama;

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
            Donne en sortie : 1) resume, 2) niveau de severite, 3) action recommandee.",
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
        "{}\n\n{}\n\nUtilise les outils disponibles avant de conclure.",
        prompt_header, sample.raw,
    );
    println!("Cas de test: {}\n{}", sample.name, prompt);

    let response = agent.prompt(prompt).await?;
    println!("{response}");

    Ok(())
}
