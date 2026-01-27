use std::env;

/// 애플리케이션 설정
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AppConfig {
    pub server_port: u16,
    pub jwt_secret: String,
    pub jwt_expiration: i64,

    // Social Login (향후 소셜 로그인 기능에서 사용)
    pub google_client_id: String,
    pub google_redirect_uri: String,
    pub kakao_client_id: String,
    pub kakao_redirect_uri: String,

    // AI Service
    pub openai_api_key: String,
}

impl AppConfig {
    /// 환경 변수에서 설정 로드
    pub fn from_env() -> Result<Self, ConfigError> {
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidPort)?;

        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| {
            tracing::warn!(
                "JWT_SECRET 환경변수가 설정되지 않았습니다. 프로덕션 환경에서는 반드시 설정하세요."
            );
            "secret".to_string()
        });

        let jwt_expiration = env::var("JWT_EXPIRATION")
            .unwrap_or_else(|_| "86400".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidExpiration)?;

        let google_client_id = env::var("GOOGLE_CLIENT_ID").unwrap_or_default();
        let google_redirect_uri = env::var("GOOGLE_REDIRECT_URI").unwrap_or_default();
        let kakao_client_id = env::var("KAKAO_CLIENT_ID").unwrap_or_default();
        let kakao_redirect_uri = env::var("KAKAO_REDIRECT_URI").unwrap_or_default();

        let openai_api_key = env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
            tracing::warn!(
                "OPENAI_API_KEY 환경변수가 설정되지 않았습니다. 프로덕션 환경에서는 반드시 설정하세요."
            );
            "test-key".to_string()
        });
        Ok(Self {
            server_port,
            jwt_secret,
            jwt_expiration,
            google_client_id,
            google_redirect_uri,
            kakao_client_id,
            kakao_redirect_uri,
            openai_api_key,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid port number")]
    InvalidPort,
    #[error("Invalid expiration time")]
    InvalidExpiration,
    #[error("JWT_SECRET environment variable is required in production")]
    MissingJwtSecret,
}
