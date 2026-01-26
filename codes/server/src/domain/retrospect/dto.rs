use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetroRoomCreateRequest {
    #[validate(length(max = 20, message = "회고 룸 이름은 20자를 초과할 수 없습니다."))]
    pub title: String,

    #[validate(length(max = 50, message = "회고 룸 한 줄 소개는 50자를 초과할 수 없습니다."))]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetroRoomCreateResponse {
    pub retro_room_id: i64,
    pub title: String,
    pub invite_code: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessRetroRoomCreateResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: RetroRoomCreateResponse,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JoinRetroRoomRequest {
    #[validate(url(message = "유효한 URL 형식이 아닙니다."))]
    pub invite_url: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JoinRetroRoomResponse {
    pub retro_room_id: i64,
    pub title: String,
    pub joined_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessJoinRetroRoomResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: JoinRetroRoomResponse,
}

// ============== API-006: 레트로룸 목록 조회 ==============

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetroRoomListItem {
    pub retro_room_id: i64,
    pub retro_room_name: String,
    pub order_index: i32,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessRetroRoomListResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: Vec<RetroRoomListItem>,
}

// ============== API-007: 레트로룸 순서 변경 ==============

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetroRoomOrderItem {
    pub retro_room_id: i64,
    #[validate(range(min = 1, message = "orderIndex는 1 이상이어야 합니다."))]
    pub order_index: i32,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRetroRoomOrderRequest {
    #[validate(length(min = 1, message = "최소 1개 이상의 순서 정보가 필요합니다."))]
    #[validate(nested)]
    pub retro_room_orders: Vec<RetroRoomOrderItem>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessEmptyResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: Option<()>,
}

// ============== API-008: 레트로룸 이름 변경 ==============

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRetroRoomNameRequest {
    #[validate(length(min = 1, max = 20, message = "레트로룸 이름은 1~20자여야 합니다."))]
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRetroRoomNameResponse {
    pub retro_room_id: i64,
    pub retro_room_name: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessUpdateRetroRoomNameResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: UpdateRetroRoomNameResponse,
}

// ============== API-009: 레트로룸 삭제 ==============

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeleteRetroRoomResponse {
    pub retro_room_id: i64,
    pub deleted_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessDeleteRetroRoomResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: DeleteRetroRoomResponse,
}

// ============== API-010: 레트로룸 내 회고 목록 조회 ==============

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetrospectListItem {
    pub retrospect_id: i64,
    pub project_name: String,
    pub retrospect_method: String,
    pub retrospect_date: String,
    pub retrospect_time: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessRetrospectListResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: Vec<RetrospectListItem>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    // ============== API-004: 레트로룸 생성 DTO 테스트 ==============

    #[test]
    fn should_validate_retro_room_create_request_success() {
        let req = RetroRoomCreateRequest {
            title: "프로젝트 회고".to_string(),
            description: Some("스프린트 회고입니다".to_string()),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn should_fail_validation_when_title_exceeds_20_chars() {
        let req = RetroRoomCreateRequest {
            title: "a".repeat(21),
            description: None,
        };
        let result = req.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("title"));
    }

    #[test]
    fn should_fail_validation_when_description_exceeds_50_chars() {
        let req = RetroRoomCreateRequest {
            title: "테스트".to_string(),
            description: Some("a".repeat(51)),
        };
        let result = req.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("description"));
    }

    #[test]
    fn should_allow_empty_description() {
        let req = RetroRoomCreateRequest {
            title: "테스트".to_string(),
            description: None,
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn should_serialize_create_response_in_camel_case() {
        let response = RetroRoomCreateResponse {
            retro_room_id: 123,
            title: "테스트".to_string(),
            invite_code: "INV-TEST-1234".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("retroRoomId"));
        assert!(json.contains("inviteCode"));
        assert!(!json.contains("retro_room_id"));
    }

    // ============== API-005: 레트로룸 참여 DTO 테스트 ==============

    #[test]
    fn should_validate_join_request_with_valid_url() {
        let req = JoinRetroRoomRequest {
            invite_url: "https://service.com/invite/INV-TEST-1234".to_string(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn should_fail_validation_with_invalid_url_format() {
        let req = JoinRetroRoomRequest {
            invite_url: "not-a-valid-url".to_string(),
        };
        let result = req.validate();
        assert!(result.is_err());
    }

    #[test]
    fn should_serialize_join_response_in_camel_case() {
        let response = JoinRetroRoomResponse {
            retro_room_id: 456,
            title: "프로젝트".to_string(),
            joined_at: "2026-01-26T10:00:00".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("retroRoomId"));
        assert!(json.contains("joinedAt"));
    }

    // ============== API-006: 레트로룸 목록 DTO 테스트 ==============

    #[test]
    fn should_serialize_list_item_in_camel_case() {
        let item = RetroRoomListItem {
            retro_room_id: 1,
            retro_room_name: "테스트 룸".to_string(),
            order_index: 1,
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("retroRoomId"));
        assert!(json.contains("retroRoomName"));
        assert!(json.contains("orderIndex"));
    }

    #[test]
    fn should_serialize_empty_list_response() {
        let response = SuccessRetroRoomListResponse {
            is_success: true,
            code: "COMMON200".to_string(),
            message: "성공입니다.".to_string(),
            result: vec![],
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"result\":[]"));
    }

    // ============== API-007: 레트로룸 순서 변경 DTO 테스트 ==============

    #[test]
    fn should_validate_order_request_success() {
        let req = UpdateRetroRoomOrderRequest {
            retro_room_orders: vec![
                RetroRoomOrderItem {
                    retro_room_id: 1,
                    order_index: 1,
                },
                RetroRoomOrderItem {
                    retro_room_id: 2,
                    order_index: 2,
                },
            ],
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn should_fail_validation_when_order_list_is_empty() {
        let req = UpdateRetroRoomOrderRequest {
            retro_room_orders: vec![],
        };
        let result = req.validate();
        assert!(result.is_err());
    }

    #[test]
    fn should_fail_validation_when_order_index_is_zero() {
        let req = UpdateRetroRoomOrderRequest {
            retro_room_orders: vec![RetroRoomOrderItem {
                retro_room_id: 1,
                order_index: 0,
            }],
        };
        let result = req.validate();
        assert!(result.is_err());
    }

    #[test]
    fn should_fail_validation_when_order_index_is_negative() {
        let req = UpdateRetroRoomOrderRequest {
            retro_room_orders: vec![RetroRoomOrderItem {
                retro_room_id: 1,
                order_index: -1,
            }],
        };
        let result = req.validate();
        assert!(result.is_err());
    }

    #[test]
    fn should_deserialize_order_request_from_camel_case() {
        let json = r#"{"retroRoomOrders":[{"retroRoomId":1,"orderIndex":1}]}"#;
        let req: UpdateRetroRoomOrderRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.retro_room_orders.len(), 1);
        assert_eq!(req.retro_room_orders[0].retro_room_id, 1);
    }

    // ============== API-008: 레트로룸 이름 변경 DTO 테스트 ==============

    #[test]
    fn should_validate_name_update_request_success() {
        let req = UpdateRetroRoomNameRequest {
            name: "새로운 이름".to_string(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn should_fail_validation_when_name_is_empty() {
        let req = UpdateRetroRoomNameRequest {
            name: "".to_string(),
        };
        let result = req.validate();
        assert!(result.is_err());
    }

    #[test]
    fn should_fail_validation_when_name_exceeds_20_chars() {
        let req = UpdateRetroRoomNameRequest {
            name: "a".repeat(21),
        };
        let result = req.validate();
        assert!(result.is_err());
    }

    #[test]
    fn should_allow_name_with_exactly_20_chars() {
        let req = UpdateRetroRoomNameRequest {
            name: "a".repeat(20),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn should_serialize_name_update_response_in_camel_case() {
        let response = UpdateRetroRoomNameResponse {
            retro_room_id: 1,
            retro_room_name: "변경된 이름".to_string(),
            updated_at: "2026-01-26T10:00:00".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("retroRoomId"));
        assert!(json.contains("retroRoomName"));
        assert!(json.contains("updatedAt"));
    }

    // ============== API-009: 레트로룸 삭제 DTO 테스트 ==============

    #[test]
    fn should_serialize_delete_response_in_camel_case() {
        let response = DeleteRetroRoomResponse {
            retro_room_id: 123,
            deleted_at: "2026-01-26T15:00:00".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("retroRoomId"));
        assert!(json.contains("deletedAt"));
    }

    // ============== API-010: 회고 목록 DTO 테스트 ==============

    #[test]
    fn should_serialize_retrospect_list_item_in_camel_case() {
        let item = RetrospectListItem {
            retrospect_id: 1,
            project_name: "프로젝트".to_string(),
            retrospect_method: "KPT".to_string(),
            retrospect_date: "2026-01-26".to_string(),
            retrospect_time: "10:00".to_string(),
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("retrospectId"));
        assert!(json.contains("projectName"));
        assert!(json.contains("retrospectMethod"));
        assert!(json.contains("retrospectDate"));
        assert!(json.contains("retrospectTime"));
    }

    #[test]
    fn should_serialize_empty_retrospect_list() {
        let response = SuccessRetrospectListResponse {
            is_success: true,
            code: "COMMON200".to_string(),
            message: "성공입니다.".to_string(),
            result: vec![],
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"result\":[]"));
    }
}
