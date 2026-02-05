use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use super::entity::member::SocialType;
use crate::utils::BaseResponse;

/// 회원 프로필 응답
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MemberProfileResponse {
    pub member_id: i64,
    pub email: String,
    pub nickname: Option<String>,
    pub insight_count: i32,
    pub social_type: SocialType,
    pub created_at: DateTime<Utc>,
}

/// 회원 프로필 조회 성공 응답 (Swagger 문서용)
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessProfileResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: MemberProfileResponse,
}

/// 회원 탈퇴 성공 응답 (Swagger 문서용)
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessWithdrawResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: Option<()>,
}

impl From<BaseResponse<()>> for SuccessWithdrawResponse {
    fn from(res: BaseResponse<()>) -> Self {
        Self {
            is_success: res.is_success,
            code: res.code,
            message: res.message,
            result: res.result,
        }
    }
}
