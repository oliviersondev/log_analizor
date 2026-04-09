use std::time::Duration;

use serde::Deserialize;

const CONTEXT7_SEARCH_URL: &str = "https://context7.com/api/v2/libs/search";

#[derive(Debug, Clone)]
pub struct Context7Library {
    pub id: String,
    pub title: String,
    pub description: String,
    pub total_snippets: usize,
    pub trust_score: Option<f64>,
    pub benchmark_score: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct Context7Snippet {
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Clone)]
pub struct Context7Client {
    client: reqwest::Client,
    api_key: String,
}

impl Context7Client {
    pub fn new(api_key: String) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("log_analizor/0.1.0")
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(12))
            .pool_idle_timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(4)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self { client, api_key }
    }

    pub async fn search_libraries(&self, query: &str) -> Result<Vec<Context7Library>, String> {
        let library_name = infer_library_name(query);

        let request = self
            .client
            .get(CONTEXT7_SEARCH_URL)
            .query(&[("libraryName", library_name.as_str()), ("query", query)]);

        let response = self
            .apply_auth_if_valid(request)
            .send()
            .await
            .map_err(|e| format!("Context7 library search failed: {e}"))?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "<no body>".to_string());
            return Err(format!("Context7 search API error {status}: {body}"));
        }

        let payload: SearchLibrariesResponse = response
            .json()
            .await
            .map_err(|e| format!("Context7 search parse failed: {e}"))?;

        Ok(payload
            .results
            .into_iter()
            .map(|item| Context7Library {
                id: item.id,
                title: item.title,
                description: item.description,
                total_snippets: item.total_snippets.unwrap_or(0),
                trust_score: item.trust_score,
                benchmark_score: item.benchmark_score,
            })
            .collect())
    }

    pub async fn fetch_snippets(
        &self,
        library_id: &str,
        topic: &str,
    ) -> Result<Vec<Context7Snippet>, String> {
        let (library, framework) = parse_library_id(library_id)?;
        let url = format!(
            "https://context7.com/api/v2/docs/code/{}/{}",
            library, framework
        );

        let response = self
            .apply_auth_if_valid(self.client.get(url).query(&[("topic", topic)]))
            .send()
            .await
            .map_err(|e| format!("Context7 docs request failed: {e}"))?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "<no body>".to_string());
            return Err(format!("Context7 docs API error {status}: {body}"));
        }

        let body = response
            .text()
            .await
            .map_err(|e| format!("Context7 docs body read failed: {e}"))?;

        Ok(parse_docs_text_snippets(&body))
    }

    fn apply_auth_if_valid(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if is_plausible_context7_key(&self.api_key) {
            request.bearer_auth(&self.api_key)
        } else {
            request
        }
    }
}

fn is_plausible_context7_key(value: &str) -> bool {
    let key = value.trim();
    key.starts_with("ctx7sk")
}

fn infer_library_name(query: &str) -> String {
    query
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '.' && c != '-' && c != '_')
        .find(|t| t.len() >= 3)
        .unwrap_or("library")
        .to_ascii_lowercase()
}

fn parse_docs_text_snippets(body: &str) -> Vec<Context7Snippet> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return vec![];
    }

    let sections = if trimmed.contains("\n--------------------------------\n") {
        trimmed
            .split("\n--------------------------------\n")
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>()
    } else {
        vec![trimmed.to_string()]
    };

    sections
        .into_iter()
        .map(|section| {
            let first = section.lines().next().unwrap_or_default().trim();
            let title = first
                .strip_prefix("### ")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());

            Context7Snippet {
                title,
                content: Some(section),
            }
        })
        .collect()
}

fn parse_library_id(library_id: &str) -> Result<(&str, &str), String> {
    let trimmed = library_id.trim_matches('/');
    let mut parts = trimmed.split('/');
    let library = parts
        .next()
        .ok_or_else(|| format!("Invalid Context7 library id: {library_id}"))?;
    let framework = parts
        .next()
        .ok_or_else(|| format!("Invalid Context7 library id: {library_id}"))?;

    if library.is_empty() || framework.is_empty() {
        return Err(format!("Invalid Context7 library id: {library_id}"));
    }

    Ok((library, framework))
}

#[derive(Debug, Deserialize)]
struct SearchLibrariesResponse {
    results: Vec<SearchLibrariesItem>,
}

#[derive(Debug, Deserialize)]
struct SearchLibrariesItem {
    id: String,
    title: String,
    #[serde(default)]
    description: String,
    #[serde(default, alias = "totalSnippets")]
    total_snippets: Option<usize>,
    #[serde(default, alias = "trustScore")]
    trust_score: Option<f64>,
    #[serde(default, alias = "benchmarkScore")]
    benchmark_score: Option<f64>,
}
