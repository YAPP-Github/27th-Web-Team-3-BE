use actix_web::{web, HttpResponse};
use validator::Validate;

use crate::config::AppConfig;
use crate::error::AppError;
use crate::models::request::{GuideRequest, RefineRequest};
use crate::models::response::{BaseResponse, GuideResponse, RefineResponse};
use crate::domain::ai::{service, validator as ai_validator};

/// Provides a writing guide for retrospective content
#[utoipa::path(
    post,
    path = "/api/ai/retrospective/guide",
    request_body = GuideRequest,
    responses(
        (status = 200, description = "성공", body = BaseResponse<GuideResponse>),
        (status = 400, description = "잘못된 요청", body = crate::error::ErrorResponse),
        (status = 401, description = "유효하지 않은 비밀 키", body = crate::error::ErrorResponse),
        (status = 500, description = "서버 에러", body = crate::error::ErrorResponse)
    ),
    tag = "AI"
)]
pub async fn provide_guide(
    req: web::Json<GuideRequest>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, AppError> {
    // Validate request
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    // Validate secret key
    ai_validator::validate_secret_key(&req.secret_key, &config)?;

    // Call AI service
    let guide_message = service::provide_writing_guide(&req.current_content, &config).await?;

    let response = BaseResponse::success(GuideResponse {
        current_content: req.current_content.clone(),
        guide_message,
    });

    Ok(HttpResponse::Ok().json(response))
}

/// Refines retrospective content with selected tone style
#[utoipa::path(
    post,
    path = "/api/ai/retrospective/refine",
    request_body = RefineRequest,
    responses(
        (status = 200, description = "성공", body = BaseResponse<RefineResponse>),
        (status = 400, description = "잘못된 요청", body = crate::error::ErrorResponse),
        (status = 401, description = "유효하지 않은 비밀 키", body = crate::error::ErrorResponse),
        (status = 500, description = "서버 에러", body = crate::error::ErrorResponse)
    ),
    tag = "AI"
)]
pub async fn refine_retrospective(
    req: web::Json<RefineRequest>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, AppError> {
    // Validate request
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    // Validate secret key
    ai_validator::validate_secret_key(&req.secret_key, &config)?;

    // Call AI service
    let refined_content = service::refine_content(&req.content, &req.tone_style, &config).await?;

    let response = BaseResponse::success(RefineResponse {
        original_content: req.content.clone(),
        refined_content,
        tone_style: req.tone_style.to_korean().to_string(),
    });

    Ok(HttpResponse::Ok().json(response))
}

/// Configure AI routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/retrospective/guide")
            .route(web::post().to(provide_guide))
    )
    .service(
        web::resource("/retrospective/refine")
            .route(web::post().to(refine_retrospective))
    );
}
