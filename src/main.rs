use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use strands_agents::models::OllamaModel;
use strands_agents::tools::{AgentTool, ToolContext, ToolResult2};
use strands_agents::{Agent, ToolSpec};

#[derive(Debug, Deserialize, Serialize)]
struct AppLog {
    level: String,
    service: String,
    message: String,
    timestamp: String,
    error_code: Option<String>,
    response_time_ms: Option<u64>,
}

fn parse_log(raw_log: String) -> String {
    match serde_json::from_str::<AppLog>(&raw_log) {
        Ok(log) => {
            format!(
                "Parsed log => level={}, service={}, message={}, error_code={:?}, response_time_ms={:?}, timestamp={}",
                log.level,
                log.service,
                log.message,
                log.error_code,
                log.response_time_ms,
                log.timestamp
            )
        }
        Err(err) => format!("Parse error: {err}"),
    }
}

fn classify_incident(raw_log: String) -> String {
    match serde_json::from_str::<AppLog>(&raw_log) {
        Ok(log) => {
            let severity = if log.level.eq_ignore_ascii_case("error") {
                "high"
            } else if log.response_time_ms.unwrap_or(0) > 2000 {
                "medium"
            } else {
                "low"
            };

            format!(
                "Incident classification => severity={}, service={}, probable_cause={}",
                severity,
                log.service,
                infer_cause(&log)
            )
        }
        Err(err) => format!("Classification impossible: {err}"),
    }
}

fn suggest_fix(raw_log: String) -> String {
    match serde_json::from_str::<AppLog>(&raw_log) {
        Ok(log) => {
            let suggestion = if let Some(code) = &log.error_code {
                match code.as_str() {
                    "DB_TIMEOUT" => {
                        "Verifier la latence DB, le pool de connexions et les requetes lentes."
                    }
                    "AUTH_INVALID_TOKEN" => {
                        "Controler la signature/expiration du token et les logs d'authentification."
                    }
                    "UPSTREAM_502" => {
                        "Verifier la disponibilite du service upstream et ajouter retry/circuit breaker."
                    }
                    _ => "Analyser les logs correles et verifier les metriques du service.",
                }
            } else if log.response_time_ms.unwrap_or(0) > 2000 {
                "Inspecter les dependances lentes et la saturation CPU / I/O."
            } else {
                "Aucune action critique immediate, surveiller l'evolution."
            };

            format!("Suggested action => {suggestion}")
        }
        Err(err) => format!("Suggestion impossible: {err}"),
    }
}

fn infer_cause(log: &AppLog) -> &'static str {
    if let Some(code) = &log.error_code {
        match code.as_str() {
            "DB_TIMEOUT" => "database latency or pool exhaustion",
            "AUTH_INVALID_TOKEN" => "authentication issue",
            "UPSTREAM_502" => "upstream service instability",
            _ => "unknown application error",
        }
    } else if log.response_time_ms.unwrap_or(0) > 2000 {
        "performance degradation"
    } else {
        "minor event"
    }
}

fn extract_raw_log(input: &serde_json::Value) -> Result<String, String> {
    input
        .get("raw_log")
        .and_then(serde_json::Value::as_str)
        .map(std::string::ToString::to_string)
        .ok_or_else(|| "Missing required parameter: raw_log".to_string())
}

#[derive(Clone, Default)]
struct ParseLogTool;

#[async_trait]
impl AgentTool for ParseLogTool {
    fn name(&self) -> &str {
        "parse_log"
    }

    fn description(&self) -> &str {
        "Parse un log JSON brut et retourne un resume exploitable."
    }

    fn tool_spec(&self) -> ToolSpec {
        ToolSpec::new(self.name(), self.description()).with_input_schema(serde_json::json!({
            "type": "object",
            "properties": {
                "raw_log": {
                    "type": "string",
                    "description": "Log JSON brut a parser"
                }
            },
            "required": ["raw_log"]
        }))
    }

    async fn invoke(
        &self,
        input: serde_json::Value,
        _context: &ToolContext,
    ) -> std::result::Result<ToolResult2, String> {
        let raw_log = extract_raw_log(&input)?;
        Ok(ToolResult2::success(parse_log(raw_log)))
    }
}

#[derive(Clone, Default)]
struct ClassifyIncidentTool;

#[async_trait]
impl AgentTool for ClassifyIncidentTool {
    fn name(&self) -> &str {
        "classify_incident"
    }

    fn description(&self) -> &str {
        "Classifie grossierement la severite du log."
    }

    fn tool_spec(&self) -> ToolSpec {
        ToolSpec::new(self.name(), self.description()).with_input_schema(serde_json::json!({
            "type": "object",
            "properties": {
                "raw_log": {
                    "type": "string",
                    "description": "Log JSON brut a classifier"
                }
            },
            "required": ["raw_log"]
        }))
    }

    async fn invoke(
        &self,
        input: serde_json::Value,
        _context: &ToolContext,
    ) -> std::result::Result<ToolResult2, String> {
        let raw_log = extract_raw_log(&input)?;
        Ok(ToolResult2::success(classify_incident(raw_log)))
    }
}

#[derive(Clone, Default)]
struct SuggestFixTool;

#[async_trait]
impl AgentTool for SuggestFixTool {
    fn name(&self) -> &str {
        "suggest_fix"
    }

    fn description(&self) -> &str {
        "Propose une action simple a partir du log."
    }

    fn tool_spec(&self) -> ToolSpec {
        ToolSpec::new(self.name(), self.description()).with_input_schema(serde_json::json!({
            "type": "object",
            "properties": {
                "raw_log": {
                    "type": "string",
                    "description": "Log JSON brut pour suggestion"
                }
            },
            "required": ["raw_log"]
        }))
    }

    async fn invoke(
        &self,
        input: serde_json::Value,
        _context: &ToolContext,
    ) -> std::result::Result<ToolResult2, String> {
        let raw_log = extract_raw_log(&input)?;
        Ok(ToolResult2::success(suggest_fix(raw_log)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();

    let ollama_model = std::env::var("OLLAMA_MODEL").map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Missing OLLAMA_MODEL. Copy .env.example to .env and set it.",
        )
    })?;
    let ollama_host = std::env::var("OLLAMA_HOST").map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Missing OLLAMA_HOST. Copy .env.example to .env and set it.",
        )
    })?;

    let raw_log = r#"{
        "level": "ERROR",
        "service": "invoice-sync",
        "message": "Database connection timeout while syncing invoice #48291",
        "timestamp": "2026-04-05T10:12:34Z",
        "error_code": "DB_TIMEOUT",
        "response_time_ms": 3120
    }"#;

    let mut agent = Agent::builder()
        .model(OllamaModel::new(ollama_model).with_host(ollama_host))
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
        raw_log
    );

    let response = agent.invoke_async(prompt).await?;
    println!("{response}");

    Ok(())
}
