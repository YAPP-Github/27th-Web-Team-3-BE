use crate::config::AppConfig;
use crate::error::AppError;

pub struct SecretKeyValidator;

impl SecretKeyValidator {
    pub fn validate(config: &AppConfig, secret_key: &str) -> Result<(), AppError> {
        if secret_key.is_empty() || secret_key != config.ai_secret_key {
            return Err(AppError::InvalidSecretKey);
        }
        Ok(())
    }
}

