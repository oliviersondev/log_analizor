use log_analizor::config::AppConfig;
use log_analizor::domain::prompt_header_for_raw_log;
use log_analizor::sample_logs::pick_random_sample;
use log_analizor::tools::{ClassifyIncidentTool, ParseLogTool, SuggestFixTool};
use strands_agents::Agent;
use strands_agents::models::OllamaModel;

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

    let mut agent = Agent::builder()
        .model(OllamaModel::new(config.ollama_model).with_host(config.ollama_host))
        .system_prompt(
            "Tu es un assistant SRE concis. \
            Analyse le log fourni en utilisant les outils disponibles. \
            Donne en sortie : 1) resume, 2) niveau de severite, 3) action recommandee.",
        )
        .tool(ParseLogTool)?
        .tool(ClassifyIncidentTool)?
        .tool(SuggestFixTool::new(
            config.context7_enabled,
            config.context7_api_key,
            config.context7_debug,
        ))?
        .build()?;

    let suggest_fix_result = agent
        .tool()
        .invoke("suggest_fix", serde_json::json!({ "raw_log": sample.raw }))
        .await?;

    let forced_suggest_fix = suggest_fix_result
        .content
        .iter()
        .filter_map(|c| c.text.as_deref())
        .collect::<Vec<_>>()
        .join("\n");

    let prompt_header = prompt_header_for_raw_log(sample.raw);

    let prompt = format!(
        "{}\n\n{}\n\nSuggestion forcee (tool suggest_fix):\n{}",
        prompt_header,
        sample.raw,
        if forced_suggest_fix.is_empty() {
            "<suggest_fix returned no text>"
        } else {
            &forced_suggest_fix
        }
    );
    println!("Cas de test: {}\n{}", sample.name, prompt);

    let response = agent.invoke_async(prompt).await?;
    println!("{response}");

    Ok(())
}
