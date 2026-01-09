use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

/// 공통 API 응답 구조체
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub status: ResponseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseStatus {
    Success,
    Fail,
    Error,
}

impl<T: Serialize> ApiResponse<T> {
    /// 성공 응답 생성
    pub fn success(data: T) -> Self {
        Self {
            status: ResponseStatus::Success,
            data: Some(data),
            message: None,
        }
    }

    /// 성공 응답 (메시지 포함)
    pub fn success_with_message(data: T, message: impl Into<String>) -> Self {
        Self {
            status: ResponseStatus::Success,
            data: Some(data),
            message: Some(message.into()),
        }
    }
}

impl ApiResponse<()> {
    /// 실패 응답 (클라이언트 오류)
    pub fn fail(message: impl Into<String>) -> Self {
        Self {
            status: ResponseStatus::Fail,
            data: None,
            message: Some(message.into()),
        }
    }

    /// 에러 응답 (서버 오류)
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: ResponseStatus::Error,
            data: None,
            message: Some(message.into()),
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let status_code = match self.status {
            ResponseStatus::Success => StatusCode::OK,
            ResponseStatus::Fail => StatusCode::BAD_REQUEST,
            ResponseStatus::Error => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status_code, Json(self)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestData {
        id: u32,
        name: String,
    }

    #[test]
    fn test_success_response() {
        let data = TestData {
            id: 1,
            name: "test".to_string(),
        };
        let response = ApiResponse::success(data);

        assert!(matches!(response.status, ResponseStatus::Success));
        assert!(response.data.is_some());
        assert!(response.message.is_none());
    }

    #[test]
    fn test_success_with_message_response() {
        let data = TestData {
            id: 1,
            name: "test".to_string(),
        };
        let response = ApiResponse::success_with_message(data, "Success!");

        assert!(matches!(response.status, ResponseStatus::Success));
        assert!(response.data.is_some());
        assert_eq!(response.message, Some("Success!".to_string()));
    }

    #[test]
    fn test_fail_response() {
        let response = ApiResponse::<()>::fail("Invalid input");

        assert!(matches!(response.status, ResponseStatus::Fail));
        assert!(response.data.is_none());
        assert_eq!(response.message, Some("Invalid input".to_string()));
    }

    #[test]
    fn test_error_response() {
        let response = ApiResponse::<()>::error("Internal error");

        assert!(matches!(response.status, ResponseStatus::Error));
        assert!(response.data.is_none());
        assert_eq!(response.message, Some("Internal error".to_string()));
    }
}

