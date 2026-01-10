use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

use crate::response::BaseResponse;

#[derive(Error, Debug)]
pub enum AppError {
    // AI_001: 401 Unauthorized
    #[error("유효하지 않은 비밀 키입니다.")]
    InvalidSecretKey,

    // AI_002: 400 Bad Request
    #[error("유효하지 않은 말투 스타일입니다. KIND 또는 POLITE만 가능합니다.")]
    InvalidToneStyle,

    // COMMON400: 400 Bad Request
    #[error("잘못된 요청입니다.")]
    BadRequest(String),

    // Validation Error
    #[error("잘못된 요청입니다: {0}")]
    ValidationError(String),

    // COMMON500: 500 Internal Server Error
    #[error("서버 에러, 관리자에게 문의 바랍니다.")]
    Internal(String),

    // AI_003: 500 - OpenAI 인증 실패 (API 키 문제)
    #[error("AI 서비스 연결에 실패했습니다. 관리자에게 문의하세요.")]
    OpenAiAuthError,

    // AI_004: 503 - OpenAI 요청 한도 초과
    #[error("AI 서비스가 일시적으로 바쁩니다. 잠시 후 다시 시도해주세요.")]
    OpenAiRateLimitError,

    // AI_005: 503 - OpenAI 일시적 오류 (타임아웃, 연결 실패 등)
    #[error("AI 서비스가 일시적으로 불안정합니다. 잠시 후 다시 시도해주세요.")]
    OpenAiTemporaryError,

    // AI_006: 500 - OpenAI 일반 오류
    #[error("AI 서비스 오류가 발생했습니다.")]
    OpenAiError(String),

    // RATE_001: 429 Too Many Requests
    #[error("요청이 너무 많습니다. 잠시 후 다시 시도해주세요.")]
    TooManyRequests,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            AppError::InvalidSecretKey => (StatusCode::UNAUTHORIZED, "AI_001"),
            AppError::InvalidToneStyle => (StatusCode::BAD_REQUEST, "AI_002"),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "COMMON400"),
            AppError::ValidationError(_) => (StatusCode::BAD_REQUEST, "COMMON400"),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "COMMON500"),
            AppError::OpenAiAuthError => (StatusCode::INTERNAL_SERVER_ERROR, "AI_003"),
            AppError::OpenAiRateLimitError => (StatusCode::SERVICE_UNAVAILABLE, "AI_004"),
            AppError::OpenAiTemporaryError => (StatusCode::SERVICE_UNAVAILABLE, "AI_005"),
            AppError::OpenAiError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "AI_006"),
            AppError::TooManyRequests => (StatusCode::TOO_MANY_REQUESTS, "RATE_001"),
        };

        // 에러 로깅 - 심각도에 따라 레벨 분류
        match &self {
            AppError::InvalidSecretKey => {
                tracing::warn!(code = %code, "Authentication failed: invalid secret key");
            }
            AppError::InvalidToneStyle => {
                tracing::warn!(code = %code, "Invalid tone style requested");
            }
            AppError::BadRequest(msg) => {
                tracing::warn!(code = %code, message = %msg, "Bad request");
            }
            AppError::ValidationError(msg) => {
                tracing::warn!(code = %code, message = %msg, "Validation error");
            }
            AppError::Internal(msg) => {
                tracing::error!(code = %code, message = %msg, "Internal server error");
            }
            AppError::OpenAiAuthError => {
                tracing::error!(code = %code, "OpenAI authentication failed - check API key");
            }
            AppError::OpenAiRateLimitError => {
                tracing::warn!(code = %code, "OpenAI rate limit exceeded");
            }
            AppError::OpenAiTemporaryError => {
                tracing::warn!(code = %code, "OpenAI temporary error (timeout/connection)");
            }
            AppError::OpenAiError(msg) => {
                tracing::error!(code = %code, message = %msg, "OpenAI API error");
            }
            AppError::TooManyRequests => {
                tracing::warn!(code = %code, "Rate limit exceeded");
            }
        }

        let body = BaseResponse::<()>::error(code, &self.to_string());

        (status, Json(body)).into_response()
    }
}

// validator 에러 변환
impl From<validator::ValidationErrors> for AppError {
    fn from(err: validator::ValidationErrors) -> Self {
        AppError::ValidationError(err.to_string())
    }
}

// JSON 파싱 에러 변환
impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        let message = rejection.body_text();

        // ToneStyle 파싱 실패 감지
        if message.contains("toneStyle") || message.contains("ToneStyle") {
            return AppError::InvalidToneStyle;
        }

        AppError::BadRequest(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Error Message Tests =====

    #[test]
    fn test_error_messages() {
        assert_eq!(
            AppError::InvalidSecretKey.to_string(),
            "유효하지 않은 비밀 키입니다."
        );
        assert_eq!(
            AppError::InvalidToneStyle.to_string(),
            "유효하지 않은 말투 스타일입니다. KIND 또는 POLITE만 가능합니다."
        );
    }

    #[test]
    fn test_bad_request_error() {
        let err = AppError::BadRequest("test".to_string());
        assert_eq!(err.to_string(), "잘못된 요청입니다.");
    }

    #[test]
    fn test_validation_error() {
        let err = AppError::ValidationError("field is required".to_string());
        assert_eq!(err.to_string(), "잘못된 요청입니다: field is required");
    }

    #[test]
    fn test_internal_error() {
        let err = AppError::Internal("database error".to_string());
        assert_eq!(err.to_string(), "서버 에러, 관리자에게 문의 바랍니다.");
    }

    #[test]
    fn test_openai_error() {
        let err = AppError::OpenAiError("some error".to_string());
        assert_eq!(err.to_string(), "AI 서비스 오류가 발생했습니다.");
    }

    // ===== Status Code Tests =====

    #[test]
    fn test_invalid_secret_key_status() {
        let err = AppError::InvalidSecretKey;
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_invalid_tone_style_status() {
        let err = AppError::InvalidToneStyle;
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_bad_request_status() {
        let err = AppError::BadRequest("test".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_validation_error_status() {
        let err = AppError::ValidationError("test".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_internal_error_status() {
        let err = AppError::Internal("test".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_openai_error_status() {
        let err = AppError::OpenAiError("test".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_too_many_requests_error() {
        let err = AppError::TooManyRequests;
        assert_eq!(
            err.to_string(),
            "요청이 너무 많습니다. 잠시 후 다시 시도해주세요."
        );
    }

    #[test]
    fn test_too_many_requests_status() {
        let err = AppError::TooManyRequests;
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    // ===== OpenAI 세분화 에러 테스트 =====

    #[test]
    fn test_openai_auth_error() {
        let err = AppError::OpenAiAuthError;
        assert_eq!(
            err.to_string(),
            "AI 서비스 연결에 실패했습니다. 관리자에게 문의하세요."
        );
    }

    #[test]
    fn test_openai_auth_error_status() {
        let err = AppError::OpenAiAuthError;
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_openai_rate_limit_error() {
        let err = AppError::OpenAiRateLimitError;
        assert_eq!(
            err.to_string(),
            "AI 서비스가 일시적으로 바쁩니다. 잠시 후 다시 시도해주세요."
        );
    }

    #[test]
    fn test_openai_rate_limit_error_status() {
        let err = AppError::OpenAiRateLimitError;
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_openai_temporary_error() {
        let err = AppError::OpenAiTemporaryError;
        assert_eq!(
            err.to_string(),
            "AI 서비스가 일시적으로 불안정합니다. 잠시 후 다시 시도해주세요."
        );
    }

    #[test]
    fn test_openai_temporary_error_status() {
        let err = AppError::OpenAiTemporaryError;
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_openai_general_error() {
        let err = AppError::OpenAiError("Some error".to_string());
        assert_eq!(err.to_string(), "AI 서비스 오류가 발생했습니다.");
    }
}
