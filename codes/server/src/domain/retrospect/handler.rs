use axum::{
    extract::{Path, State},
    Json,
};

use crate::state::AppState;
use crate::utils::auth::AuthUser;
use crate::utils::error::AppError;
use crate::utils::BaseResponse;

use super::dto::{SubmitRetrospectRequest, SubmitRetrospectResponse};
use super::service::RetrospectService;

/// 회고 최종 제출 API (API-017)
///
/// 작성한 모든 답변(총 5개)을 최종 제출합니다.
/// 각 답변은 최대 1,000자까지 입력 가능하며, 제출 완료 시 회고 상태가 SUBMITTED로 변경됩니다.
#[utoipa::path(
    post,
    path = "/api/v1/retrospects/{retrospectId}/submit",
    params(
        ("retrospectId" = i64, Path, description = "제출할 회고의 고유 식별자")
    ),
    request_body = SubmitRetrospectRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "회고 제출이 성공적으로 완료되었습니다.", body = SuccessSubmitRetrospectResponse),
        (status = 400, description = "잘못된 요청 (답변 누락, 답변 길이 초과, 공백만 입력 등)", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "이미 제출 완료된 회고", body = ErrorResponse),
        (status = 404, description = "존재하지 않는 회고", body = ErrorResponse),
        (status = 500, description = "서버 내부 오류", body = ErrorResponse)
    ),
    tag = "Retrospect"
)]
pub async fn submit_retrospect(
    user: AuthUser,
    State(state): State<AppState>,
    Path(retrospect_id): Path<i64>,
    Json(req): Json<SubmitRetrospectRequest>,
) -> Result<Json<BaseResponse<SubmitRetrospectResponse>>, AppError> {
    // retrospectId 검증 (1 이상의 양수)
    if retrospect_id < 1 {
        return Err(AppError::BadRequest(
            "retrospectId는 1 이상의 양수여야 합니다.".to_string(),
        ));
    }

    // 사용자 ID 추출
    let user_id: i64 = user
        .0
        .sub
        .parse()
        .map_err(|_| AppError::Unauthorized("유효하지 않은 사용자 ID입니다.".to_string()))?;

    // 서비스 호출
    let result = RetrospectService::submit_retrospect(state, user_id, retrospect_id, req).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "회고 제출이 성공적으로 완료되었습니다.",
    )))
}
