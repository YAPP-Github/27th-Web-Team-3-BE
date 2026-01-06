use actix_web::{web, HttpResponse};
use validator::Validate;

use crate::config::AppConfig;
use crate::error::AppError;
use crate::models::request::{GuideRequest, RefineRequest};
use crate::models::response::{BaseResponse, GuideResponse, RefineResponse};

use crate::domain::ai::{service, validator::SecretKeyValidator};

#[utoipa::path(
    post,
    path = "/api/ai/guide",
    request_body = GuideRequest,
    responses(
        (status = 200, description = "Success", body = BaseResponse<GuideResponse>),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "AI"
)]
pub async fn provide_guide(
    config: web::Data<AppConfig>,
    req: web::Json<GuideRequest>,
) -> Result<HttpResponse, AppError> {
    // validate
    req.validate()
        .map_err(|e| AppError::BadRequest(format!("Validation error: {}", e)))?;

    // validate secret key
    SecretKeyValidator::validate(&config, &req.secret_key)?;

    // call service - now returns Result
    let guide = service::generate_guide(&req.current_content).await?;

    let resp = GuideResponse {
        current_content: req.current_content.clone(),
        guide_message: guide,
    };

    Ok(HttpResponse::Ok().json(BaseResponse::success(resp)))
}

#[utoipa::path(
    post,
    path = "/api/ai/refine",
    request_body = RefineRequest,
    responses(
        (status = 200, description = "Success", body = BaseResponse<RefineResponse>),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "AI"
)]
pub async fn refine_retrospective(
    config: web::Data<AppConfig>,
    req: web::Json<RefineRequest>,
) -> Result<HttpResponse, AppError> {
    // validate
    req.validate()
        .map_err(|e| AppError::BadRequest(format!("Validation error: {}", e)))?;

    // validate secret key
    SecretKeyValidator::validate(&config, &req.secret_key)?;

    // call service - now returns Result
    let (refined, tone) = service::refine_content(&req.content, &req.tone_style).await?;

    let resp = RefineResponse {
        original_content: req.content.clone(),
        refined_content: refined,
        tone_style: tone,
    };

    Ok(HttpResponse::Ok().json(BaseResponse::success(resp)))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/guide")
            .route(web::post().to(provide_guide))
    )
    .service(
        web::resource("/refine")
            .route(web::post().to(refine_retrospective))
    );
}
