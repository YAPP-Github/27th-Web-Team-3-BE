use axum::{
    extract::{Path, Query, State},
    Json,
};
use validator::Validate;

use crate::state::AppState;
use crate::utils::auth::AuthUser;
use crate::utils::error::AppError;
use crate::utils::BaseResponse;

use super::dto::{
    AnalysisResponse, CreateCommentRequest, CreateCommentResponse, CreateParticipantResponse,
    CreateRetrospectRequest, CreateRetrospectResponse, DraftSaveRequest, DraftSaveResponse,
    ListCommentsQuery, ListCommentsResponse, ReferenceItem, RetrospectDetailResponse,
    StorageQueryParams, StorageResponse, SubmitRetrospectRequest, SubmitRetrospectResponse,
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

/// 회고 분석 API (API-022)
///
/// 특정 회고 세션에 쌓인 모든 팀원의 답변을 종합 분석하여 AI 인사이트, 감정 통계, 맞춤형 미션을 생성합니다.
#[utoipa::path(
    post,
    path = "/api/v1/retrospects/{retrospectId}/analysis",
    params(
        ("retrospectId" = i64, Path, description = "분석할 회고 ID")
    ),
    request_body = (),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "회고 분석 성공", body = SuccessAnalysisResponse),
        (status = 400, description = "잘못된 Path Parameter", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "월간 한도 초과 또는 접근 권한 없음", body = ErrorResponse),
        (status = 404, description = "회고 없음", body = ErrorResponse),
        (status = 409, description = "이미 분석 완료된 회고", body = ErrorResponse),
        (status = 422, description = "분석 데이터 부족", body = ErrorResponse),
        (status = 500, description = "AI 분석 실패", body = ErrorResponse)
    ),
    tag = "Retrospect"
)]
pub async fn analyze_retrospective_handler(
    user: AuthUser,
    State(state): State<AppState>,
    Path(retrospect_id): Path<i64>,
) -> Result<Json<BaseResponse<AnalysisResponse>>, AppError> {
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
    let result = RetrospectService::analyze_retrospective(state, user_id, retrospect_id).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "회고 분석이 성공적으로 완료되었습니다.",
    )))
}

/// 회고 답변 댓글 목록 조회 API (API-026)
///
/// 특정 회고 답변에 작성된 댓글 리스트를 조회합니다.
/// 커서 기반 페이지네이션을 사용하여 무한 스크롤을 지원합니다.
#[utoipa::path(
    get,
    path = "/api/v1/responses/{responseId}/comments",
    params(
        ("responseId" = i64, Path, description = "댓글을 조회할 회고 답변의 고유 식별자"),
        ("cursor" = Option<i64>, Query, description = "마지막으로 조회된 댓글 ID (첫 요청 시 생략)"),
        ("size" = Option<i32>, Query, description = "페이지당 조회 개수 (1~100, 기본값: 20)")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "댓글 조회를 성공했습니다.", body = SuccessListCommentsResponse),
        (status = 400, description = "잘못된 요청 (responseId, cursor, size 유효성 오류)", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "접근 권한 없음", body = ErrorResponse),
        (status = 404, description = "존재하지 않는 회고 답변", body = ErrorResponse),
        (status = 500, description = "서버 내부 오류", body = ErrorResponse)
    ),
    tag = "Response"
)]
pub async fn list_comments(
    user: AuthUser,
    State(state): State<AppState>,
    Path(response_id): Path<i64>,
    Query(query): Query<ListCommentsQuery>,
) -> Result<Json<BaseResponse<ListCommentsResponse>>, AppError> {
    // responseId 검증 (1 이상의 양수)
    if response_id < 1 {
        return Err(AppError::BadRequest(
            "responseId는 1 이상의 양수여야 합니다.".to_string(),
        ));
    }

    // cursor 검증 (있으면 1 이상)
    if let Some(cursor) = query.cursor {
        if cursor < 1 {
            return Err(AppError::BadRequest(
                "cursor는 1 이상의 양수여야 합니다.".to_string(),
            ));
        }
    }

    // size 검증 (1~100, 기본값 20)
    let size = query.size.unwrap_or(20);
    if !(1..=100).contains(&size) {
        return Err(AppError::BadRequest(
            "size는 1~100 범위의 정수여야 합니다.".to_string(),
        ));
    }

    // 사용자 ID 추출
    let user_id: i64 = user
        .0
        .sub
        .parse()
        .map_err(|_| AppError::Unauthorized("유효하지 않은 사용자 ID입니다.".to_string()))?;

    // 서비스 호출
    let result =
        RetrospectService::list_comments(state, user_id, response_id, query.cursor, size).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "댓글 조회를 성공했습니다.",
    )))
}

/// 회고 답변 댓글 작성 API (API-027)
///
/// 동료의 회고 답변에 댓글(의견)을 남깁니다.
/// 댓글 내용은 최대 200자까지 작성이 가능합니다.
#[utoipa::path(
    post,
    path = "/api/v1/responses/{responseId}/comments",
    params(
        ("responseId" = i64, Path, description = "댓글을 작성할 대상 답변의 고유 ID")
    ),
    request_body = CreateCommentRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "댓글이 성공적으로 등록되었습니다.", body = SuccessCreateCommentResponse),
        (status = 400, description = "잘못된 요청 (content 필드 누락, 길이 초과)", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "권한 없음", body = ErrorResponse),
        (status = 404, description = "존재하지 않는 회고 답변", body = ErrorResponse),
        (status = 500, description = "서버 내부 오류", body = ErrorResponse)
    ),
    tag = "Response"
)]
pub async fn create_comment(
    user: AuthUser,
    State(state): State<AppState>,
    Path(response_id): Path<i64>,
    Json(req): Json<CreateCommentRequest>,
) -> Result<Json<BaseResponse<CreateCommentResponse>>, AppError> {
    // responseId 검증 (1 이상의 양수)
    if response_id < 1 {
        return Err(AppError::BadRequest(
            "responseId는 1 이상의 양수여야 합니다.".to_string(),
        ));
    }

    // 입력값 검증
    req.validate()?;

    // 사용자 ID 추출
    let user_id: i64 = user
        .0
        .sub
        .parse()
        .map_err(|_| AppError::Unauthorized("유효하지 않은 사용자 ID입니다.".to_string()))?;

    // 서비스 호출
    let result = RetrospectService::create_comment(state, user_id, response_id, req).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "댓글이 성공적으로 등록되었습니다.",
    )))
}
