use serde::Serialize;
use utoipa::ToSchema;

/// API 공통 응답 형식
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BaseResponse<T: Serialize> {
    /// 성공 여부
    #[schema(example = true)]
    pub is_success: bool,

    /// 응답 코드
    #[schema(example = "COMMON200")]
    pub code: String,

    /// 응답 메시지
    #[schema(example = "성공입니다.")]
    pub message: String,

    /// 응답 데이터
    pub result: Option<T>,
}

/// 에러 응답 형식
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    /// 성공 여부 (에러 시 항상 false)
    #[schema(example = false)]
    pub is_success: bool,

    /// 에러 코드
    #[schema(example = "AI_001")]
    pub code: String,

    /// 에러 메시지
    #[schema(example = "유효하지 않은 비밀 키입니다.")]
    pub message: String,
}

impl<T: Serialize> BaseResponse<T> {
    /// 성공 응답 생성 (Phase 4에서 사용)
    #[allow(dead_code)]
    pub fn success(result: T) -> Self {
        Self {
            is_success: true,
            code: "COMMON200".to_string(),
            message: "성공입니다.".to_string(),
            result: Some(result),
        }
    }

    /// 에러 응답 생성
    pub fn error(code: &str, message: &str) -> BaseResponse<()> {
        BaseResponse {
            is_success: false,
            code: code.to_string(),
            message: message.to_string(),
            result: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_response_format() {
        #[derive(Serialize)]
        struct TestData {
            value: String,
        }

        let response = BaseResponse::success(TestData {
            value: "test".to_string(),
        });
        let json = serde_json::to_value(&response).unwrap();

        assert_eq!(json["isSuccess"], true);
        assert_eq!(json["code"], "COMMON200");
        assert_eq!(json["message"], "성공입니다.");
        assert_eq!(json["result"]["value"], "test");
    }

    #[test]
    fn test_error_response_format() {
        let response = BaseResponse::<()>::error("AI_001", "유효하지 않은 비밀 키입니다.");
        let json = serde_json::to_value(&response).unwrap();

        assert_eq!(json["isSuccess"], false);
        assert_eq!(json["code"], "AI_001");
        assert!(json["result"].is_null());
    }
}
