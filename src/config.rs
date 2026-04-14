#[derive(Debug, Clone)]
pub struct AppConfig {
    pub ollama_model: String,
    pub ollama_host: String,
    pub context7_enabled: bool,
    pub context7_api_key: Option<String>,
    pub context7_debug: bool,
    pub stream_debug: bool,
}

fn parse_bool_env(var_name: &str) -> bool {
    std::env::var(var_name)
        .ok()
        .map(|v| {
            let lower = v.trim().to_ascii_lowercase();
            matches!(lower.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(false)
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

        let context7_enabled = parse_bool_env("CONTEXT7_ENABLED");
        let context7_debug = parse_bool_env("CONTEXT7_DEBUG");
        let stream_debug = parse_bool_env("STREAM_DEBUG");

        Ok(Self {
            ollama_model,
            ollama_host,
            context7_enabled,
            context7_api_key,
            context7_debug,
            stream_debug,
        })
    }

    pub fn should_print_debug_config(&self) -> bool {
        self.context7_debug || self.stream_debug
    }

    pub fn debug_config_line(&self) -> String {
        format!(
            "Debug config => context7_enabled={}, context7_debug={}, stream_debug={}, api_key_present={}",
            self.context7_enabled,
            self.context7_debug,
            self.stream_debug,
            self.context7_api_key.is_some()
        )
    }
}
