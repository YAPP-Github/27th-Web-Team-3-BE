use actix_web::{web, HttpResponse};
use crate::error::AppError;
use crate::models::response::BaseResponse;
use crate::rate_limiter::RateLimiter;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct TestRequest {
    #[schema(example = "user123")]
    pub user_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TestResponse {
    pub message: String,
    pub remaining: u32,
}

#[utoipa::path(
    post,
    path = "/api/test/rate-limit",
    request_body = TestRequest,
    responses(
        (status = 200, description = "Success", body = BaseResponse<TestResponse>),
        (status = 429, description = "Too Many Requests")
    ),
    tag = "Test"
)]
pub async fn test_rate_limit(
    req: web::Json<TestRequest>,
    rate_limiter: web::Data<RateLimiter>,
) -> Result<HttpResponse, AppError> {
    // Rate limiting 체크
    rate_limiter.check_rate_limit(&req.user_id)?;

    let remaining = rate_limiter.get_remaining_requests(&req.user_id);

    let resp = TestResponse {
        message: format!("요청이 성공했습니다. user_id: {}", req.user_id),
        remaining,
    };

    Ok(HttpResponse::Ok().json(BaseResponse::success(resp)))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/rate-limit")
            .route(web::post().to(test_rate_limit))
    );
}

