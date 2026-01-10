use axum::{
    extract::{rejection::JsonRejection, State},
    Json,
};
use validator::Validate;

use crate::error::AppError;
use crate::response::{BaseResponse, ErrorResponse};
use crate::AppState;

use super::dto::{GuideRequest, GuideResponse, RefineRequest, RefineResponse};

/// 회고 작성 가이드 제공
///
/// 현재 작성 중인 회고 내용을 분석하여 AI가 작성 가이드 메시지를 제공합니다.
#[utoipa::path(
    post,
    path = "/api/ai/retrospective/guide",
    tag = "AI",
    request_body = GuideRequest,
    responses(
        (status = 200, description = "가이드 생성 성공", body = BaseResponse<GuideResponse>),
        (status = 400, description = "잘못된 요청", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 500, description = "서버 에러", body = ErrorResponse)
    )
)]
pub async fn provide_guide(
    State(state): State<AppState>,
    request: Result<Json<GuideRequest>, JsonRejection>,
) -> Result<Json<BaseResponse<GuideResponse>>, AppError> {
    // JSON 파싱 에러 처리
    let Json(request) = request.map_err(AppError::from)?;

    tracing::info!(
        content_length = request.current_content.len(),
        "Guide request received"
    );

    // 입력 검증
    request.validate()?;
    tracing::debug!("Request validation passed");

    // 서비스 호출
    let response = state
        .ai_service
        .provide_guide(&request.current_content, &request.secret_key)
        .await?;

    tracing::info!(
        guide_length = response.guide_message.len(),
        "Guide generated successfully"
    );

    Ok(Json(BaseResponse::success(response)))
}

/// 회고 말투 정제
///
/// 작성된 회고 내용을 선택한 말투 스타일(상냥체/정중체)로 정제합니다.
#[utoipa::path(
    post,
    path = "/api/ai/retrospective/refine",
    tag = "AI",
    request_body = RefineRequest,
    responses(
        (status = 200, description = "정제 성공", body = BaseResponse<RefineResponse>),
        (status = 400, description = "잘못된 요청", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 500, description = "서버 에러", body = ErrorResponse)
    )
)]
pub async fn refine_retrospective(
    State(state): State<AppState>,
    request: Result<Json<RefineRequest>, JsonRejection>,
) -> Result<Json<BaseResponse<RefineResponse>>, AppError> {
    // JSON 파싱 에러 처리
    let Json(request) = request.map_err(AppError::from)?;

    tracing::info!(
        content_length = request.content.len(),
        tone_style = ?request.tone_style,
        "Refine request received"
    );

    // 입력 검증
    request.validate()?;
    tracing::debug!("Request validation passed");

    // 서비스 호출
    let response = state
        .ai_service
        .refine_retrospective(&request.content, request.tone_style, &request.secret_key)
        .await?;

    tracing::info!(
        refined_length = response.refined_content.len(),
        "Content refined successfully"
    );

    Ok(Json(BaseResponse::success(response)))
}
