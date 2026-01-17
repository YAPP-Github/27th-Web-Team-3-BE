use serde::Serialize;
use utoipa::ToSchema;

/// 통일된 에러 응답 구조체
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: Option<()>,
}

impl ErrorResponse {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            is_success: false,
            code: code.into(),
            message: message.into(),
            result: None,
        }
    }
}
