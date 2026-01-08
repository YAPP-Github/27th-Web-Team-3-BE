use crate::config::AppConfig;
use crate::error::AppError;

/// secretKey 유효성 검증
pub fn validate_secret_key(secret_key: &str, config: &AppConfig) -> Result<(), AppError> {
    if secret_key != config.ai_secret_key {
        return Err(AppError::InvalidSecretKey);
    }
    Ok(())
}

