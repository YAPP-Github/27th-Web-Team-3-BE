use std::env;

/// 애플리케이션 설정
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server_port: u16,
    pub openai_api_key: String,
    pub secret_key: String,
}

impl AppConfig {
    /// 환경 변수에서 설정 로드
    pub fn from_env() -> Result<Self, ConfigError> {
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidPort)?;

        let openai_api_key = env::var("OPENAI_API_KEY")
            .map_err(|_| ConfigError::MissingEnvVar("OPENAI_API_KEY".to_string()))?;

        let secret_key = env::var("SECRET_KEY")
            .map_err(|_| ConfigError::MissingEnvVar("SECRET_KEY".to_string()))?;

        Ok(Self {
            server_port,
            openai_api_key,
            secret_key,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),

    #[error("Invalid port number")]
    InvalidPort,
}
