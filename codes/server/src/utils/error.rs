use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    extract::rejection::JsonRejection,
    Json,
};
use tracing::error;
use validator::ValidationErrors;

use super::response::ErrorResponse;

/// 애플리케이션 전역 에러 타입
/// API 명세에 정의된 에러 코드를 사용합니다.
#[derive(Debug)]
#[allow(dead_code)]
pub enum AppError {
    /// AI_001: 유효하지 않은 비밀 키 (401)
    InvalidSecretKey,

    /// AI_002: 유효하지 않은 말투 스타일 (400)
    InvalidToneStyle,

    /// AI_003: AI 서비스 연결 실패 (500)
    AiConnectionFailed(String),

    /// AI_005: AI 서비스 일시적 오류 (503)
    AiServiceUnavailable(String),

    /// AI_006: AI 서비스 일반 오류 (500)
    AiGeneralError(String),

    /// COMMON400: 잘못된 요청 (400)
    BadRequest(String),

    /// COMMON400: 유효성 검증 실패 (400)
    ValidationError(String),

    /// COMMON500: 서버 내부 에러 (500)
    InternalError(String),

    /// JSON 파싱 실패 (400)
    JsonParseFailed(String),
}

impl AppError {
    /// 에러 메시지 반환
    pub fn message(&self) -> String {
        match self {
            AppError::InvalidSecretKey => "유효하지 않은 비밀 키입니다.".to_string(),
            AppError::InvalidToneStyle => {
                "유효하지 않은 말투 스타일입니다. KIND 또는 POLITE만 가능합니다.".to_string()
            }
            AppError::AiConnectionFailed(msg) => {
                format!("AI 서비스 연결에 실패했습니다: {}", msg)
            }
            AppError::AiServiceUnavailable(_) => {
                "AI 서비스가 일시적으로 불안정합니다. 잠시 후 다시 시도해주세요.".to_string()
            }
            AppError::AiGeneralError(msg) => format!("AI 서비스 오류: {}", msg),
            AppError::BadRequest(msg) => format!("잘못된 요청입니다: {}", msg),
            AppError::ValidationError(msg) => format!("잘못된 요청입니다: {}", msg),
            AppError::InternalError(_) => "서버 에러, 관리자에게 문의 바랍니다.".to_string(),
            AppError::JsonParseFailed(msg) => format!("JSON 파싱 실패: {}", msg),
        }
    }

    /// 에러 코드 반환
    pub fn error_code(&self) -> &str {
        match self {
            AppError::InvalidSecretKey => "AI_001",
            AppError::InvalidToneStyle => "AI_002",
            AppError::AiConnectionFailed(_) => "AI_003",
            AppError::AiServiceUnavailable(_) => "AI_005",
            AppError::AiGeneralError(_) => "AI_006",
            AppError::BadRequest(_) => "COMMON400",
            AppError::ValidationError(_) => "COMMON400",
            AppError::InternalError(_) => "COMMON500",
            AppError::JsonParseFailed(_) => "COMMON400",
        }
    }

    /// HTTP 상태 코드 반환
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::InvalidSecretKey => StatusCode::UNAUTHORIZED,
            AppError::InvalidToneStyle => StatusCode::BAD_REQUEST,
            AppError::AiConnectionFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::AiServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::AiGeneralError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::JsonParseFailed(_) => StatusCode::BAD_REQUEST,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_code = self.error_code().to_string();
        let message = self.message();

        // 에러 로깅
        match &self {
            AppError::InternalError(msg) => {
                error!("Internal Server Error: {}", msg);
            }
            AppError::AiConnectionFailed(msg) | AppError::AiGeneralError(msg) => {
                error!("AI Error [{}]: {}", error_code, msg);
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

/// ValidationErrors를 AppError로 변환
impl From<ValidationErrors> for AppError {
    fn from(errors: ValidationErrors) -> Self {
        let messages: Vec<String> = errors
            .field_errors()
            .iter()
            .flat_map(|(field, errs)| {
                errs.iter().map(move |e| {
                    e.message
                        .as_ref()
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| format!("{} 필드가 유효하지 않습니다", field))
                })
            })
            .collect();

        AppError::ValidationError(messages.join(", "))
    }
}

/// 편의 함수들
#[allow(dead_code)]
impl AppError {
    pub fn bad_request(msg: impl Into<String>) -> Self {
        AppError::BadRequest(msg.into())
    }

    pub fn internal_error(msg: impl Into<String>) -> Self {
        AppError::InternalError(msg.into())
    }

    pub fn validation_error(msg: impl Into<String>) -> Self {
        AppError::ValidationError(msg.into())
    }
}

