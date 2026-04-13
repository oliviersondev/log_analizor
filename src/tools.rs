use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use strands_agents::ToolSpec;
use strands_agents::tools::{AgentTool, ToolContext, ToolResult2};

use crate::context7::{Context7Client, Context7Library};
use crate::domain::{classify_incident, context7_query_from_raw_log, parse_log, suggest_fix};

#[derive(Debug, Clone)]
struct Context7Resolution {
    selected_library_id: String,
    snippets: Vec<String>,
    fallback_attempts: usize,
    candidates: Vec<String>,
}

#[derive(Debug, Clone)]
struct Context7ResolutionError {
    message: String,
    candidates: Vec<String>,
}

fn extract_raw_log(input: &serde_json::Value) -> Result<String, String> {
    input
        .get("raw_log")
        .and_then(serde_json::Value::as_str)
        .map(std::string::ToString::to_string)
        .ok_or_else(|| "Missing required parameter: raw_log".to_string())
}

fn score_library(search_query: &str, lib: &Context7Library) -> f64 {
    let query = search_query.to_ascii_lowercase();
    let title = lib.title.to_ascii_lowercase();
    let desc = lib.description.to_ascii_lowercase();

    let query_terms = query
        .split_whitespace()
        .filter(|t| t.len() > 2)
        .collect::<Vec<_>>();

    let mut term_hits = 0.0;
    for term in query_terms {
        if title.contains(term) {
            term_hits += 2.0;
        } else if desc.contains(term) {
            term_hits += 1.0;
        }
    }

    term_hits
        + (lib.total_snippets.min(200) as f64 / 100.0)
        + lib.trust_score.unwrap_or(0.0)
        + lib.benchmark_score.unwrap_or(0.0) / 100.0
}

async fn resolve_context7_snippets(
    client: &Context7Client,
    search_query: &str,
    topic: &str,
) -> Result<Context7Resolution, Context7ResolutionError> {
    let mut libraries =
        client
            .search_libraries(search_query)
            .await
            .map_err(|e| Context7ResolutionError {
                message: e,
                candidates: vec![],
            })?;

    if libraries.is_empty() {
        return Err(Context7ResolutionError {
            message: "No library found by Context7 search".to_string(),
            candidates: vec![],
        });
    }

    libraries.sort_by(|a, b| {
        let score_a = score_library(search_query, a);
        let score_b = score_library(search_query, b);
        score_b
            .partial_cmp(&score_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let max_attempts = libraries.len().min(3);
    let candidates = libraries
        .iter()
        .take(max_attempts)
        .map(|lib| lib.id.clone())
        .collect::<Vec<_>>();

    let mut last_error = "Context7 candidates exhausted".to_string();

    for (idx, lib) in libraries.into_iter().take(max_attempts).enumerate() {
        match client.fetch_snippets(&lib.id, topic).await {
            Ok(snippets) if !snippets.is_empty() => {
                let rendered = snippets
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
                return Ok(Context7Resolution {
                    selected_library_id: lib.id,
                    snippets: rendered,
                    fallback_attempts: idx + 1,
                    candidates,
                });
            }
            Ok(_) => {
                last_error = format!("No snippets returned for library {}", lib.id);
            }
            Err(err) => {
                last_error = format!("{} (library: {})", err, lib.id);
            }
        }
    }

    Err(Context7ResolutionError {
        message: last_error,
        candidates,
    })
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
    context7_enabled: bool,
    context7_api_key: Option<String>,
    context7_debug: bool,
    context7_cache:
        Arc<Mutex<HashMap<String, Result<Context7Resolution, Context7ResolutionError>>>>,
}

impl SuggestFixTool {
    pub fn new(
        context7_enabled: bool,
        context7_api_key: Option<String>,
        context7_debug: bool,
    ) -> Self {
        Self {
            context7_enabled,
            context7_api_key,
            context7_debug,
            context7_cache: Arc::new(Mutex::new(HashMap::new())),
        }
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

        let context7_section = match (self.context7_enabled, self.context7_api_key.as_deref()) {
            (false, _) => {
                "Context7:\n- called: no\n- reason: disabled (set CONTEXT7_ENABLED=true to enable)"
                    .to_string()
            }
            (true, None) => {
                "Context7:\n- called: no\n- reason: missing CONTEXT7_API_KEY".to_string()
            }
            (true, Some(_)) if context7_query_from_raw_log(&raw_log).is_none() => {
                "Context7:\n- called: no\n- reason: no mapping for this log".to_string()
            }
            (true, Some(api_key)) => {
                let query = context7_query_from_raw_log(&raw_log)
                    .ok_or_else(|| "No Context7 mapping for this log".to_string())?;

                let cache_key = format!("{}::{}", query.search_query, query.topic);

                let cached_result = self
                    .context7_cache
                    .lock()
                    .ok()
                    .and_then(|cache| cache.get(&cache_key).cloned());

                let client = Context7Client::new(api_key.to_string());

                let resolution = if let Some(cached) = cached_result {
                    cached
                } else {
                    let fresh =
                        resolve_context7_snippets(&client, &query.search_query, &query.topic).await;
                    if let Ok(mut cache) = self.context7_cache.lock() {
                        cache.insert(cache_key, fresh.clone());
                    }
                    fresh
                };

                match resolution {
                    Ok(resolution) => {
                        let mut section = format!(
                            "Context7:\n- called: yes\n- search_query: {}\n- selected_library_id: {}\n- fallback_attempts: {}\n- snippets:\n{}",
                            query.search_query,
                            resolution.selected_library_id,
                            resolution.fallback_attempts,
                            resolution.snippets.join("\n")
                        );

                        if self.context7_debug {
                            section.push_str(&format!(
                                "\n- candidates_tested: {}",
                                resolution.candidates.join(", ")
                            ));
                        }

                        section
                    }
                    Err(err) => {
                        let mut section = format!(
                            "Context7:\n- called: yes\n- search_query: {}\n- selected_library_id: none\n- error: {}",
                            query.search_query, err.message
                        );

                        if self.context7_debug {
                            let candidates = if err.candidates.is_empty() {
                                "none".to_string()
                            } else {
                                err.candidates.join(", ")
                            };

                            section.push_str(&format!("\n- candidates_tested: {candidates}"));
                        }

                        section
                    }
                }
            }
        };

        let enriched = format!("{base_suggestion}\n\n{context7_section}");

        Ok(ToolResult2::success(enriched))
    }
}
