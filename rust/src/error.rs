use actix_web::{error::ResponseError, HttpResponse};
use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    #[serde(rename = "isSuccess")]
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: Option<serde_json::Value>,
}

#[derive(Debug)]
pub enum AppError {
    InvalidSecretKey,
    InvalidToneStyle(String),
    BadRequest(String),
    Conflict(String),
    RateLimitExceeded(String),
    InternalServerError(String),
    InternalError(String),
    ExternalApiError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::InvalidSecretKey => write!(f, "유효하지 않은 비밀 키입니다."),
            AppError::InvalidToneStyle(style) => {
                write!(f, "유효하지 않은 말투 스타일입니다. KIND 또는 POLITE만 가능합니다. 입력값: {}", style)
            }
            AppError::BadRequest(msg) => write!(f, "잘못된 요청입니다: {}", msg),
            AppError::Conflict(msg) => write!(f, "중복된 데이터: {}", msg),
            AppError::RateLimitExceeded(msg) => write!(f, "{}", msg),
            AppError::InternalServerError(msg) => write!(f, "서버 에러, 관리자에게 문의 바랍니다: {}", msg),
            AppError::InternalError(msg) => write!(f, "내부 에러: {}", msg),
            AppError::ExternalApiError(msg) => write!(f, "외부 API 에러: {}", msg),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let (code, message, status) = match self {
            AppError::InvalidSecretKey => (
                "AI_001".to_string(),
                "유효하지 않은 비밀 키입니다.".to_string(),
                actix_web::http::StatusCode::UNAUTHORIZED,
            ),
            AppError::InvalidToneStyle(_) => (
                "AI_002".to_string(),
                "유효하지 않은 말투 스타일입니다. KIND 또는 POLITE만 가능합니다.".to_string(),
                actix_web::http::StatusCode::BAD_REQUEST,
            ),
            AppError::BadRequest(_) => (
                "COMMON400".to_string(),
                "잘못된 요청입니다.".to_string(),
                actix_web::http::StatusCode::BAD_REQUEST,
            ),
            AppError::Conflict(msg) => (
                "COMMON409".to_string(),
                msg.clone(),
                actix_web::http::StatusCode::CONFLICT,
            ),
            AppError::RateLimitExceeded(msg) => (
                "COMMON429".to_string(),
                msg.clone(),
                actix_web::http::StatusCode::TOO_MANY_REQUESTS,
            ),
            AppError::InternalServerError(_) => (
                "COMMON500".to_string(),
                "서버 에러, 관리자에게 문의 바랍니다.".to_string(),
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ),
            AppError::InternalError(_) => (
                "COMMON500".to_string(),
                "서버 내부 에러, 관리자에게 문의 바랍니다.".to_string(),
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ),
            AppError::ExternalApiError(_) => (
                "AI_003".to_string(),
                "외부 API 호출에 실패했습니다.".to_string(),
                actix_web::http::StatusCode::SERVICE_UNAVAILABLE,
            ),
        };

        HttpResponse::build(status).json(ErrorResponse {
            is_success: false,
            code,
            message,
            result: None,
        })
    }
}

impl std::error::Error for AppError {}

