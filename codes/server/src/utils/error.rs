use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use tracing::error;

use super::response::ErrorResponse;

/// 애플리케이션 전역 에러 타입
#[derive(Debug)]
pub enum AppError {
    #[allow(dead_code)]
    BadRequest(String),
    #[allow(dead_code)]
    NotFound(String),
    Unauthorized(String),
    #[allow(dead_code)]
    Forbidden(String),
    InternalError(String),
    ValidationError(String),
    JsonParseFailed(String),
}

impl AppError {
    /// 에러 메시지 반환
    pub fn message(&self) -> String {
        match self {
            AppError::BadRequest(msg) => msg.clone(),
            AppError::NotFound(msg) => msg.clone(),
            AppError::Unauthorized(msg) => msg.clone(),
            AppError::Forbidden(msg) => msg.clone(),
            AppError::InternalError(msg) => msg.clone(),
            AppError::ValidationError(msg) => msg.clone(),
            AppError::JsonParseFailed(msg) => format!("잘못된 요청 형식입니다: {}", msg),
        }
    }

    /// 에러 코드 반환
    pub fn error_code(&self) -> String {
        match self {
            AppError::BadRequest(_) => "COMMON400",
            AppError::NotFound(_) => "COMMON404",
            AppError::Unauthorized(_) => "AI_001",
            AppError::Forbidden(_) => "COMMON403",
            AppError::InternalError(_) => "COMMON500",
            AppError::ValidationError(_) => "COMMON400",
            AppError::JsonParseFailed(_) => "COMMON400",
        }
        .to_string()
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
        let error_code = self.error_code();
        let message = self.message();

        // 에러 로깅
        match &self {
            AppError::InternalError(_) => {
                error!("Internal Server Error: {}", message);
            }
            _ => {
                error!("Error [{}]: {}", error_code, message);
            }
        }

        let error_response = ErrorResponse::new(error_code, message);

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
    #[allow(dead_code)]
    pub fn bad_request(msg: impl Into<String>) -> Self {
        AppError::BadRequest(msg.into())
    }

    #[allow(dead_code)]
    pub fn not_found(msg: impl Into<String>) -> Self {
        AppError::NotFound(msg.into())
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        AppError::Unauthorized(msg.into())
    }

    #[allow(dead_code)]
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
