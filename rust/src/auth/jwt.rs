use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;

use crate::common::error::AppError;

/// JWT Claims 구조체
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,        // Subject (user id)
    pub email: String,      // User email
    pub exp: i64,          // Expiration time
    pub iat: i64,          // Issued at
}

impl Claims {
    /// 새로운 Claims 생성
    pub fn new(user_id: String, email: String, expires_in_hours: i64) -> Self {
        let now = Utc::now();
        let expiration = now + Duration::hours(expires_in_hours);

        Self {
            sub: user_id,
            email,
            exp: expiration.timestamp(),
            iat: now.timestamp(),
        }
    }
}

/// JWT 토큰 생성
pub fn create_jwt(user_id: String, email: String) -> Result<String, AppError> {
    let secret = env::var("SECRET_KEY")
        .map_err(|_| AppError::InternalError("SECRET_KEY not configured".to_string()))?;

    let claims = Claims::new(user_id, email, 24); // 24시간 유효

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}

/// JWT 토큰 검증 및 Claims 반환
pub fn verify_jwt(token: &str) -> Result<Claims, AppError> {
    let secret = env::var("SECRET_KEY")
        .map_err(|_| AppError::InternalError("SECRET_KEY not configured".to_string()))?;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

/// Refresh 토큰 생성 (7일 유효)
pub fn create_refresh_token(user_id: String, email: String) -> Result<String, AppError> {
    let secret = env::var("SECRET_KEY")
        .map_err(|_| AppError::InternalError("SECRET_KEY not configured".to_string()))?;

    let claims = Claims::new(user_id, email, 24 * 7); // 7일 유효

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_create_and_verify_jwt() {
        unsafe {
            std::env::set_var("SECRET_KEY", "test_secret_key_123");
        }

        let user_id = "user123".to_string();
        let email = "test@example.com".to_string();

        let token = create_jwt(user_id.clone(), email.clone()).unwrap();
        assert!(!token.is_empty());

        let claims = verify_jwt(&token).unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.email, email);
    }

    #[test]
    fn test_invalid_token() {
        unsafe {
            std::env::set_var("SECRET_KEY", "test_secret_key_123");
        }

        let result = verify_jwt("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_claims_creation() {
        let user_id = "user123".to_string();
        let email = "test@example.com".to_string();

        let claims = Claims::new(user_id.clone(), email.clone(), 24);

        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.email, email);
        assert!(claims.exp > claims.iat);
    }
}

