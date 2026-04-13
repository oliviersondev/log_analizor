use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex};

use rig::completion::ToolDefinition;
use rig::tool::Tool;

use crate::context7::Context7Client;
use crate::domain::{context7_query_from_raw_log, suggest_fix};
use crate::tools::{
    Context7Resolution, Context7ResolutionError, RawLogArgs, missing_raw_log_error,
    resolve_context7_snippets,
};

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
