use axum::{extract::State, Json};

use super::service::MemberService;
use crate::state::AppState;
use crate::utils::auth::AuthUser;
use crate::utils::error::AppError;
use crate::utils::BaseResponse;

/// 서비스 탈퇴 API (API-028)
///
/// 현재 로그인한 사용자의 계정을 삭제하고 서비스를 탈퇴 처리합니다.
/// - 탈퇴 시 해당 사용자와 연결된 모든 개인 정보 및 데이터는 즉시 파기되며, 이는 복구가 불가능합니다.
#[utoipa::path(
    delete,
    path = "/api/v1/members/me",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "회원 탈퇴 성공", body = SuccessWithdrawResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 404, description = "존재하지 않는 사용자", body = ErrorResponse),
        (status = 500, description = "서버 내부 오류", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn withdraw(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<BaseResponse<()>>, AppError> {
    let member_id: i64 = user
        .0
        .sub
        .parse()
        .map_err(|_| AppError::Unauthorized("잘못된 인증 정보입니다.".into()))?;

    MemberService::withdraw(state, member_id).await?;

    Ok(Json(BaseResponse {
        is_success: true,
        code: "COMMON200".to_string(),
        message: "회원 탈퇴가 성공적으로 완료되었습니다.".to_string(),
        result: None,
    }))
}
