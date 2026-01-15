use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;
use validator::Validate;

// ===== ToneStyle Enum =====

/// 말투 스타일
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ToneStyle {
    /// 상냥체 (e.g., "오늘 힘들었어요~")
    Kind,
    /// 정중체 (e.g., "오늘 힘들었습니다.")
    Polite,
}

impl ToneStyle {
    /// 한글 스타일 이름 반환
    pub fn style_name(&self) -> &'static str {
        match self {
            ToneStyle::Kind => "상냥체",
            ToneStyle::Polite => "정중체",
        }
    }
}

impl fmt::Display for ToneStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToneStyle::Kind => write!(f, "KIND"),
            ToneStyle::Polite => write!(f, "POLITE"),
        }
    }
}

// ===== Guide API =====

/// 최대 입력 길이 (OpenAI gpt-4o-mini 토큰 한도 고려, 한글 1자 ≈ 2-3 토큰)
pub const MAX_CONTENT_LENGTH: u64 = 5000;

/// 회고 작성 가이드 요청
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GuideRequest {
    /// 현재 작성 중인 회고 내용
    #[schema(example = "오늘 프로젝트를 진행하면서 새로운 기술을 배웠다.")]
    #[validate(length(min = 1, max = 5000, message = "내용은 1자 이상 5000자 이하여야 합니다"))]
    pub current_content: String,

    /// API 인증 키
    #[schema(example = "your-secret-key")]
    #[validate(length(min = 1, message = "비밀 키는 필수입니다"))]
    pub secret_key: String,
}

/// 회고 작성 가이드 응답
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GuideResponse {
    /// 원본 회고 내용
    #[schema(example = "오늘 프로젝트를 진행하면서 새로운 기술을 배웠다.")]
    pub current_content: String,

    /// AI가 생성한 가이드 메시지
    #[schema(
        example = "좋은 시작이에요! 어떤 기술을 배우셨나요? 그 기술을 배우게 된 계기와 느낀 점도 함께 적어보세요."
    )]
    pub guide_message: String,
}

// ===== Refine API =====

/// 회고 말투 정제 요청
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RefineRequest {
    /// 정제할 회고 내용
    #[schema(example = "오늘 일 힘들었음")]
    #[validate(length(min = 1, max = 5000, message = "내용은 1자 이상 5000자 이하여야 합니다"))]
    pub content: String,

    /// 말투 스타일 (KIND: 상냥체, POLITE: 정중체)
    #[schema(example = "KIND")]
    pub tone_style: ToneStyle,

    /// API 인증 키
    #[schema(example = "your-secret-key")]
    #[validate(length(min = 1, message = "비밀 키는 필수입니다"))]
    pub secret_key: String,
}

/// 회고 말투 정제 응답
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RefineResponse {
    /// 원본 회고 내용
    #[schema(example = "오늘 일 힘들었음")]
    pub original_content: String,

    /// 정제된 회고 내용
    #[schema(example = "오늘 일이 많이 힘들었어요.")]
    pub refined_content: String,

    /// 적용된 말투 스타일
    #[schema(example = "KIND")]
    pub tone_style: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== GuideRequest Tests =====

    #[test]
    fn guide_request_should_deserialize_from_camel_case() {
        let json = r#"{"currentContent": "test content", "secretKey": "key123"}"#;
        let request: GuideRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.current_content, "test content");
        assert_eq!(request.secret_key, "key123");
    }

    #[test]
    fn guide_request_should_validate_empty_content() {
        let request = GuideRequest {
            current_content: "".to_string(),
            secret_key: "key".to_string(),
        };

        let result = request.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("current_content"));
    }

    #[test]
    fn guide_request_should_validate_empty_secret_key() {
        let request = GuideRequest {
            current_content: "content".to_string(),
            secret_key: "".to_string(),
        };

        let result = request.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("secret_key"));
    }

    #[test]
    fn guide_request_should_pass_with_valid_input() {
        let request = GuideRequest {
            current_content: "오늘 프로젝트를 진행하면서...".to_string(),
            secret_key: "valid-key".to_string(),
        };

        let result = request.validate();
        assert!(result.is_ok());
    }

    // ===== GuideResponse Tests =====

    #[test]
    fn guide_response_should_serialize_to_camel_case() {
        let response = GuideResponse {
            current_content: "test".to_string(),
            guide_message: "guide".to_string(),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert!(json.get("currentContent").is_some());
        assert!(json.get("guideMessage").is_some());
        // snake_case 필드가 없어야 함
        assert!(json.get("current_content").is_none());
        assert!(json.get("guide_message").is_none());
    }

    #[test]
    fn guide_response_should_contain_correct_values() {
        let response = GuideResponse {
            current_content: "원본 내용".to_string(),
            guide_message: "가이드 메시지".to_string(),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["currentContent"], "원본 내용");
        assert_eq!(json["guideMessage"], "가이드 메시지");
    }

    // ===== ToneStyle Tests =====

    #[test]
    fn tone_style_should_deserialize_kind() {
        let json = r#""KIND""#;
        let style: ToneStyle = serde_json::from_str(json).unwrap();
        assert_eq!(style, ToneStyle::Kind);
    }

    #[test]
    fn tone_style_should_deserialize_polite() {
        let json = r#""POLITE""#;
        let style: ToneStyle = serde_json::from_str(json).unwrap();
        assert_eq!(style, ToneStyle::Polite);
    }

    #[test]
    fn tone_style_should_fail_invalid_style() {
        let json = r#""INVALID""#;
        let result: Result<ToneStyle, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn tone_style_should_display_correctly() {
        assert_eq!(ToneStyle::Kind.to_string(), "KIND");
        assert_eq!(ToneStyle::Polite.to_string(), "POLITE");
    }

    #[test]
    fn tone_style_should_return_korean_style_name() {
        assert_eq!(ToneStyle::Kind.style_name(), "상냥체");
        assert_eq!(ToneStyle::Polite.style_name(), "정중체");
    }

    // ===== RefineRequest Tests =====

    #[test]
    fn refine_request_should_parse_correctly() {
        let json = r#"{
            "content": "오늘 힘들었음",
            "toneStyle": "KIND",
            "secretKey": "key123"
        }"#;

        let request: RefineRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.content, "오늘 힘들었음");
        assert_eq!(request.tone_style, ToneStyle::Kind);
        assert_eq!(request.secret_key, "key123");
    }

    #[test]
    fn refine_request_should_validate_empty_content() {
        let request = RefineRequest {
            content: "".to_string(),
            tone_style: ToneStyle::Kind,
            secret_key: "key".to_string(),
        };

        let result = request.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("content"));
    }

    #[test]
    fn refine_request_should_validate_empty_secret_key() {
        let request = RefineRequest {
            content: "content".to_string(),
            tone_style: ToneStyle::Polite,
            secret_key: "".to_string(),
        };

        let result = request.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("secret_key"));
    }

    // ===== RefineResponse Tests =====

    #[test]
    fn refine_response_should_serialize_to_camel_case() {
        let response = RefineResponse {
            original_content: "test".to_string(),
            refined_content: "refined".to_string(),
            tone_style: "KIND".to_string(),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert!(json.get("originalContent").is_some());
        assert!(json.get("refinedContent").is_some());
        assert!(json.get("toneStyle").is_some());
    }

    #[test]
    fn refine_response_should_contain_correct_values() {
        let response = RefineResponse {
            original_content: "원본".to_string(),
            refined_content: "정제됨".to_string(),
            tone_style: "POLITE".to_string(),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["originalContent"], "원본");
        assert_eq!(json["refinedContent"], "정제됨");
        assert_eq!(json["toneStyle"], "POLITE");
    }

    // ===== Max Length Validation Tests =====

    #[test]
    fn guide_request_should_reject_too_long_content() {
        let long_content = "가".repeat(5001);
        let request = GuideRequest {
            current_content: long_content,
            secret_key: "key".to_string(),
        };

        let result = request.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("current_content"));
    }

    #[test]
    fn guide_request_should_accept_max_length_content() {
        let max_content = "가".repeat(5000);
        let request = GuideRequest {
            current_content: max_content,
            secret_key: "key".to_string(),
        };

        let result = request.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn refine_request_should_reject_too_long_content() {
        let long_content = "가".repeat(5001);
        let request = RefineRequest {
            content: long_content,
            tone_style: ToneStyle::Kind,
            secret_key: "key".to_string(),
        };

        let result = request.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("content"));
    }

    #[test]
    fn refine_request_should_accept_max_length_content() {
        let max_content = "가".repeat(5000);
        let request = RefineRequest {
            content: max_content,
            tone_style: ToneStyle::Polite,
            secret_key: "key".to_string(),
        };

        let result = request.validate();
        assert!(result.is_ok());
    }
}
