use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
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
    /// COMMON400: 잘못된 요청 (400)
    BadRequest(String),

    /// COMMON400: 유효성 검증 실패 (400)
    ValidationError(String),

    /// COMMON500: 서버 내부 에러 (500)
    InternalError(String),

    /// JSON 파싱 실패 (400)
    JsonParseFailed(String),

    /// AUTH4001: 인증 실패 (401)
    Unauthorized(String),

    /// COMMON403: 권한 없음 (403)
    Forbidden(String),

    /// RETRO4041: 존재하지 않는 회고 (404)
    RetrospectNotFound(String),

    /// RETRO4002: 답변 누락 (400)
    RetroAnswersMissing(String),

    /// RETRO4003: 답변 길이 초과 (400)
    RetroAnswerTooLong(String),

    /// RETRO4007: 공백만 입력 (400)
    RetroAnswerWhitespaceOnly(String),

    /// RETRO4033: 이미 제출 완료 (403)
    RetroAlreadySubmitted(String),
}

impl AppError {
    /// 에러 메시지 반환
    pub fn message(&self) -> String {
        match self {
            AppError::BadRequest(msg) => format!("잘못된 요청입니다: {}", msg),
            AppError::ValidationError(msg) => format!("잘못된 요청입니다: {}", msg),
            AppError::InternalError(_) => "서버 에러, 관리자에게 문의 바랍니다.".to_string(),
            AppError::JsonParseFailed(msg) => format!("JSON 파싱 실패: {}", msg),
            AppError::Unauthorized(msg) => format!("인증 실패: {}", msg),
            AppError::Forbidden(msg) => format!("권한 없음: {}", msg),
            AppError::RetrospectNotFound(msg) => msg.clone(),
            AppError::RetroAnswersMissing(msg) => msg.clone(),
            AppError::RetroAnswerTooLong(msg) => msg.clone(),
            AppError::RetroAnswerWhitespaceOnly(msg) => msg.clone(),
            AppError::RetroAlreadySubmitted(msg) => msg.clone(),
        }
    }

    /// 에러 코드 반환
    pub fn error_code(&self) -> &str {
        match self {
            AppError::BadRequest(_) => "COMMON400",
            AppError::ValidationError(_) => "COMMON400",
            AppError::InternalError(_) => "COMMON500",
            AppError::JsonParseFailed(_) => "COMMON400",
            AppError::Unauthorized(_) => "AUTH4001",
            AppError::Forbidden(_) => "COMMON403",
            AppError::RetrospectNotFound(_) => "RETRO4041",
            AppError::RetroAnswersMissing(_) => "RETRO4002",
            AppError::RetroAnswerTooLong(_) => "RETRO4003",
            AppError::RetroAnswerWhitespaceOnly(_) => "RETRO4007",
            AppError::RetroAlreadySubmitted(_) => "RETRO4033",
        }
    }

    /// HTTP 상태 코드 반환
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::JsonParseFailed(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::RetrospectNotFound(_) => StatusCode::NOT_FOUND,
            AppError::RetroAnswersMissing(_) => StatusCode::BAD_REQUEST,
            AppError::RetroAnswerTooLong(_) => StatusCode::BAD_REQUEST,
            AppError::RetroAnswerWhitespaceOnly(_) => StatusCode::BAD_REQUEST,
            AppError::RetroAlreadySubmitted(_) => StatusCode::FORBIDDEN,
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
