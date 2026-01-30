use serde::Serialize;
use utoipa::ToSchema;

use crate::utils::BaseResponse;

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
