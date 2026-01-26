//! API-008: 레트로룸 이름 변경 테스트
//!
//! 테스트 대상:
//! - PATCH /api/v1/retro-rooms/{retro_room_id}/name
//! - UpdateRetroRoomNameRequest 유효성 검증
//! - UpdateRetroRoomNameResponse 직렬화

use server::domain::retrospect::dto::{UpdateRetroRoomNameRequest, UpdateRetroRoomNameResponse};
use validator::Validate;

// ============== 유효성 검증 테스트 ==============

#[test]
fn should_validate_name_update_request_success() {
    // Arrange
    let req = UpdateRetroRoomNameRequest {
        name: "새로운 이름".to_string(),
    };

    // Act & Assert
    assert!(req.validate().is_ok());
}

#[test]
fn should_fail_validation_when_name_is_empty() {
    // Arrange
    let req = UpdateRetroRoomNameRequest {
        name: "".to_string(),
    };

    // Act
    let result = req.validate();

    // Assert
    assert!(result.is_err());
}

#[test]
fn should_fail_validation_when_name_exceeds_20_chars() {
    // Arrange
    let req = UpdateRetroRoomNameRequest {
        name: "a".repeat(21),
    };

    // Act
    let result = req.validate();

    // Assert
    assert!(result.is_err());
}

#[test]
fn should_allow_name_with_exactly_20_chars() {
    // Arrange
    let req = UpdateRetroRoomNameRequest {
        name: "a".repeat(20),
    };

    // Act & Assert
    assert!(req.validate().is_ok());
}

#[test]
fn should_allow_name_with_exactly_1_char() {
    // Arrange
    let req = UpdateRetroRoomNameRequest {
        name: "a".to_string(),
    };

    // Act & Assert
    assert!(req.validate().is_ok());
}

#[test]
fn should_allow_name_with_korean_characters() {
    // Arrange
    let req = UpdateRetroRoomNameRequest {
        name: "한글이름테스트".to_string(),
    };

    // Act & Assert
    assert!(req.validate().is_ok());
}

#[test]
fn should_allow_name_with_special_characters() {
    // Arrange
    let req = UpdateRetroRoomNameRequest {
        name: "룸-이름_123".to_string(),
    };

    // Act & Assert
    assert!(req.validate().is_ok());
}

// ============== 직렬화 테스트 ==============

#[test]
fn should_serialize_name_update_response_in_camel_case() {
    // Arrange
    let response = UpdateRetroRoomNameResponse {
        retro_room_id: 1,
        retro_room_name: "변경된 이름".to_string(),
        updated_at: "2026-01-26T10:00:00".to_string(),
    };

    // Act
    let json = serde_json::to_string(&response).unwrap();

    // Assert
    assert!(json.contains("retroRoomId"));
    assert!(json.contains("retroRoomName"));
    assert!(json.contains("updatedAt"));
    assert!(!json.contains("retro_room_id"));
}

// ============== 역직렬화 테스트 ==============

#[test]
fn should_deserialize_name_request_from_camel_case() {
    // Arrange
    let json = r#"{"name":"새이름"}"#;

    // Act
    let req: UpdateRetroRoomNameRequest = serde_json::from_str(json).unwrap();

    // Assert
    assert_eq!(req.name, "새이름");
}
