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

    /// COMMON401: 인증 실패 (401)
    Unauthorized(String),

    /// COMMON403: 권한 없음 (403)
    Forbidden(String),

    /// RETRO4001: 프로젝트 이름 길이 유효성 검사 실패 (400)
    RetroProjectNameInvalid(String),

    /// RETRO4005: 유효하지 않은 회고 방식 (400)
    RetroMethodInvalid(String),

    /// RETRO4006: 유효하지 않은 URL 형식 (400)
    RetroUrlInvalid(String),

    /// TEAM4031: 팀 접근 권한 없음 (403)
    TeamAccessDenied(String),

    /// TEAM4041: 존재하지 않는 팀 (404)
    TeamNotFound(String),
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
            AppError::RetroProjectNameInvalid(msg) => msg.clone(),
            AppError::RetroMethodInvalid(msg) => msg.clone(),
            AppError::RetroUrlInvalid(msg) => msg.clone(),
            AppError::TeamAccessDenied(msg) => msg.clone(),
            AppError::TeamNotFound(msg) => msg.clone(),
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
            AppError::RetroProjectNameInvalid(_) => "RETRO4001",
            AppError::RetroMethodInvalid(_) => "RETRO4005",
            AppError::RetroUrlInvalid(_) => "RETRO4006",
            AppError::TeamAccessDenied(_) => "TEAM4031",
            AppError::TeamNotFound(_) => "TEAM4041",
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
            AppError::RetroProjectNameInvalid(_) => StatusCode::BAD_REQUEST,
            AppError::RetroMethodInvalid(_) => StatusCode::BAD_REQUEST,
            AppError::RetroUrlInvalid(_) => StatusCode::BAD_REQUEST,
            AppError::TeamAccessDenied(_) => StatusCode::FORBIDDEN,
            AppError::TeamNotFound(_) => StatusCode::NOT_FOUND,
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
        let message = rejection.to_string();

        // retrospectMethod 필드의 enum 파싱 실패 감지
        if message.contains("retrospectMethod") || message.contains("unknown variant") {
            return AppError::RetroMethodInvalid("유효하지 않은 회고 방식입니다.".to_string());
        }

        AppError::JsonParseFailed(message)
    }
}

/// ValidationErrors를 AppError로 변환
impl From<ValidationErrors> for AppError {
    fn from(errors: ValidationErrors) -> Self {
        let field_errors = errors.field_errors();

        // project_name 필드 검증 실패 시 RETRO4001 반환
        if field_errors.contains_key("project_name") {
            return AppError::RetroProjectNameInvalid(
                "프로젝트 이름은 1자 이상 20자 이하여야 합니다.".to_string(),
            );
        }

        let messages: Vec<String> = field_errors
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
