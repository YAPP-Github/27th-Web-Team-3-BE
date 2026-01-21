use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// 말투 스타일 Enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum ToneStyle {
    /// 상냥체 (친근하고 상냥한 표현)
    Kind,
    /// 정중체 (존댓말 등 정중한 표현)
    Polite,
}

/// 회고 정제 요청 DTO
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RefineRequest {
    /// 정제할 회고 내용 (1 ~ 5000자)
    #[validate(length(
        min = 1,
        max = 5000,
        message = "내용은 1자 이상 5000자 이하여야 합니다"
    ))]
    pub content: String,

    /// 말투 스타일 (KIND 또는 POLITE)
    pub tone_style: ToneStyle,

    /// API 인증 키
    #[validate(length(min = 1, message = "비밀 키는 필수입니다"))]
    pub secret_key: String,
}

/// 회고 정제 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RefineResponse {
    /// 원본 회고 내용
    pub original_content: String,

    /// 정제된 회고 내용
    pub refined_content: String,

    /// 적용된 말투 스타일
    pub tone_style: ToneStyle,
}

impl RefineResponse {
    pub fn new(original_content: String, refined_content: String, tone_style: ToneStyle) -> Self {
        Self {
            original_content,
            refined_content,
            tone_style,
        }
    }
}

/// 회고 정제 성공 응답 (OpenAPI 스키마용)
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RefineSuccessResponse {
    /// 성공 여부
    #[schema(example = true)]
    pub is_success: bool,

    /// 응답 코드
    #[schema(example = "COMMON200")]
    pub code: String,

    /// 응답 메시지
    #[schema(example = "성공입니다.")]
    pub message: String,

    /// 정제 결과
    pub result: RefineResponse,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_deserialize_kind_tone_style() {
        // Arrange
        let json = r#""KIND""#;

        // Act
        let result: ToneStyle = serde_json::from_str(json).unwrap();

        // Assert
        assert_eq!(result, ToneStyle::Kind);
    }

    #[test]
    fn should_deserialize_polite_tone_style() {
        // Arrange
        let json = r#""POLITE""#;

        // Act
        let result: ToneStyle = serde_json::from_str(json).unwrap();

        // Assert
        assert_eq!(result, ToneStyle::Polite);
    }

    #[test]
    fn should_reject_invalid_tone_style() {
        // Arrange
        let json = r#""INVALID""#;

        // Act
        let result: Result<ToneStyle, _> = serde_json::from_str(json);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn should_serialize_tone_style_as_uppercase() {
        // Arrange
        let tone = ToneStyle::Kind;

        // Act
        let json = serde_json::to_string(&tone).unwrap();

        // Assert
        assert_eq!(json, "\"KIND\"");
    }

    #[test]
    fn should_deserialize_refine_request() {
        // Arrange
        let json = r#"{
            "content": "오늘 힘들었음",
            "toneStyle": "KIND",
            "secretKey": "test-secret"
        }"#;

        // Act
        let result: RefineRequest = serde_json::from_str(json).unwrap();

        // Assert
        assert_eq!(result.content, "오늘 힘들었음");
        assert_eq!(result.tone_style, ToneStyle::Kind);
        assert_eq!(result.secret_key, "test-secret");
    }
}
