use axum::{
    async_trait, extract::FromRequestParts, http::header::AUTHORIZATION, http::request::Parts,
};

use crate::state::AppState;
use crate::utils::error::AppError;
use crate::utils::jwt::{decode_access_token, Claims};

/// 인증된 사용자 정보를 담는 Extractor
pub struct AuthUser(pub Claims);

impl AuthUser {
    /// JWT Claims에서 사용자 ID를 추출합니다.
    pub fn user_id(&self) -> Result<i64, AppError> {
        self.0
            .sub
            .parse()
            .map_err(|_| AppError::Unauthorized("유효하지 않은 사용자 ID입니다.".to_string()))
    }
}

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Authorization 헤더 추출
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or_else(|| AppError::Unauthorized("로그인이 필요합니다.".to_string()))?;

        // 헤더 값 문자열 변환
        let auth_header_str = auth_header
            .to_str()
            .map_err(|_| AppError::Unauthorized("잘못된 헤더 형식입니다.".to_string()))?;

        // Bearer 스키마 확인
        if !auth_header_str.starts_with("Bearer ") {
            return Err(AppError::Unauthorized(
                "토큰 형식이 올바르지 않습니다.".to_string(),
            ));
        }

        // 토큰 추출
        let token = &auth_header_str[7..];

        // 토큰 검증 및 디코딩 (access token만 허용)
        let claims = decode_access_token(token, &state.config.jwt_secret)?;

        Ok(AuthUser(claims))
    }
}
