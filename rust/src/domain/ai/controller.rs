use actix_web::{web, HttpResponse};
use ::validator::Validate;

use crate::config::AppConfig;
use crate::error::AppError;
use crate::models::request::{GuideRequest, RefineRequest, RetrospectiveGuideRequest};
use crate::models::response::{BaseResponse, GuideResponse, RefineResponse, RetrospectiveGuideResponse};
use crate::rate_limiter::RateLimiter;

use crate::domain::ai::{service, validator};

#[utoipa::path(
    post,
    path = "/api/ai/guide",
    request_body = GuideRequest,
    responses(
        (status = 200, description = "Success", body = BaseResponse<GuideResponse>),
        (status = 400, description = "Bad Request - COMMON400"),
        (status = 401, description = "Unauthorized - AI_001"),
        (status = 429, description = "Too Many Requests - COMMON429"),
        (status = 500, description = "Internal Server Error - COMMON500")
    ),
    tag = "AI"
)]
pub async fn provide_guide(
    req: web::Json<GuideRequest>,
    config: web::Data<AppConfig>,
    rate_limiter: web::Data<RateLimiter>,
) -> Result<HttpResponse, AppError> {
    // Rate limiting 체크
    rate_limiter.check_rate_limit(&req.secret_key)?;

    // Validate
    req.validate()
        .map_err(|e| AppError::BadRequest(format!("Validation error: {}", e)))?;

    // Validate secret key
    validator::validate_secret_key(&req.secret_key, &config)?;

    // Call service
    let guide_message = service::generate_retrospective_guide(&req.current_content).await?;

    let resp = GuideResponse {
        current_content: req.current_content.clone(),
        guide_message,
    };

    Ok(HttpResponse::Ok().json(BaseResponse::success(resp)))
}

#[utoipa::path(
    post,
    path = "/api/ai/retrospective/guide",
    request_body = RetrospectiveGuideRequest,
    responses(
        (status = 200, description = "Success", body = BaseResponse<RetrospectiveGuideResponse>),
        (status = 400, description = "Bad Request - COMMON400"),
        (status = 401, description = "Unauthorized - AI_001"),
        (status = 429, description = "Too Many Requests - COMMON429"),
        (status = 500, description = "Internal Server Error - COMMON500")
    ),
    tag = "AI"
)]
pub async fn provide_retrospective_guide(
    req: web::Json<RetrospectiveGuideRequest>,
    config: web::Data<AppConfig>,
    rate_limiter: web::Data<RateLimiter>,
) -> Result<HttpResponse, AppError> {
    // Rate limiting 체크
    rate_limiter.check_rate_limit(&req.secret_key)?;

    // Validate
    req.validate()
        .map_err(|e| AppError::BadRequest(format!("Validation error: {}", e)))?;

    // Validate secret key
    validator::validate_secret_key(&req.secret_key, &config)?;

    // Call service
    let guide_message = service::generate_retrospective_guide(&req.content).await?;

    let resp = RetrospectiveGuideResponse {
        guide_message,
    };

    Ok(HttpResponse::Ok().json(BaseResponse::success(resp)))
}

#[utoipa::path(
    post,
    path = "/api/ai/refine",
    request_body = RefineRequest,
    responses(
        (status = 200, description = "Success", body = BaseResponse<RefineResponse>),
        (status = 400, description = "Bad Request - COMMON400 or AI_002"),
        (status = 401, description = "Unauthorized - AI_001"),
        (status = 429, description = "Too Many Requests - COMMON429"),
        (status = 500, description = "Internal Server Error - COMMON500")
    ),
    tag = "AI"
)]
pub async fn refine_retrospective(
    req: web::Json<RefineRequest>,
    config: web::Data<AppConfig>,
    rate_limiter: web::Data<RateLimiter>,
) -> Result<HttpResponse, AppError> {
    // Rate limiting 체크
    rate_limiter.check_rate_limit(&req.secret_key)?;

    // Validate
    req.validate()
        .map_err(|e| AppError::BadRequest(format!("Validation error: {}", e)))?;

    // Validate secret key
    validator::validate_secret_key(&req.secret_key, &config)?;

    let tone_style = req.tone_style.to_korean();

    // Call service
    let refined_content = service::refine_retrospective(&req.content, tone_style).await?;

    let resp = RefineResponse {
        original_content: req.content.clone(),
        refined_content,
        tone_style: tone_style.to_string(),
    };

    Ok(HttpResponse::Ok().json(BaseResponse::success(resp)))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/guide")
            .route(web::post().to(provide_guide))
    )
    .service(
        web::resource("/retrospective/guide")
            .route(web::post().to(provide_retrospective_guide))
    )
    .service(
        web::resource("/refine")
            .route(web::post().to(refine_retrospective))
    );
}

