use async_trait::async_trait;
use serde::Deserialize;
use strands_agents::ToolSpec;
use strands_agents::tools::{AgentTool, ToolContext, ToolResult2};

use crate::domain::{
    Context7Query, classify_incident, context7_query_from_raw_log, parse_log, suggest_fix,
};

#[derive(Debug, Deserialize)]
struct Context7Snippet {
    title: Option<String>,
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Context7Response {
    snippets: Vec<Context7Snippet>,
}

fn extract_raw_log(input: &serde_json::Value) -> Result<String, String> {
    input
        .get("raw_log")
        .and_then(serde_json::Value::as_str)
        .map(std::string::ToString::to_string)
        .ok_or_else(|| "Missing required parameter: raw_log".to_string())
}

async fn fetch_context7_snippets(
    query: &Context7Query,
    api_key: &str,
) -> Result<Vec<String>, String> {
    let url = format!(
        "https://context7.com/api/v2/docs/code/{}/{}",
        query.library, query.framework
    );

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .query(&[("topic", query.topic)])
        .bearer_auth(api_key)
        .send()
        .await
        .map_err(|e| format!("Context7 request failed: {e}"))?;

    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<no body>".to_string());
        return Err(format!("Context7 API error {status}: {body}"));
    }

    let payload: Context7Response = response
        .json()
        .await
        .map_err(|e| format!("Context7 response parse failed: {e}"))?;

    let snippets = payload
        .snippets
        .into_iter()
        .take(2)
        .map(|s| {
            let title = s.title.unwrap_or_else(|| "Snippet".to_string());
            let content = s
                .content
                .unwrap_or_else(|| "No snippet content returned".to_string());
            format!("- {title}: {content}")
        })
        .collect::<Vec<_>>();

    Ok(snippets)
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
pub struct SuggestFixTool {
    context7_api_key: Option<String>,
}

impl SuggestFixTool {
    pub fn new(context7_api_key: Option<String>) -> Self {
        Self { context7_api_key }
    }
}

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
        let base_suggestion = suggest_fix(raw_log.clone());

        let context7_section = match self.context7_api_key.as_deref() {
            None => "Context7:\n- called: no\n- reason: missing CONTEXT7_API_KEY".to_string(),
            Some(_) if context7_query_from_raw_log(&raw_log).is_none() => {
                "Context7:\n- called: no\n- reason: no mapping for this log".to_string()
            }
            Some(api_key) => {
                let query = context7_query_from_raw_log(&raw_log)
                    .ok_or_else(|| "No Context7 mapping for this log".to_string())?;

                match fetch_context7_snippets(&query, api_key).await {
                    Ok(snippets) if !snippets.is_empty() => format!(
                        "Context7:\n- called: yes\n- snippets:\n{}",
                        snippets.join("\n")
                    ),
                    Ok(_) => "Context7:\n- called: yes\n- snippets: none returned".to_string(),
                    Err(err) => format!("Context7:\n- called: yes\n- error: {err}"),
                }
            }
        };

        let enriched = format!("{base_suggestion}\n\n{context7_section}");

        Ok(ToolResult2::success(enriched))
    }
}
