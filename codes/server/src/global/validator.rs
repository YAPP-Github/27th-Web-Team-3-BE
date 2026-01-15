use crate::error::AppError;
use subtle::ConstantTimeEq;

#[derive(Clone)]
pub struct SecretKeyValidator {
    expected_key: String,
}

impl SecretKeyValidator {
    pub fn new(expected_key: String) -> Self {
        Self { expected_key }
    }

    /// Validates the provided secret key using constant-time comparison
    /// to prevent timing attacks.
    pub fn validate(&self, provided_key: &str) -> Result<(), AppError> {
        let expected = self.expected_key.as_bytes();
        let provided = provided_key.as_bytes();

        // Check length equality first (will fail anyway if different)
        let length_matches = expected.len() == provided.len();

        // Perform constant-time comparison on the shorter length
        // This ensures timing doesn't leak information about the key
        let min_len = std::cmp::min(expected.len(), provided.len());
        let content_matches = expected[..min_len].ct_eq(&provided[..min_len]).unwrap_u8() == 1;

        if !length_matches || !content_matches {
            tracing::warn!(
                event = "invalid_secret_key_attempt",
                "Invalid secret key attempt detected"
            );
            return Err(AppError::InvalidSecretKey);
        }

        tracing::debug!("Secret key validated successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_pass_with_correct_key() {
        let validator = SecretKeyValidator::new("correct-key".to_string());
        let result = validator.validate("correct-key");

        assert!(result.is_ok());
    }

    #[test]
    fn should_fail_with_incorrect_key() {
        let validator = SecretKeyValidator::new("correct-key".to_string());
        let result = validator.validate("wrong-key");

        assert!(result.is_err());
        assert!(matches!(result, Err(AppError::InvalidSecretKey)));
    }

    #[test]
    fn should_fail_with_empty_key() {
        let validator = SecretKeyValidator::new("correct-key".to_string());
        let result = validator.validate("");

        assert!(result.is_err());
        assert!(matches!(result, Err(AppError::InvalidSecretKey)));
    }

    #[test]
    fn should_fail_with_shorter_key() {
        let validator = SecretKeyValidator::new("secret123".to_string());
        let result = validator.validate("secret");

        assert!(result.is_err());
        assert!(matches!(result, Err(AppError::InvalidSecretKey)));
    }

    #[test]
    fn should_fail_with_longer_key() {
        let validator = SecretKeyValidator::new("secret123".to_string());
        let result = validator.validate("secret123456");

        assert!(result.is_err());
        assert!(matches!(result, Err(AppError::InvalidSecretKey)));
    }

    #[test]
    fn should_fail_with_same_length_but_different_content() {
        let validator = SecretKeyValidator::new("secret123".to_string());
        let result = validator.validate("secret456");

        assert!(result.is_err());
        assert!(matches!(result, Err(AppError::InvalidSecretKey)));
    }
}
