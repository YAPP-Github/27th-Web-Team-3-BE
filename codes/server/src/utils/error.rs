use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    extract::rejection::JsonRejection,
    Json,
};
use tracing::error;

use super::response::ApiErrorResponse;

/// 애플리케이션 전역 에러 타입
#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    NotFound(String),
    Unauthorized(String),
    Forbidden(String),
    InternalError(String),
    ValidationError(String),
    JsonParseFailed(String),
}

impl AppError {
    /// 에러 메시지 반환
    pub fn message(&self) -> &str {
        match self {
            AppError::BadRequest(msg) => msg,
            AppError::NotFound(msg) => msg,
            AppError::Unauthorized(msg) => msg,
            AppError::Forbidden(msg) => msg,
            AppError::InternalError(msg) => msg,
            AppError::ValidationError(msg) => msg,
            AppError::JsonParseFailed(msg) => msg,
        }
    }

    /// 에러 코드 반환
    pub fn error_code(&self) -> &str {
        match self {
            AppError::BadRequest(_) => "BAD_REQUEST",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Unauthorized(_) => "UNAUTHORIZED",
            AppError::Forbidden(_) => "FORBIDDEN",
            AppError::InternalError(_) => "INTERNAL_SERVER_ERROR",
            AppError::ValidationError(_) => "VALIDATION_ERROR",
            AppError::JsonParseFailed(_) => "JSON_PARSE_ERROR",
        }
    }

    /// HTTP 상태 코드 반환
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::JsonParseFailed(_) => StatusCode::BAD_REQUEST,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_code = self.error_code().to_string();
        let message = self.message().to_string();

        // 에러 로깅
        match &self {
            AppError::InternalError(_) => {
                error!("Internal Server Error: {}", message);
            }
            _ => {
                error!("Error [{}]: {}", error_code, message);
            }
        }

        let error_response = ApiErrorResponse::new(error_code, message, None);

        (status, Json(error_response)).into_response()
    }
}

/// JsonRejection을 AppError로 변환
impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        AppError::JsonParseFailed(rejection.to_string())
    }
}

/// 편의 함수들
impl AppError {
    pub fn bad_request(msg: impl Into<String>) -> Self {
        AppError::BadRequest(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        AppError::NotFound(msg.into())
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        AppError::Unauthorized(msg.into())
    }

    pub fn forbidden(msg: impl Into<String>) -> Self {
        AppError::Forbidden(msg.into())
    }

    pub fn internal_error(msg: impl Into<String>) -> Self {
        AppError::InternalError(msg.into())
    }

    pub fn validation_error(msg: impl Into<String>) -> Self {
        AppError::ValidationError(msg.into())
    }
}

