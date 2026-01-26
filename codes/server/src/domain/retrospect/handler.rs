use axum::{
    extract::{Path, Query, State},
    Json,
};

use crate::state::AppState;
use crate::utils::auth::AuthUser;
use crate::utils::error::AppError;
use crate::utils::BaseResponse;

use super::dto::{
    DraftSaveRequest, DraftSaveResponse, RetrospectDetailResponse, StorageQueryParams,
    StorageResponse, SubmitRetrospectRequest, SubmitRetrospectResponse,
};
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

/// 회고 상세 정보 조회 API (API-012)
///
/// 특정 회고 세션의 상세 정보(제목, 일시, 유형, 참여 멤버, 질문 리스트 및 전체 통계)를 조회합니다.
#[utoipa::path(
    get,
    path = "/api/v1/retrospects/{retrospectId}",
    params(
        ("retrospectId" = i64, Path, description = "조회할 회고의 고유 식별자")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "회고 상세 정보 조회를 성공했습니다.", body = SuccessRetrospectDetailResponse),
        (status = 400, description = "잘못된 Path Parameter", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "접근 권한 없음", body = ErrorResponse),
        (status = 404, description = "존재하지 않는 회고", body = ErrorResponse),
        (status = 500, description = "서버 내부 오류", body = ErrorResponse)
    ),
    tag = "Retrospect"
)]
pub async fn get_retrospect_detail(
    user: AuthUser,
    State(state): State<AppState>,
    Path(retrospect_id): Path<i64>,
) -> Result<Json<BaseResponse<RetrospectDetailResponse>>, AppError> {
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
    let result = RetrospectService::get_retrospect_detail(state, user_id, retrospect_id).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "회고 상세 정보 조회를 성공했습니다.",
    )))
}

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

/// 보관함 조회 API (API-019)
///
/// 완료된 회고 목록을 연도별로 그룹화하여 조회합니다.
/// 기간 필터를 통해 특정 기간의 회고만 조회할 수 있습니다.
#[utoipa::path(
    get,
    path = "/api/v1/retrospects/storage",
    params(StorageQueryParams),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "보관함 조회를 성공했습니다.", body = SuccessStorageResponse),
        (status = 400, description = "유효하지 않은 기간 필터", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 500, description = "서버 내부 오류", body = ErrorResponse)
    ),
    tag = "Retrospect"
)]
pub async fn get_storage(
    user: AuthUser,
    State(state): State<AppState>,
    Query(params): Query<StorageQueryParams>,
) -> Result<Json<BaseResponse<StorageResponse>>, AppError> {
    // 사용자 ID 추출
    let user_id: i64 = user
        .0
        .sub
        .parse()
        .map_err(|_| AppError::Unauthorized("유효하지 않은 사용자 ID입니다.".to_string()))?;

    // 서비스 호출
    let result = RetrospectService::get_storage(state, user_id, params).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "보관함 조회를 성공했습니다.",
    )))
}
