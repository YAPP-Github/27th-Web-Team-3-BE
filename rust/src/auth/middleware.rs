use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::{header, request::Parts, StatusCode},
    middleware::Next,
    response::Response,
    RequestPartsExt,
};

use crate::common::error::AppError;
use super::{jwt::{verify_jwt, Claims}, service::AuthService};

/// 인증된 사용자 정보를 담는 구조체
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub email: String,
}

/// Request에서 JWT 토큰을 추출하고 검증하는 Extractor
#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Authorization 헤더에서 토큰 추출
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("인증 토큰이 필요합니다".to_string()))?;

        // Bearer 토큰 파싱
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("올바르지 않은 토큰 형식입니다".to_string()))?;

        // JWT 검증
        let claims = verify_jwt(token)?;

        Ok(AuthUser {
            user_id: claims.sub,
            email: claims.email,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::jwt::create_jwt;
    use axum::http::{Request, header};

    #[tokio::test]
    async fn test_auth_user_extraction() {
        unsafe {
            std::env::set_var("SECRET_KEY", "test_secret_key_123");
        }

        let token = create_jwt("user123".to_string(), "test@example.com".to_string()).unwrap();

        let req = Request::builder()
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(())
            .unwrap();

        let (mut parts, _) = req.into_parts();

        let auth_user = AuthUser::from_request_parts(&mut parts, &()).await;
        assert!(auth_user.is_ok());

        let user = auth_user.unwrap();
        assert_eq!(user.user_id, "user123");
        assert_eq!(user.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_missing_auth_header() {
        let req = Request::builder()
            .body(())
            .unwrap();

        let (mut parts, _) = req.into_parts();

        let result = AuthUser::from_request_parts(&mut parts, &()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_token_format() {
        let req = Request::builder()
            .header(header::AUTHORIZATION, "InvalidFormat token")
            .body(())
            .unwrap();

        let (mut parts, _) = req.into_parts();

        let result = AuthUser::from_request_parts(&mut parts, &()).await;
        assert!(result.is_err());
    }
}

