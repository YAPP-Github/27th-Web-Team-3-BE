use serde::{Deserialize, Serialize};
use validator::Validate;
use utoipa::ToSchema;

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
    pub is_new_member: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signup_token: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessLoginResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: LoginResponse,
}