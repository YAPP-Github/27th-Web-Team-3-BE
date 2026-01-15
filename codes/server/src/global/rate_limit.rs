//! Rate Limiting 모듈
//!
//! IP 기반 요청 제한을 통해 악의적인 대량 요청으로부터 서비스를 보호합니다.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorError,
    GovernorLayer,
};

use crate::response::BaseResponse;

/// Rate Limiter 설정 상수
mod config {
    /// 초당 허용 요청 수
    pub const REQUESTS_PER_SECOND: u64 = 10;
    /// 버스트 허용량 (순간 최대 요청 수)
    pub const BURST_SIZE: u32 = 50;
}

/// AI 엔드포인트용 Rate Limiter 레이어 생성
///
/// # 설정
/// - 초당 10 요청 허용 (replenish rate)
/// - 버스트 사이즈 50 (순간 허용 요청 수)
/// - IP 기반 제한 (X-Forwarded-For 헤더 지원)
pub(crate) fn create_ai_rate_limiter(
) -> GovernorLayer<SmartIpKeyExtractor, governor::middleware::NoOpMiddleware> {
    let config = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(config::REQUESTS_PER_SECOND)
            .burst_size(config::BURST_SIZE)
            .key_extractor(SmartIpKeyExtractor)
            .error_handler(|err| RateLimitResponse::from(err).into_response())
            .finish()
            .expect("Failed to build rate limiter config"),
    );

    GovernorLayer { config }
}

/// Rate Limit 초과 시 응답
#[derive(Debug)]
struct RateLimitResponse {
    retry_after: Option<u64>,
}

impl From<GovernorError> for RateLimitResponse {
    fn from(err: GovernorError) -> Self {
        match err {
            GovernorError::TooManyRequests { wait_time, .. } => Self {
                retry_after: Some(wait_time),
            },
            _ => Self { retry_after: None },
        }
    }
}

impl IntoResponse for RateLimitResponse {
    fn into_response(self) -> Response {
        tracing::warn!(
            retry_after_secs = ?self.retry_after,
            "Rate limit exceeded"
        );

        let body = BaseResponse::<()>::error(
            "RATE_LIMIT",
            "요청이 너무 많습니다. 잠시 후 다시 시도해주세요.",
        );

        let mut response = (StatusCode::TOO_MANY_REQUESTS, Json(body)).into_response();

        // Retry-After 헤더 추가
        if let Some(retry_after) = self.retry_after {
            if let Ok(value) = retry_after.to_string().parse() {
                response.headers_mut().insert("Retry-After", value);
            }
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limiter_should_be_created_successfully() {
        let layer = create_ai_rate_limiter();
        // 레이어가 생성되면 성공
        assert!(std::mem::size_of_val(&layer) > 0);
    }

    #[test]
    fn rate_limit_response_should_have_correct_status() {
        let response = RateLimitResponse {
            retry_after: Some(60),
        };
        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn rate_limit_response_should_include_retry_after_header() {
        let response = RateLimitResponse {
            retry_after: Some(30),
        };
        let response = response.into_response();

        let retry_after = response.headers().get("Retry-After");
        assert!(retry_after.is_some());
        assert_eq!(retry_after.unwrap().to_str().unwrap(), "30");
    }

    #[test]
    fn rate_limit_response_without_retry_after_should_work() {
        let response = RateLimitResponse { retry_after: None };
        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(response.headers().get("Retry-After").is_none());
    }
}
