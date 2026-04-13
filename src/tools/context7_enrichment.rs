use crate::context7::{Context7Client, Context7Library};

#[derive(Debug, Clone)]
pub struct Context7Resolution {
    pub selected_library_id: String,
    pub snippets: Vec<String>,
    pub fallback_attempts: usize,
    pub candidates: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Context7ResolutionError {
    pub message: String,
    pub candidates: Vec<String>,
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

pub async fn resolve_context7_snippets(
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
