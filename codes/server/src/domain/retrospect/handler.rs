use axum::{
    extract::{Path, State},
    Json,
};
use validator::Validate;

use crate::state::AppState;
use crate::utils::auth::AuthUser;
use crate::utils::error::AppError;
use crate::utils::BaseResponse;

use super::dto::{
    CreateParticipantResponse, CreateRetrospectRequest, CreateRetrospectResponse, ReferenceItem,
    TeamRetrospectListItem,
};
use super::service::RetrospectService;

/// 회고 생성 API
///
/// 진행한 프로젝트에 대한 회고 세션을 생성합니다.
/// 프로젝트 정보, 회고 방식, 참고 자료 등을 포함하며 생성된 회고의 고유 식별자를 반환합니다.
#[utoipa::path(
    post,
    path = "/api/v1/retrospects",
    request_body = CreateRetrospectRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "회고가 성공적으로 생성되었습니다.", body = SuccessCreateRetrospectResponse),
        (status = 400, description = "잘못된 요청 (프로젝트 이름 길이 초과, 날짜 형식 오류, URL 형식 오류 등)", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "팀 접근 권한 없음", body = ErrorResponse),
        (status = 404, description = "존재하지 않는 팀", body = ErrorResponse),
        (status = 500, description = "서버 내부 오류", body = ErrorResponse)
    ),
    tag = "Retrospect"
)]
pub async fn create_retrospect(
    user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateRetrospectRequest>,
) -> Result<Json<BaseResponse<CreateRetrospectResponse>>, AppError> {
    // 입력값 검증
    req.validate()?;

    // 사용자 ID 추출
    let user_id: i64 = user
        .0
        .sub
        .parse()
        .map_err(|_| AppError::Unauthorized("유효하지 않은 사용자 ID입니다.".to_string()))?;

    // 서비스 호출
    let result = RetrospectService::create_retrospect(state, user_id, req).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "회고가 성공적으로 생성되었습니다.",
    )))
}

/// 팀 회고 목록 조회 API (API-010)
///
/// 특정 팀에 속한 모든 회고 목록을 조회합니다.
/// 과거, 오늘, 예정된 회고 데이터가 모두 포함되며 최신순으로 정렬됩니다.
#[utoipa::path(
    get,
    path = "/api/v1/teams/{teamId}/retrospects",
    params(
        ("teamId" = i64, Path, description = "조회를 원하는 팀의 고유 ID")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "팀 내 전체 회고 목록 조회를 성공했습니다.", body = SuccessTeamRetrospectListResponse),
        (status = 400, description = "잘못된 요청 (team_id는 1 이상이어야 합니다.)", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "팀 접근 권한 없음", body = ErrorResponse),
        (status = 404, description = "존재하지 않는 팀", body = ErrorResponse),
        (status = 500, description = "서버 내부 오류", body = ErrorResponse)
    ),
    tag = "Retrospect"
)]
pub async fn list_team_retrospects(
    user: AuthUser,
    State(state): State<AppState>,
    Path(team_id): Path<i64>,
) -> Result<Json<BaseResponse<Vec<TeamRetrospectListItem>>>, AppError> {
    // teamId 검증 (1 이상의 양수)
    if team_id < 1 {
        return Err(AppError::BadRequest(
            "팀 ID는 1 이상이어야 합니다.".to_string(),
        ));
    }

    // 사용자 ID 추출
    let user_id: i64 = user
        .0
        .sub
        .parse()
        .map_err(|_| AppError::Unauthorized("유효하지 않은 사용자 ID입니다.".to_string()))?;

    // 서비스 호출
    let result = RetrospectService::list_team_retrospects(state, user_id, team_id).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "팀 내 전체 회고 목록 조회를 성공했습니다.",
    )))
}

/// 회고 참석자 등록 API (API-014)
///
/// 진행 예정인 회고에 참석자로 등록합니다.
/// JWT의 유저 정보를 기반으로 참석을 처리하며, 해당 회고가 속한 팀의 멤버만 참석이 가능합니다.
#[utoipa::path(
    post,
    path = "/api/v1/retrospects/{retrospectId}/participants",
    params(
        ("retrospectId" = i64, Path, description = "참여하고자 하는 회고의 고유 ID")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "회고 참석자로 성공적으로 등록되었습니다.", body = SuccessCreateParticipantResponse),
        (status = 400, description = "잘못된 요청 (retrospectId 유효성 오류)", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 404, description = "존재하지 않는 회고이거나 접근 권한 없음", body = ErrorResponse),
        (status = 409, description = "중복 참석", body = ErrorResponse),
        (status = 500, description = "서버 내부 오류", body = ErrorResponse)
    ),
    tag = "Retrospect"
)]
pub async fn create_participant(
    user: AuthUser,
    State(state): State<AppState>,
    Path(retrospect_id): Path<i64>,
) -> Result<Json<BaseResponse<CreateParticipantResponse>>, AppError> {
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
    let result = RetrospectService::create_participant(state, user_id, retrospect_id).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "회고 참석자로 성공적으로 등록되었습니다.",
    )))
}

/// 회고 참고자료 목록 조회 API (API-018)
///
/// 특정 회고에 등록된 모든 참고자료(URL) 목록을 조회합니다.
/// 회고 생성 시 등록했던 외부 링크들을 확인할 수 있습니다.
#[utoipa::path(
    get,
    path = "/api/v1/retrospects/{retrospectId}/references",
    params(
        ("retrospectId" = i64, Path, description = "조회를 원하는 회고의 고유 ID")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "참고자료 목록을 성공적으로 조회했습니다.", body = SuccessReferencesListResponse),
        (status = 400, description = "잘못된 요청 (retrospectId 유효성 오류)", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 404, description = "존재하지 않는 회고이거나 접근 권한 없음", body = ErrorResponse),
        (status = 500, description = "서버 내부 오류", body = ErrorResponse)
    ),
    tag = "Retrospect"
)]
pub async fn list_references(
    user: AuthUser,
    State(state): State<AppState>,
    Path(retrospect_id): Path<i64>,
) -> Result<Json<BaseResponse<Vec<ReferenceItem>>>, AppError> {
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
    let result = RetrospectService::list_references(state, user_id, retrospect_id).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "참고자료 목록을 성공적으로 조회했습니다.",
    )))
}
