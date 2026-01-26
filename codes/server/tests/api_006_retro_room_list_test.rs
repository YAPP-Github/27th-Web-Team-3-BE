//! API-006: 레트로룸 목록 조회 테스트
//!
//! 테스트 대상:
//! - GET /api/v1/retro-rooms
//! - RetroRoomListItem 직렬화
//! - SuccessRetroRoomListResponse 직렬화

use server::domain::retrospect::dto::{RetroRoomListItem, SuccessRetroRoomListResponse};

// ============== 직렬화 테스트 ==============

#[test]
fn should_serialize_list_item_in_camel_case() {
    // Arrange
    let item = RetroRoomListItem {
        retro_room_id: 1,
        retro_room_name: "테스트 룸".to_string(),
        order_index: 1,
    };

    // Act
    let json = serde_json::to_string(&item).unwrap();

    // Assert
    assert!(json.contains("retroRoomId"));
    assert!(json.contains("retroRoomName"));
    assert!(json.contains("orderIndex"));
    assert!(!json.contains("retro_room_id"));
}

#[test]
fn should_serialize_empty_list_response() {
    // Arrange
    let response = SuccessRetroRoomListResponse {
        is_success: true,
        code: "COMMON200".to_string(),
        message: "성공입니다.".to_string(),
        result: vec![],
    };

    // Act
    let json = serde_json::to_string(&response).unwrap();

    // Assert
    assert!(json.contains("\"result\":[]"));
    assert!(json.contains("\"isSuccess\":true"));
}

#[test]
fn should_serialize_list_with_multiple_items() {
    // Arrange
    let response = SuccessRetroRoomListResponse {
        is_success: true,
        code: "COMMON200".to_string(),
        message: "성공입니다.".to_string(),
        result: vec![
            RetroRoomListItem {
                retro_room_id: 1,
                retro_room_name: "룸1".to_string(),
                order_index: 1,
            },
            RetroRoomListItem {
                retro_room_id: 2,
                retro_room_name: "룸2".to_string(),
                order_index: 2,
            },
        ],
    };

    // Act
    let json = serde_json::to_string(&response).unwrap();

    // Assert
    assert!(json.contains("룸1"));
    assert!(json.contains("룸2"));
}

#[test]
fn should_preserve_order_index_values() {
    // Arrange
    let item = RetroRoomListItem {
        retro_room_id: 1,
        retro_room_name: "테스트".to_string(),
        order_index: 999,
    };

    // Act
    let json = serde_json::to_string(&item).unwrap();

    // Assert
    assert!(json.contains("999"));
}
