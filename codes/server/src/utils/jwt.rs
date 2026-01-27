use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use super::error::AppError;

/// JWT Claims 구조체
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (User ID)
    pub sub: String,
    /// Issued At
    pub iat: usize,
    /// Expiration
    pub exp: usize,
    /// Email (for signup token)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Token Type (access, refresh, signup)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,
}

/// JWT 토큰 생성 (Access Token)
pub fn encode_token(
    sub: String,
    secret: &str,
    expiration_seconds: i64,
) -> Result<String, AppError> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::seconds(expiration_seconds))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub,
        iat: Utc::now().timestamp() as usize,
        exp: expiration,
        email: None,
        token_type: Some("access".to_string()),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::InternalError(format!("Token creation failed: {}", e)))
}

/// Refresh Token 생성
pub fn encode_refresh_token(
    sub: String,
    secret: &str,
    expiration_seconds: i64,
) -> Result<String, AppError> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::seconds(expiration_seconds))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub,
        iat: Utc::now().timestamp() as usize,
        exp: expiration,
        email: None,
        token_type: Some("refresh".to_string()),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::InternalError(format!("Refresh token creation failed: {}", e)))
}

/// Signup Token 생성
pub fn encode_signup_token(
    email: String,
    secret: &str,
    expiration_seconds: i64,
) -> Result<String, AppError> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::seconds(expiration_seconds))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: "".to_string(), // No user ID yet
        iat: Utc::now().timestamp() as usize,
        exp: expiration,
        email: Some(email),
        token_type: Some("signup".to_string()),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::InternalError(format!("Signup token creation failed: {}", e)))
}

/// JWT 토큰 검증
pub fn decode_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    let validation = Validation::default();

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
            AppError::Unauthorized("토큰이 만료되었습니다.".into())
        }
        _ => AppError::Unauthorized("유효하지 않은 토큰입니다.".into()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_and_decode() {
        let secret = "test_secret";
        let sub = "user_123".to_string();
        let expiration = 3600;

        let token = encode_token(sub.clone(), secret, expiration).expect("Token generation failed");
        let claims = decode_token(&token, secret).expect("Token validation failed");

        assert_eq!(claims.sub, sub);
    }

    #[test]
    fn test_invalid_token() {
        let secret = "test_secret";
        let result = decode_token("invalid_token", secret);
        assert!(result.is_err());
    }
}
