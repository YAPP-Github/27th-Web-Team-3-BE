use axum::http::header::SET_COOKIE;
use axum::http::HeaderValue;

use crate::utils::error::AppError;

/// 쿠키 이름 상수
pub const ACCESS_TOKEN_COOKIE: &str = "access_token";
pub const REFRESH_TOKEN_COOKIE: &str = "refresh_token";
pub const SIGNUP_TOKEN_COOKIE: &str = "signup_token";

/// 공통 쿠키 생성 헬퍼 함수
fn build_cookie(name: &str, value: &str, max_age_seconds: i64) -> Result<HeaderValue, AppError> {
    let cookie = format!(
        "{}={}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age={}",
        name, value, max_age_seconds
    );
    HeaderValue::from_str(&cookie)
        .map_err(|_| AppError::InternalError(format!("Invalid {} cookie value", name)))
}

/// Access Token 쿠키 생성
pub fn create_access_token_cookie(
    token: &str,
    max_age_seconds: i64,
) -> Result<HeaderValue, AppError> {
    build_cookie(ACCESS_TOKEN_COOKIE, token, max_age_seconds)
}

/// Refresh Token 쿠키 생성
pub fn create_refresh_token_cookie(
    token: &str,
    max_age_seconds: i64,
) -> Result<HeaderValue, AppError> {
    build_cookie(REFRESH_TOKEN_COOKIE, token, max_age_seconds)
}

/// Signup Token 쿠키 생성 (짧은 TTL)
pub fn create_signup_token_cookie(
    token: &str,
    max_age_seconds: i64,
) -> Result<HeaderValue, AppError> {
    build_cookie(SIGNUP_TOKEN_COOKIE, token, max_age_seconds)
}

/// Access Token 쿠키 삭제 (만료 처리)
pub fn clear_access_token_cookie() -> Result<HeaderValue, AppError> {
    build_cookie(ACCESS_TOKEN_COOKIE, "", 0)
}

/// Refresh Token 쿠키 삭제 (만료 처리)
pub fn clear_refresh_token_cookie() -> Result<HeaderValue, AppError> {
    build_cookie(REFRESH_TOKEN_COOKIE, "", 0)
}

/// Signup Token 쿠키 삭제 (만료 처리)
pub fn clear_signup_token_cookie() -> Result<HeaderValue, AppError> {
    build_cookie(SIGNUP_TOKEN_COOKIE, "", 0)
}

/// Set-Cookie 헤더 키
pub fn set_cookie_header() -> axum::http::HeaderName {
    SET_COOKIE
}
