use std::env;

/// 애플리케이션 설정
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server_port: u16,
}

impl AppConfig {
    /// 환경 변수에서 설정 로드
    pub fn from_env() -> Result<Self, ConfigError> {
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidPort)?;

        Ok(Self {
            server_port,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid port number")]
    InvalidPort,
}
