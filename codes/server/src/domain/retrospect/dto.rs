use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ============================================
// API-016: 회고 답변 임시 저장 DTO
// ============================================

/// 회고 답변 임시 저장 요청 DTO
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DraftSaveRequest {
    /// 임시 저장할 답변 데이터 리스트 (최소 1개, 최대 5개)
    pub drafts: Vec<DraftItem>,
}

/// 임시 저장 답변 아이템
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DraftItem {
    /// 질문 번호 (1~5)
    pub question_number: i32,
    /// 답변 내용 (최대 1,000자, null 또는 빈 문자열 허용)
    pub content: Option<String>,
}

/// 회고 답변 임시 저장 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DraftSaveResponse {
    /// 해당 회고의 고유 ID
    pub retrospect_id: i64,
    /// 최종 저장 날짜 (YYYY-MM-DD)
    pub updated_at: String,
}

/// Swagger용 회고 답변 임시 저장 성공 응답 타입
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessDraftSaveResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: DraftSaveResponse,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // API-016: DraftItem 직렬화/역직렬화 테스트
    // ========================================

    #[test]
    fn should_deserialize_draft_save_request() {
        // Arrange
        let json = r#"{
            "drafts": [
                { "questionNumber": 1, "content": "첫 번째 답변" },
                { "questionNumber": 3, "content": "세 번째 답변" }
            ]
        }"#;

        // Act
        let req: DraftSaveRequest = serde_json::from_str(json).unwrap();

        // Assert
        assert_eq!(req.drafts.len(), 2);
        assert_eq!(req.drafts[0].question_number, 1);
        assert_eq!(req.drafts[0].content.as_deref(), Some("첫 번째 답변"));
        assert_eq!(req.drafts[1].question_number, 3);
        assert_eq!(req.drafts[1].content.as_deref(), Some("세 번째 답변"));
    }

    #[test]
    fn should_deserialize_draft_item_with_null_content() {
        // Arrange
        let json = r#"{
            "drafts": [
                { "questionNumber": 2, "content": null }
            ]
        }"#;

        // Act
        let req: DraftSaveRequest = serde_json::from_str(json).unwrap();

        // Assert
        assert_eq!(req.drafts.len(), 1);
        assert_eq!(req.drafts[0].question_number, 2);
        assert!(req.drafts[0].content.is_none());
    }

    #[test]
    fn should_deserialize_draft_item_with_empty_content() {
        // Arrange
        let json = r#"{
            "drafts": [
                { "questionNumber": 1, "content": "" }
            ]
        }"#;

        // Act
        let req: DraftSaveRequest = serde_json::from_str(json).unwrap();

        // Assert
        assert_eq!(req.drafts[0].content.as_deref(), Some(""));
    }

    #[test]
    fn should_deserialize_draft_item_without_content_field() {
        // Arrange - content 필드가 아예 없는 경우
        let json = r#"{
            "drafts": [
                { "questionNumber": 1 }
            ]
        }"#;

        // Act
        let req: DraftSaveRequest = serde_json::from_str(json).unwrap();

        // Assert
        assert!(req.drafts[0].content.is_none());
    }

    #[test]
    fn should_serialize_draft_save_response_in_camel_case() {
        // Arrange
        let response = DraftSaveResponse {
            retrospect_id: 101,
            updated_at: "2026-01-24".to_string(),
        };

        // Act
        let json = serde_json::to_value(&response).unwrap();

        // Assert
        assert_eq!(json["retrospectId"], 101);
        assert_eq!(json["updatedAt"], "2026-01-24");
        // snake_case 키가 없는지 확인
        assert!(json.get("retrospect_id").is_none());
        assert!(json.get("updated_at").is_none());
    }

    #[test]
    fn should_serialize_success_draft_save_response_in_camel_case() {
        // Arrange
        let response = SuccessDraftSaveResponse {
            is_success: true,
            code: "COMMON200".to_string(),
            message: "임시 저장이 완료되었습니다.".to_string(),
            result: DraftSaveResponse {
                retrospect_id: 101,
                updated_at: "2026-01-24".to_string(),
            },
        };

        // Act
        let json = serde_json::to_value(&response).unwrap();

        // Assert
        assert_eq!(json["isSuccess"], true);
        assert_eq!(json["code"], "COMMON200");
        assert_eq!(json["message"], "임시 저장이 완료되었습니다.");
        assert_eq!(json["result"]["retrospectId"], 101);
        assert_eq!(json["result"]["updatedAt"], "2026-01-24");
    }
}
