#[derive(Debug, Clone)]
pub struct AppConfig {
    pub ollama_model: String,
    pub ollama_host: String,
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

        Ok(Self {
            ollama_model,
            ollama_host,
        })
    }
}
