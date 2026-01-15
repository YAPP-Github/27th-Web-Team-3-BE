use serde::Serialize;
use chrono::Utc;

/// 통일된 성공 응답 구조체
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    pub message: String,
    pub timestamp: String,
}

impl<T: Serialize> ApiResponse<T> {
    /// 성공 응답 생성 (기본 메시지)
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: "Success".to_string(),
            timestamp: Utc::now().to_rfc3339(),
        }
    }

    /// 성공 응답 생성 (커스텀 메시지)
    pub fn success_with_message(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: message.into(),
            timestamp: Utc::now().to_rfc3339(),
        }
    }
}

/// 데이터 없는 성공 응답 (예: 삭제 완료)
impl ApiResponse<()> {
    pub fn success_no_data(message: impl Into<String>) -> Self {
        Self {
            success: true,
            data: None,
            message: message.into(),
            timestamp: Utc::now().to_rfc3339(),
        }
    }
}

/// 통일된 에러 응답 구조체
#[derive(Serialize)]
pub struct ApiErrorResponse {
    pub success: bool,
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    pub timestamp: String,
}

impl ApiErrorResponse {
    /// 에러 응답 생성
    pub fn new(
        error: impl Into<String>,
        message: impl Into<String>,
        details: Option<String>,
    ) -> Self {
        Self {
            success: false,
            error: error.into(),
            message: message.into(),
            details,
            timestamp: Utc::now().to_rfc3339(),
        }
    }

    /// 상세 정보와 함께 에러 응답 생성
    pub fn with_details(
        error: impl Into<String>,
        message: impl Into<String>,
        details: impl Into<String>,
    ) -> Self {
        Self::new(error, message, Some(details.into()))
    }
}

