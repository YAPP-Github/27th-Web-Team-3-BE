use actix_web::{web, HttpResponse};
use validator::Validate;

use crate::error::AppError;
use crate::models::request::SignUpRequest;
use crate::models::response::{BaseResponse, SignUpResponse};
use crate::rate_limiter::RateLimiter;

use crate::domain::auth::service;

#[utoipa::path(
    post,
    path = "/api/auth/signup",
    request_body = SignUpRequest,
    responses(
        (status = 200, description = "Success", body = BaseResponse<SignUpResponse>),
        (status = 400, description = "Bad Request"),
        (status = 409, description = "Conflict - User already exists"),
        (status = 429, description = "Too Many Requests")
    ),
    tag = "Auth"
)]
pub async fn sign_up(
    req: web::Json<SignUpRequest>,
    rate_limiter: web::Data<RateLimiter>,
) -> Result<HttpResponse, AppError> {
    // Rate limiting 체크 (이메일을 user_id로 사용)
    rate_limiter.check_rate_limit(&req.email)?;

    // validate
    req.validate()
        .map_err(|e| AppError::BadRequest(format!("Validation error: {}", e)))?;

    // call service
    let user = service::create_user(&req).await?;

    let resp = SignUpResponse {
        user_id: user.id,
        email: user.email,
        username: user.username,
        created_at: user.created_at,
    };

    Ok(HttpResponse::Ok().json(BaseResponse::success(resp)))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/signup")
            .route(web::post().to(sign_up))
    );
}

