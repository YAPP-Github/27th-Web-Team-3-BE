use std::env;

#[derive(Clone)]
pub struct AppConfig {
    pub openai_api_key: String,
    pub ai_secret_key: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            openai_api_key: env::var("OPENAI_API_KEY")
                .expect("OPENAI_API_KEY must be set"),
            ai_secret_key: env::var("AI_SECRET_KEY")
                .expect("AI_SECRET_KEY must be set"),
        }
    }
}

