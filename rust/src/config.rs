use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub openai_api_key: String,
    pub ai_secret_key: String,
    pub server_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();

        Ok(Self {
            openai_api_key: env::var("OPENAI_API_KEY")
                .map_err(|_| ConfigError::Missing("OPENAI_API_KEY"))?,
            ai_secret_key: env::var("AI_SECRET_KEY")
                .map_err(|_| ConfigError::Missing("AI_SECRET_KEY"))?,
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .map_err(|_| ConfigError::Invalid("SERVER_PORT"))?,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    Missing(&'static str),

    #[error("Invalid environment variable: {0}")]
    Invalid(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_messages() {
        let missing = ConfigError::Missing("TEST_VAR");
        assert_eq!(
            missing.to_string(),
            "Missing environment variable: TEST_VAR"
        );

        let invalid = ConfigError::Invalid("TEST_VAR");
        assert_eq!(
            invalid.to_string(),
            "Invalid environment variable: TEST_VAR"
        );
    }
}
