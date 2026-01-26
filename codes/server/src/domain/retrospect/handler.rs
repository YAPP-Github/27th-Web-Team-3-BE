use axum::{
    extract::{Path, State},
    Json,
};

use crate::state::AppState;
use crate::utils::auth::AuthUser;
use crate::utils::error::AppError;
use crate::utils::BaseResponse;

use super::dto::{DraftSaveRequest, DraftSaveResponse};
use super::service::RetrospectService;

/// 회고 답변 임시 저장 API (API-016)
///
/// 진행 중인 회고의 답변을 임시로 저장합니다.
/// 기존에 저장된 내용이 있다면 전달받은 내용으로 덮어쓰기 처리됩니다.
#[utoipa::path(
    put,
    path = "/api/v1/retrospects/{retrospectId}/drafts",
    params(
        ("retrospectId" = i64, Path, description = "임시 저장할 회고의 고유 식별자")
    ),
    request_body = DraftSaveRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "임시 저장이 완료되었습니다.", body = SuccessDraftSaveResponse),
        (status = 400, description = "잘못된 요청 (답변 길이 초과, 잘못된 질문 번호, 빈 배열, 중복 질문 번호)", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "작성 권한 없음", body = ErrorResponse),
        (status = 404, description = "존재하지 않는 회고", body = ErrorResponse),
        (status = 500, description = "서버 내부 오류", body = ErrorResponse)
    ),
    tag = "Retrospect"
)]
pub async fn save_draft(
    user: AuthUser,
    State(state): State<AppState>,
    Path(retrospect_id): Path<i64>,
    Json(req): Json<DraftSaveRequest>,
) -> Result<Json<BaseResponse<DraftSaveResponse>>, AppError> {
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
    let result = RetrospectService::save_draft(state, user_id, retrospect_id, req).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "임시 저장이 완료되었습니다.",
    )))
}
