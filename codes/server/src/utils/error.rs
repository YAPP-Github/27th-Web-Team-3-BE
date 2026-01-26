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

    /// TEAM4031: 팀 접근 권한 없음 (403)
    TeamAccessDenied(String),

    /// AI_001: 유효하지 않은 비밀 키 (401)
    InvalidSecretKey(String),

    /// RETRO4091: 이미 분석 완료된 회고 (409)
    RetroAlreadyAnalyzed(String),

    /// AI4031: 월간 분석 가능 횟수 초과 (403)
    AiMonthlyLimitExceeded(String),

    /// RETRO4042: 분석할 회고 답변 데이터 부족 (404)
    RetroInsufficientData(String),

    /// AI5001: 데이터 종합 분석 중 오류 (500)
    AiAnalysisFailed(String),

    /// AI_003: AI 연결 실패 (500)
    AiConnectionFailed(String),

    /// AI_005: AI 서비스 일시적 오류 (503)
    AiServiceUnavailable(String),

    /// AI_006: AI 일반 오류 (500)
    AiGeneralError(String),
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
            AppError::TeamAccessDenied(msg) => msg.clone(),
            AppError::RetroAlreadyAnalyzed(msg) => msg.clone(),
            AppError::InvalidSecretKey(msg) => msg.clone(),
            AppError::AiMonthlyLimitExceeded(msg) => msg.clone(),
            AppError::RetroInsufficientData(msg) => msg.clone(),
            AppError::AiAnalysisFailed(msg) => msg.clone(),
            AppError::AiConnectionFailed(msg) => msg.clone(),
            AppError::AiServiceUnavailable(msg) => msg.clone(),
            AppError::AiGeneralError(msg) => msg.clone(),
        }
    }

    /// 에러 코드 반환
    pub fn error_code(&self) -> &str {
        match self {
            AppError::BadRequest(_) => "COMMON400",
            AppError::ValidationError(_) => "COMMON400",
            AppError::InternalError(_) => "COMMON500",
            AppError::JsonParseFailed(_) => "COMMON400",
            AppError::Unauthorized(_) => "COMMON401",
            AppError::Forbidden(_) => "COMMON403",
            AppError::RetrospectNotFound(_) => "RETRO4041",
            AppError::RetroAnswersMissing(_) => "RETRO4002",
            AppError::RetroAnswerTooLong(_) => "RETRO4003",
            AppError::RetroAnswerWhitespaceOnly(_) => "RETRO4007",
            AppError::RetroAlreadySubmitted(_) => "RETRO4033",
            AppError::TeamAccessDenied(_) => "TEAM4031",
            AppError::RetroAlreadyAnalyzed(_) => "RETRO4091",
            AppError::InvalidSecretKey(_) => "AI_001",
            AppError::AiMonthlyLimitExceeded(_) => "AI4031",
            AppError::RetroInsufficientData(_) => "RETRO4042",
            AppError::AiAnalysisFailed(_) => "AI5001",
            AppError::AiConnectionFailed(_) => "AI_003",
            AppError::AiServiceUnavailable(_) => "AI_005",
            AppError::AiGeneralError(_) => "AI_006",
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
            AppError::TeamAccessDenied(_) => StatusCode::FORBIDDEN,
            AppError::RetroAlreadyAnalyzed(_) => StatusCode::CONFLICT,
            AppError::InvalidSecretKey(_) => StatusCode::UNAUTHORIZED,
            AppError::AiMonthlyLimitExceeded(_) => StatusCode::FORBIDDEN,
            AppError::RetroInsufficientData(_) => StatusCode::NOT_FOUND,
            AppError::AiAnalysisFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::AiConnectionFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::AiServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::AiGeneralError(_) => StatusCode::INTERNAL_SERVER_ERROR,
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
            AppError::AiAnalysisFailed(msg) => {
                error!("AI Analysis Failed: {}", msg);
            }
            AppError::AiConnectionFailed(msg) => {
                error!("AI Connection Failed: {}", msg);
            }
            AppError::AiServiceUnavailable(msg) => {
                error!("AI Service Unavailable: {}", msg);
            }
            AppError::AiGeneralError(msg) => {
                error!("AI General Error: {}", msg);
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
