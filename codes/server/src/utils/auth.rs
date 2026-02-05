use axum::{
    async_trait, extract::FromRequestParts, http::header::AUTHORIZATION, http::header::COOKIE,
    http::request::Parts,
};

use crate::state::AppState;
use crate::utils::cookie::ACCESS_TOKEN_COOKIE;
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
        // 1. Authorization 헤더에서 토큰 추출 시도
        let token = if let Some(auth_header) = parts.headers.get(AUTHORIZATION) {
            let auth_header_str = auth_header
                .to_str()
                .map_err(|_| AppError::Unauthorized("잘못된 헤더 형식입니다.".to_string()))?;

            if !auth_header_str.starts_with("Bearer ") {
                return Err(AppError::Unauthorized(
                    "토큰 형식이 올바르지 않습니다.".to_string(),
                ));
            }

            auth_header_str[7..].to_string()
        } else {
            // 2. 쿠키에서 토큰 추출 시도
            extract_token_from_cookie(parts)?
        };

        // 토큰 검증 및 디코딩 (access token만 허용)
        let claims = decode_access_token(&token, &state.config.jwt_secret)?;

        Ok(AuthUser(claims))
    }
}

/// 쿠키에서 access_token 추출
fn extract_token_from_cookie(parts: &Parts) -> Result<String, AppError> {
    let cookie_header = parts
        .headers
        .get(COOKIE)
        .ok_or_else(|| AppError::Unauthorized("로그인이 필요합니다.".to_string()))?;

    let cookie_str = cookie_header
        .to_str()
        .map_err(|_| AppError::Unauthorized("잘못된 쿠키 형식입니다.".to_string()))?;

    // 쿠키 파싱: "name1=value1; name2=value2" 형식
    for cookie in cookie_str.split(';') {
        let cookie = cookie.trim();
        if let Some(value) = cookie.strip_prefix(&format!("{}=", ACCESS_TOKEN_COOKIE)) {
            if !value.is_empty() {
                return Ok(value.to_string());
            }
        }
    }

    Err(AppError::Unauthorized("로그인이 필요합니다.".to_string()))
}
