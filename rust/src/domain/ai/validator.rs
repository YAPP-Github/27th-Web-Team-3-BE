use crate::config::AppConfig;
use crate::error::AppError;

/// Validates the secret key
pub fn validate_secret_key(secret_key: &str, config: &AppConfig) -> Result<(), AppError> {
    if secret_key != config.ai_secret_key {
        return Err(AppError::InvalidSecretKey);
    }
    Ok(())
}

