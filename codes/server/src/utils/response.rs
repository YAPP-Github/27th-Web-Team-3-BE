use serde::Serialize;
use utoipa::ToSchema;

/// API 명세에 맞는 기본 응답 구조체
///
/// 형식:
/// ```json
/// {
///   "isSuccess": true,
///   "code": "COMMON200",
///   "message": "성공입니다.",
///   "result": { ... }
/// }
/// ```
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseResponse<T: Serialize> {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: Option<T>,
}

impl<T: Serialize> BaseResponse<T> {
    /// 성공 응답 생성
    pub fn success(result: T) -> Self {
        Self {
            is_success: true,
            code: "COMMON200".to_string(),
            message: "성공입니다.".to_string(),
            result: Some(result),
        }
    }
}

/// 에러 응답 구조체
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: Option<()>,
}

impl ErrorResponse {
    /// 에러 응답 생성
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            is_success: false,
            code: code.into(),
            message: message.into(),
            result: None,
        }
    }
}
