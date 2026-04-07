use async_trait::async_trait;
use strands_agents::ToolSpec;
use strands_agents::tools::{AgentTool, ToolContext, ToolResult2};

use crate::domain::{classify_incident, parse_log, suggest_fix};

fn extract_raw_log(input: &serde_json::Value) -> Result<String, String> {
    input
        .get("raw_log")
        .and_then(serde_json::Value::as_str)
        .map(std::string::ToString::to_string)
        .ok_or_else(|| "Missing required parameter: raw_log".to_string())
}

#[derive(Clone, Default)]
pub struct ParseLogTool;

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
    ) -> Result<ToolResult2, String> {
        let raw_log = extract_raw_log(&input)?;
        Ok(ToolResult2::success(parse_log(raw_log)))
    }
}

#[derive(Clone, Default)]
pub struct ClassifyIncidentTool;

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
    ) -> Result<ToolResult2, String> {
        let raw_log = extract_raw_log(&input)?;
        Ok(ToolResult2::success(classify_incident(raw_log)))
    }
}

#[derive(Clone, Default)]
pub struct SuggestFixTool;

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
    ) -> Result<ToolResult2, String> {
        let raw_log = extract_raw_log(&input)?;
        Ok(ToolResult2::success(suggest_fix(raw_log)))
    }
}
