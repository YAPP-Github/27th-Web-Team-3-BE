//! API-004: 레트로룸 생성 테스트
//!
//! 테스트 대상:
//! - POST /api/v1/retro-rooms
//! - RetroRoomCreateRequest 유효성 검증
//! - RetroRoomCreateResponse 직렬화

use server::domain::retrospect::dto::{RetroRoomCreateRequest, RetroRoomCreateResponse};
use validator::Validate;

// ============== 유효성 검증 테스트 ==============

#[test]
fn should_validate_retro_room_create_request_success() {
    // Arrange
    let req = RetroRoomCreateRequest {
        title: "프로젝트 회고".to_string(),
        description: Some("스프린트 회고입니다".to_string()),
    };

    // Act & Assert
    assert!(req.validate().is_ok());
}

#[test]
fn should_fail_validation_when_title_is_empty() {
    // Arrange
    let req = RetroRoomCreateRequest {
        title: "".to_string(),
        description: None,
    };

    // Act
    let result = req.validate();

    // Assert
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.field_errors().contains_key("title"));
}

#[test]
fn should_fail_validation_when_title_exceeds_20_chars() {
    // Arrange
    let req = RetroRoomCreateRequest {
        title: "a".repeat(21),
        description: None,
    };

    // Act
    let result = req.validate();

    // Assert
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.field_errors().contains_key("title"));
}

#[test]
fn should_fail_validation_when_description_exceeds_50_chars() {
    // Arrange
    let req = RetroRoomCreateRequest {
        title: "테스트".to_string(),
        description: Some("a".repeat(51)),
    };

    // Act
    let result = req.validate();

    // Assert
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.field_errors().contains_key("description"));
}

#[test]
fn should_allow_empty_description() {
    // Arrange
    let req = RetroRoomCreateRequest {
        title: "테스트".to_string(),
        description: None,
    };

    // Act & Assert
    assert!(req.validate().is_ok());
}

// ============== 직렬화 테스트 ==============

#[test]
fn should_serialize_create_response_in_camel_case() {
    // Arrange
    let response = RetroRoomCreateResponse {
        retro_room_id: 123,
        title: "테스트".to_string(),
        invite_code: "INV-TEST-1234".to_string(),
    };

    // Act
    let json = serde_json::to_string(&response).unwrap();

    // Assert
    assert!(json.contains("retroRoomId"));
    assert!(json.contains("inviteCode"));
    assert!(!json.contains("retro_room_id"));
}

#[test]
fn should_allow_title_with_exactly_20_chars() {
    // Arrange
    let req = RetroRoomCreateRequest {
        title: "a".repeat(20),
        description: None,
    };

    // Act & Assert
    assert!(req.validate().is_ok());
}

#[test]
fn should_allow_description_with_exactly_50_chars() {
    // Arrange
    let req = RetroRoomCreateRequest {
        title: "테스트".to_string(),
        description: Some("a".repeat(50)),
    };

    // Act & Assert
    assert!(req.validate().is_ok());
}
