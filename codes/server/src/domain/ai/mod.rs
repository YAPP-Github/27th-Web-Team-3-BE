pub mod client;
pub mod dto;
pub mod handler;
mod prompt;
mod retry;
pub mod service;

use axum::{routing::post, Router};

use crate::{global::rate_limit::create_ai_rate_limiter, AppState};

/// AI 라우터 생성 (Rate Limiting 포함)
///
/// 프로덕션 환경에서 사용됩니다.
/// - 초당 10 요청, 버스트 50 허용
/// - IP 기반 제한 (X-Forwarded-For 헤더 지원)
pub fn router() -> Router<AppState> {
    router_without_rate_limit().layer(create_ai_rate_limiter())
}

/// AI 라우터 생성 (Rate Limiting 미포함)
///
/// 테스트 환경에서 사용됩니다.
pub fn router_without_rate_limit() -> Router<AppState> {
    Router::new()
        .route("/api/ai/retrospective/guide", post(handler::provide_guide))
        .route(
            "/api/ai/retrospective/refine",
            post(handler::refine_retrospective),
        )
}
