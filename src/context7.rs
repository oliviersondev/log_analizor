use serde::Deserialize;

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

pub struct Context7Client {
    client: reqwest::Client,
    api_key: String,
}

impl Context7Client {
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
        }
    }

    pub async fn search_libraries(&self, query: &str) -> Result<Vec<Context7Library>, String> {
        let response = self
            .client
            .get("https://context7.com/search/libraries")
            .query(&[("query", query)])
            .bearer_auth(&self.api_key)
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

        let results = payload
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
            .collect();

        Ok(results)
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
            .client
            .get(url)
            .query(&[("topic", topic)])
            .bearer_auth(&self.api_key)
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

        let payload: Context7DocsResponse = response
            .json()
            .await
            .map_err(|e| format!("Context7 docs parse failed: {e}"))?;

        Ok(payload
            .snippets
            .into_iter()
            .map(|s| Context7Snippet {
                title: s.title,
                content: s.content,
            })
            .collect())
    }
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

#[derive(Debug, Deserialize)]
struct Context7DocsResponse {
    snippets: Vec<Context7DocsSnippet>,
}

#[derive(Debug, Deserialize)]
struct Context7DocsSnippet {
    title: Option<String>,
    content: Option<String>,
}
