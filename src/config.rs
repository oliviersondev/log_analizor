#[derive(Debug, Clone)]
pub struct AppConfig {
    pub ollama_model: String,
    pub ollama_host: String,
    pub context7_api_key: Option<String>,
    pub context7_debug: bool,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, std::io::Error> {
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

        let context7_api_key = std::env::var("CONTEXT7_API_KEY")
            .ok()
            .and_then(|v| if v.trim().is_empty() { None } else { Some(v) });

        let context7_debug = std::env::var("CONTEXT7_DEBUG")
            .ok()
            .map(|v| {
                let lower = v.trim().to_ascii_lowercase();
                matches!(lower.as_str(), "1" | "true" | "yes" | "on")
            })
            .unwrap_or(false);

        Ok(Self {
            ollama_model,
            ollama_host,
            context7_api_key,
            context7_debug,
        })
    }
}
