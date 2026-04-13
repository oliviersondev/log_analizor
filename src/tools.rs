use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex};

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::Deserialize;

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

#[derive(Debug, Deserialize)]
pub struct RawLogArgs {
    raw_log: String,
}

fn missing_raw_log_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        "Missing required parameter: raw_log",
    )
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

impl Tool for ParseLogTool {
    const NAME: &'static str = "parse_log";

    type Error = io::Error;
    type Args = RawLogArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Parse un log JSON brut et retourne un resume exploitable.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "raw_log": {
                        "type": "string",
                        "description": "Log JSON brut a parser"
                    }
                },
                "required": ["raw_log"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        if args.raw_log.trim().is_empty() {
            return Err(missing_raw_log_error());
        }

        Ok(parse_log(args.raw_log))
    }
}

#[derive(Clone, Default)]
pub struct ClassifyIncidentTool;

impl Tool for ClassifyIncidentTool {
    const NAME: &'static str = "classify_incident";

    type Error = io::Error;
    type Args = RawLogArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Classifie grossierement la severite du log.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "raw_log": {
                        "type": "string",
                        "description": "Log JSON brut a classifier"
                    }
                },
                "required": ["raw_log"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        if args.raw_log.trim().is_empty() {
            return Err(missing_raw_log_error());
        }

        Ok(classify_incident(args.raw_log))
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

impl Tool for SuggestFixTool {
    const NAME: &'static str = "suggest_fix";

    type Error = io::Error;
    type Args = RawLogArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Propose une action simple a partir du log.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "raw_log": {
                        "type": "string",
                        "description": "Log JSON brut pour suggestion"
                    }
                },
                "required": ["raw_log"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        if args.raw_log.trim().is_empty() {
            return Err(missing_raw_log_error());
        }

        let raw_log = args.raw_log;
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
                    .ok_or_else(|| io::Error::other("No Context7 mapping for this log"))?;

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

        Ok(enriched)
    }
}
