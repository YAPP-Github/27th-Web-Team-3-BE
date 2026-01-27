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
    /// Token Type ("access" or "refresh")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,
}

/// JWT 토큰 생성 (token_type 포함)
pub fn encode_token_with_type(
    sub: String,
    secret: &str,
    expiration_seconds: i64,
    token_type: &str,
) -> Result<String, AppError> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::seconds(expiration_seconds))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub,
        iat: Utc::now().timestamp() as usize,
        exp: expiration,
        token_type: Some(token_type.to_string()),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::InternalError(format!("Token creation failed: {}", e)))
}

/// JWT 토큰 생성 (기존 호환용 - access 타입)
#[allow(dead_code)]
pub fn encode_token(
    sub: String,
    secret: &str,
    expiration_seconds: i64,
) -> Result<String, AppError> {
    encode_token_with_type(sub, secret, expiration_seconds, "access")
}

/// Access Token 생성
pub fn encode_access_token(
    sub: String,
    secret: &str,
    expiration_seconds: i64,
) -> Result<String, AppError> {
    encode_token_with_type(sub, secret, expiration_seconds, "access")
}

/// Refresh Token 생성
pub fn encode_refresh_token(
    sub: String,
    secret: &str,
    expiration_seconds: i64,
) -> Result<String, AppError> {
    encode_token_with_type(sub, secret, expiration_seconds, "refresh")
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

/// Access Token 전용 검증 (token_type == "access" 확인)
pub fn decode_access_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    let claims = decode_token(token, secret)?;
    match claims.token_type.as_deref() {
        Some("access") => Ok(claims),
        _ => Err(AppError::Unauthorized(
            "유효하지 않은 액세스 토큰입니다.".into(),
        )),
    }
}

/// Refresh Token 전용 검증 (token_type == "refresh" 확인)
pub fn decode_refresh_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    let claims = decode_token(token, secret)?;
    match claims.token_type.as_deref() {
        Some("refresh") => Ok(claims),
        _ => Err(AppError::Unauthorized(
            "유효하지 않은 리프레시 토큰입니다.".into(),
        )),
    }
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

    #[test]
    fn test_access_token_validation() {
        let secret = "test_secret";
        let sub = "user_123".to_string();
        let expiration = 3600;

        let token =
            encode_access_token(sub.clone(), secret, expiration).expect("Token generation failed");
        let claims = decode_access_token(&token, secret).expect("Token validation failed");

        assert_eq!(claims.sub, sub);
        assert_eq!(claims.token_type, Some("access".to_string()));
    }

    #[test]
    fn test_refresh_token_validation() {
        let secret = "test_secret";
        let sub = "user_123".to_string();
        let expiration = 3600;

        let token =
            encode_refresh_token(sub.clone(), secret, expiration).expect("Token generation failed");
        let claims = decode_refresh_token(&token, secret).expect("Token validation failed");

        assert_eq!(claims.sub, sub);
        assert_eq!(claims.token_type, Some("refresh".to_string()));
    }

    #[test]
    fn test_refresh_token_rejected_as_access() {
        let secret = "test_secret";
        let sub = "user_123".to_string();
        let expiration = 3600;

        let refresh_token =
            encode_refresh_token(sub, secret, expiration).expect("Token generation failed");
        let result = decode_access_token(&refresh_token, secret);

        assert!(result.is_err());
    }

    #[test]
    fn test_access_token_rejected_as_refresh() {
        let secret = "test_secret";
        let sub = "user_123".to_string();
        let expiration = 3600;

        let access_token =
            encode_access_token(sub, secret, expiration).expect("Token generation failed");
        let result = decode_refresh_token(&access_token, secret);

        assert!(result.is_err());
    }
}
