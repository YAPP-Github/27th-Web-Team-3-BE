#[allow(unused_imports)]
use super::{
    dto::{RetrospectiveGuideRequest, RetrospectiveGuideResponse, RetrospectiveGuideResult},
    service::AiService,
};
use crate::utils::{error::AppError, response::ErrorResponse};
use axum::{response::IntoResponse, Json};
use validator::Validate;

/// 회고 작성 가이드 API 핸들러
#[utoipa::path(
    post,
    path = "/api/ai/retrospective/guide",
    request_body = RetrospectiveGuideRequest,
    responses(
        (status = 200, body = RetrospectiveGuideResponse),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    )
)]
pub async fn guide_handler(
    Json(req): Json<RetrospectiveGuideRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 1. 입력값 검증
    req.validate()
        .map_err(|e| AppError::validation_error(e.to_string()))?;

    // 2. 서비스 로직 호출
    let result = AiService::generate_guide(req).await?;

    // 3. 성공 응답 반환
    Ok(Json(RetrospectiveGuideResponse::success(result)))
}
