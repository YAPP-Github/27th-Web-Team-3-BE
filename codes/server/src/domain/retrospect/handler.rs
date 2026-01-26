use axum::{extract::State, Json};
use validator::Validate;

use crate::state::AppState;
use crate::utils::auth::AuthUser;
use crate::utils::error::AppError;
use crate::utils::BaseResponse;

use super::dto::{CreateRetrospectRequest, CreateRetrospectResponse};
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
