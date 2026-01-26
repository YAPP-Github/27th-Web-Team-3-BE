use axum::{extract::State, Json};
use validator::Validate;

use crate::state::AppState;
use crate::utils::error::AppError;
use crate::utils::BaseResponse;
use crate::utils::auth::AuthUser;
use super::dto::{TeamCreateRequest, TeamCreateResponse, SuccessTeamCreateResponse};
use super::service::RetrospectService;

/// 팀 생성 API
///
/// 새로운 팀(회고방)을 생성하고 생성자를 관리자로 설정합니다.
#[utoipa::path(
    post,
    path = "/api/v1/teams",
    request_body = TeamCreateRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "팀 생성 성공", body = SuccessTeamCreateResponse),
        (status = 400, description = "잘못된 요청", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 409, description = "팀 이름 중복", body = ErrorResponse)
    ),
    tag = "Team"
)]
pub async fn create_team(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<TeamCreateRequest>,
) -> Result<Json<BaseResponse<TeamCreateResponse>>, AppError> {
    req.validate()?;
    
    let member_id = user.0.sub.parse::<i64>().map_err(|_| {
        AppError::Unauthorized("유효하지 않은 사용자 정보입니다.".into())
    })?;
    
    let result = RetrospectService::create_team(state, member_id, req).await?;
    
    Ok(Json(BaseResponse::success(result)))
}
