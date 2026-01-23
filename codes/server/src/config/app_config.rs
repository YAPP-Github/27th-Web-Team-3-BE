use std::env;

/// 애플리케이션 설정
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server_port: u16,
    pub jwt_secret: String,
    pub jwt_expiration: i64,
}

impl AppConfig {
    /// 환경 변수에서 설정 로드
    pub fn from_env() -> Result<Self, ConfigError> {
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidPort)?;

        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "secret".to_string());

        let jwt_expiration = env::var("JWT_EXPIRATION")
            .unwrap_or_else(|_| "86400".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidExpiration)?;

        Ok(Self {
            server_port,
            jwt_secret,
            jwt_expiration,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid port number")]
    InvalidPort,
    #[error("Invalid expiration time")]
    InvalidExpiration,
}
