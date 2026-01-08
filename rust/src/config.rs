use std::env;

#[derive(Clone)]
pub struct AppConfig {
    pub openai_api_key: String,
    pub openai_api_base: String,
    pub ai_secret_key: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            openai_api_key: env::var("OPENAI_API_KEY")
                .expect("OPENAI_API_KEY must be set"),
            openai_api_base: env::var("OPENAI_API_BASE")
                .unwrap_or_else(|_| "https://api.openai.com".to_string()),
            ai_secret_key: env::var("AI_SECRET_KEY")
                .expect("AI_SECRET_KEY must be set"),
        }
    }
}
