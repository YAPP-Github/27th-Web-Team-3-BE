use axum::http::header::SET_COOKIE;
use axum::http::HeaderValue;

/// 쿠키 이름 상수
pub const ACCESS_TOKEN_COOKIE: &str = "access_token";
pub const REFRESH_TOKEN_COOKIE: &str = "refresh_token";

/// Access Token 쿠키 생성
pub fn create_access_token_cookie(token: &str, max_age_seconds: i64) -> HeaderValue {
    let cookie = format!(
        "{}={}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age={}",
        ACCESS_TOKEN_COOKIE, token, max_age_seconds
    );
    HeaderValue::from_str(&cookie).expect("Invalid cookie value")
}

/// Refresh Token 쿠키 생성
pub fn create_refresh_token_cookie(token: &str, max_age_seconds: i64) -> HeaderValue {
    let cookie = format!(
        "{}={}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age={}",
        REFRESH_TOKEN_COOKIE, token, max_age_seconds
    );
    HeaderValue::from_str(&cookie).expect("Invalid cookie value")
}

/// Access Token 쿠키 삭제 (만료 처리)
pub fn clear_access_token_cookie() -> HeaderValue {
    let cookie = format!(
        "{}=; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=0",
        ACCESS_TOKEN_COOKIE
    );
    HeaderValue::from_str(&cookie).expect("Invalid cookie value")
}

/// Refresh Token 쿠키 삭제 (만료 처리)
pub fn clear_refresh_token_cookie() -> HeaderValue {
    let cookie = format!(
        "{}=; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=0",
        REFRESH_TOKEN_COOKIE
    );
    HeaderValue::from_str(&cookie).expect("Invalid cookie value")
}

/// Set-Cookie 헤더 키
pub fn set_cookie_header() -> axum::http::HeaderName {
    SET_COOKIE
}
