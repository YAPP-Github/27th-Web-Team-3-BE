use axum::{extract::State, Json};
use validator::Validate;

use super::dto::{
    JoinRetroRoomRequest, JoinRetroRoomResponse, RetroRoomCreateRequest, RetroRoomCreateResponse,
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
