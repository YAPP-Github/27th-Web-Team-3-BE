use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::response::{ApiResponse, ResponseStatus};

/// 애플리케이션 에러 타입
#[derive(Debug, Error)]
pub enum AppError {
    #[error("인증 실패: {0}")]
    AuthError(String),

    #[error("권한 없음: {0}")]
    Unauthorized(String),

    #[error("리소스를 찾을 수 없음: {0}")]
    NotFound(String),

    #[error("잘못된 요청: {0}")]
    BadRequest(String),

    #[error("유효성 검사 실패: {0}")]
    ValidationError(String),

    #[error("중복된 리소스: {0}")]
    Conflict(String),

    #[error("내부 서버 오류: {0}")]
    InternalError(String),

    #[error("데이터베이스 오류: {0}")]
    DatabaseError(String),

    #[error("JWT 오류: {0}")]
    JwtError(String),
}

/// 에러 응답 구조체
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub status: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status_code, error_code, message) = match self {
            AppError::AuthError(msg) => (StatusCode::UNAUTHORIZED, "AUTH_FAILED", msg),
            AppError::Unauthorized(msg) => (StatusCode::FORBIDDEN, "UNAUTHORIZED", msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg),
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_FAILED", msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg),
            AppError::InternalError(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg)
            }
            AppError::DatabaseError(msg) => {
                tracing::error!("Database error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", "데이터베이스 오류가 발생했습니다".to_string())
            }
            AppError::JwtError(msg) => (StatusCode::UNAUTHORIZED, "JWT_ERROR", msg),
        };

        let error_response = ErrorResponse {
            status: error_code.to_string(),
            message,
            details: None,
        };

        (status_code, Json(error_response)).into_response()
    }
}

// anyhow::Error를 AppError로 변환
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::InternalError(err.to_string())
    }
}

// JWT 에러 변환
impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AppError::JwtError(format!("JWT 처리 실패: {}", err))
    }
}

// bcrypt 에러 변환
impl From<bcrypt::BcryptError> for AppError {
    fn from(err: bcrypt::BcryptError) -> Self {
        AppError::InternalError(format!("암호화 처리 실패: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_error() {
        let error = AppError::AuthError("Invalid credentials".to_string());
        assert_eq!(error.to_string(), "인증 실패: Invalid credentials");
    }

    #[test]
    fn test_bad_request_error() {
        let error = AppError::BadRequest("Missing field".to_string());
        assert_eq!(error.to_string(), "잘못된 요청: Missing field");
    }

    #[test]
    fn test_validation_error() {
        let error = AppError::ValidationError("Invalid email format".to_string());
        assert_eq!(error.to_string(), "유효성 검사 실패: Invalid email format");
    }

    #[test]
    fn test_conflict_error() {
        let error = AppError::Conflict("User already exists".to_string());
        assert_eq!(error.to_string(), "중복된 리소스: User already exists");
    }
}

