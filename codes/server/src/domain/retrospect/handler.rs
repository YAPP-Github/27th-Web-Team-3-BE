use axum::{
    extract::{Path, State},
    Json,
};
use validator::Validate;

use super::dto::{
    DeleteRetroRoomResponse, JoinRetroRoomRequest, JoinRetroRoomResponse, RetroRoomCreateRequest,
    RetroRoomCreateResponse, RetroRoomListItem, RetrospectListItem, UpdateRetroRoomNameRequest,
    UpdateRetroRoomNameResponse, UpdateRetroRoomOrderRequest,
};
use super::service::RetrospectService;
use crate::state::AppState;
use crate::utils::auth::AuthUser;
use crate::utils::error::AppError;
use crate::utils::BaseResponse;

/// 회고 룸 생성 API
///
/// 새로운 회고 룸을 생성하고 생성자를 관리자로 설정합니다.
#[utoipa::path(
    post,
    path = "/api/v1/retro-rooms",
    request_body = RetroRoomCreateRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "회고 룸 생성 성공", body = SuccessRetroRoomCreateResponse),
        (status = 400, description = "잘못된 요청", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 409, description = "이름 중복", body = ErrorResponse)
    ),
    tag = "RetroRoom"
)]
pub async fn create_retro_room(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<RetroRoomCreateRequest>,
) -> Result<Json<BaseResponse<RetroRoomCreateResponse>>, AppError> {
    req.validate()?;

    let member_id = user
        .0
        .sub
        .parse::<i64>()
        .map_err(|_| AppError::Unauthorized("유효하지 않은 사용자 정보입니다.".into()))?;

    let result = RetrospectService::create_retro_room(state, member_id, req).await?;

    Ok(Json(BaseResponse::success(result)))
}

/// 회고 룸 참여 API (초대 코드)
///
/// 초대 링크(코드)를 통해 회고 룸에 참여합니다.
#[utoipa::path(
    post,
    path = "/api/v1/retro-rooms/join",
    request_body = JoinRetroRoomRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "회고 룸 참여 성공", body = SuccessJoinRetroRoomResponse),
        (status = 400, description = "잘못된 초대 링크 또는 만료됨", body = ErrorResponse),
        (status = 404, description = "존재하지 않는 룸", body = ErrorResponse),
        (status = 409, description = "이미 참여 중", body = ErrorResponse)
    ),
    tag = "RetroRoom"
)]
pub async fn join_retro_room(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<JoinRetroRoomRequest>,
) -> Result<Json<BaseResponse<JoinRetroRoomResponse>>, AppError> {
    req.validate()?;

    let member_id = user
        .0
        .sub
        .parse::<i64>()
        .map_err(|_| AppError::Unauthorized("유효하지 않은 사용자 정보입니다.".into()))?;

    let result = RetrospectService::join_retro_room(state, member_id, req).await?;

    Ok(Json(BaseResponse::success(result)))
}

/// API-006: 참여 중인 레트로룸 목록 조회
///
/// 현재 로그인한 사용자가 참여 중인 모든 레트로룸 목록을 조회합니다.
#[utoipa::path(
    get,
    path = "/api/v1/retro-rooms",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "레트로룸 목록 조회 성공", body = SuccessRetroRoomListResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse)
    ),
    tag = "RetroRoom"
)]
pub async fn list_retro_rooms(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<BaseResponse<Vec<RetroRoomListItem>>>, AppError> {
    let member_id = user
        .0
        .sub
        .parse::<i64>()
        .map_err(|_| AppError::Unauthorized("유효하지 않은 사용자 정보입니다.".into()))?;

    let result = RetrospectService::list_retro_rooms(state, member_id).await?;

    Ok(Json(BaseResponse::success(result)))
}

/// API-007: 레트로룸 순서 변경
///
/// 드래그 앤 드롭으로 변경된 레트로룸들의 정렬 순서를 서버에 일괄 저장합니다.
#[utoipa::path(
    patch,
    path = "/api/v1/retro-rooms/order",
    request_body = UpdateRetroRoomOrderRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "순서 변경 성공", body = SuccessEmptyResponse),
        (status = 400, description = "잘못된 순서 데이터", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "권한 없음", body = ErrorResponse)
    ),
    tag = "RetroRoom"
)]
pub async fn update_retro_room_order(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<UpdateRetroRoomOrderRequest>,
) -> Result<Json<BaseResponse<()>>, AppError> {
    req.validate()?;

    let member_id = user
        .0
        .sub
        .parse::<i64>()
        .map_err(|_| AppError::Unauthorized("유효하지 않은 사용자 정보입니다.".into()))?;

    RetrospectService::update_retro_room_order(state, member_id, req).await?;

    Ok(Json(BaseResponse::success(())))
}

/// API-008: 레트로룸 이름 변경
///
/// 기존 레트로룸의 이름을 새로운 이름으로 변경합니다. (Owner만 가능)
#[utoipa::path(
    patch,
    path = "/api/v1/retro-rooms/{retro_room_id}/name",
    request_body = UpdateRetroRoomNameRequest,
    params(
        ("retro_room_id" = i64, Path, description = "레트로룸 ID")
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "이름 변경 성공", body = SuccessUpdateRetroRoomNameResponse),
        (status = 400, description = "이름 길이 초과", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "권한 없음", body = ErrorResponse),
        (status = 404, description = "룸 없음", body = ErrorResponse),
        (status = 409, description = "이름 중복", body = ErrorResponse)
    ),
    tag = "RetroRoom"
)]
pub async fn update_retro_room_name(
    State(state): State<AppState>,
    user: AuthUser,
    Path(retro_room_id): Path<i64>,
    Json(req): Json<UpdateRetroRoomNameRequest>,
) -> Result<Json<BaseResponse<UpdateRetroRoomNameResponse>>, AppError> {
    req.validate()?;

    let member_id = user
        .0
        .sub
        .parse::<i64>()
        .map_err(|_| AppError::Unauthorized("유효하지 않은 사용자 정보입니다.".into()))?;

    let result =
        RetrospectService::update_retro_room_name(state, member_id, retro_room_id, req).await?;

    Ok(Json(BaseResponse::success(result)))
}

/// API-009: 레트로룸 삭제
///
/// 레트로룸을 완전히 삭제합니다. (Owner만 가능)
#[utoipa::path(
    delete,
    path = "/api/v1/retro-rooms/{retro_room_id}",
    params(
        ("retro_room_id" = i64, Path, description = "레트로룸 ID")
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "삭제 성공", body = SuccessDeleteRetroRoomResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "권한 없음", body = ErrorResponse),
        (status = 404, description = "룸 없음", body = ErrorResponse)
    ),
    tag = "RetroRoom"
)]
pub async fn delete_retro_room(
    State(state): State<AppState>,
    user: AuthUser,
    Path(retro_room_id): Path<i64>,
) -> Result<Json<BaseResponse<DeleteRetroRoomResponse>>, AppError> {
    let member_id = user
        .0
        .sub
        .parse::<i64>()
        .map_err(|_| AppError::Unauthorized("유효하지 않은 사용자 정보입니다.".into()))?;

    let result = RetrospectService::delete_retro_room(state, member_id, retro_room_id).await?;

    Ok(Json(BaseResponse::success(result)))
}

/// API-010: 레트로룸 내 회고 목록 조회
///
/// 특정 레트로룸에 속한 모든 회고 목록을 조회합니다.
#[utoipa::path(
    get,
    path = "/api/v1/retro-rooms/{retro_room_id}/retrospects",
    params(
        ("retro_room_id" = i64, Path, description = "레트로룸 ID")
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "회고 목록 조회 성공", body = SuccessRetrospectListResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "권한 없음", body = ErrorResponse),
        (status = 404, description = "룸 없음", body = ErrorResponse)
    ),
    tag = "RetroRoom"
)]
pub async fn list_retrospects(
    State(state): State<AppState>,
    user: AuthUser,
    Path(retro_room_id): Path<i64>,
) -> Result<Json<BaseResponse<Vec<RetrospectListItem>>>, AppError> {
    let member_id = user
        .0
        .sub
        .parse::<i64>()
        .map_err(|_| AppError::Unauthorized("유효하지 않은 사용자 정보입니다.".into()))?;

    let result = RetrospectService::list_retrospects(state, member_id, retro_room_id).await?;

    Ok(Json(BaseResponse::success(result)))
}
