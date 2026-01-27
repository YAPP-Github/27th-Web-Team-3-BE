use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::domain::member::entity::member::SocialType;

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub social_type: SocialType, // JSON string "KAKAO" or "GOOGLE"

    #[validate(length(min = 1))]
    pub token: String, // Access Token from Provider or ID Token
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EmailLoginRequest {
    #[validate(email(message = "이메일 형식이 올바르지 않습니다"))]
    pub email: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessLoginResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: LoginResponse,
}

/// 로그아웃 요청 DTO
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LogoutRequest {
    /// 무효화할 Refresh Token
    #[validate(length(min = 1, message = "refreshToken은 필수입니다"))]
    pub refresh_token: String,
}

/// 토큰 갱신 요청 DTO
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TokenRefreshRequest {
    /// 갱신에 사용할 Refresh Token
    #[validate(length(min = 1, message = "refreshToken은 필수입니다"))]
    pub refresh_token: String,
}

/// 토큰 갱신 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TokenRefreshResponse {
    pub access_token: String,
    pub refresh_token: String,
}

/// Swagger용 토큰 갱신 성공 응답 타입
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessTokenRefreshResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: TokenRefreshResponse,
}

/// Swagger용 로그아웃 성공 응답 타입
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessLogoutResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: Option<()>,
}
