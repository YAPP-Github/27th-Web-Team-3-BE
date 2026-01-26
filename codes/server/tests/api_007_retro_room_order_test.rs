//! API-007: 레트로룸 순서 변경 테스트
//!
//! 테스트 대상:
//! - PATCH /api/v1/retro-rooms/order
//! - UpdateRetroRoomOrderRequest 유효성 검증
//! - RetroRoomOrderItem 유효성 검증

use server::domain::retrospect::dto::{RetroRoomOrderItem, UpdateRetroRoomOrderRequest};
use validator::Validate;

// ============== 유효성 검증 테스트 ==============

#[test]
fn should_validate_order_request_success() {
    // Arrange
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

    // Act & Assert
    assert!(req.validate().is_ok());
}

#[test]
fn should_fail_validation_when_order_list_is_empty() {
    // Arrange
    let req = UpdateRetroRoomOrderRequest {
        retro_room_orders: vec![],
    };

    // Act
    let result = req.validate();

    // Assert
    assert!(result.is_err());
}

#[test]
fn should_fail_validation_when_order_index_is_zero() {
    // Arrange
    let req = UpdateRetroRoomOrderRequest {
        retro_room_orders: vec![RetroRoomOrderItem {
            retro_room_id: 1,
            order_index: 0,
        }],
    };

    // Act
    let result = req.validate();

    // Assert
    assert!(result.is_err());
}

#[test]
fn should_fail_validation_when_order_index_is_negative() {
    // Arrange
    let req = UpdateRetroRoomOrderRequest {
        retro_room_orders: vec![RetroRoomOrderItem {
            retro_room_id: 1,
            order_index: -1,
        }],
    };

    // Act
    let result = req.validate();

    // Assert
    assert!(result.is_err());
}

#[test]
fn should_validate_order_index_with_large_value() {
    // Arrange
    let req = UpdateRetroRoomOrderRequest {
        retro_room_orders: vec![RetroRoomOrderItem {
            retro_room_id: 1,
            order_index: 9999,
        }],
    };

    // Act & Assert
    assert!(req.validate().is_ok());
}

// ============== 역직렬화 테스트 ==============

#[test]
fn should_deserialize_order_request_from_camel_case() {
    // Arrange
    let json = r#"{"retroRoomOrders":[{"retroRoomId":1,"orderIndex":1}]}"#;

    // Act
    let req: UpdateRetroRoomOrderRequest = serde_json::from_str(json).unwrap();

    // Assert
    assert_eq!(req.retro_room_orders.len(), 1);
    assert_eq!(req.retro_room_orders[0].retro_room_id, 1);
    assert_eq!(req.retro_room_orders[0].order_index, 1);
}

#[test]
fn should_deserialize_order_request_with_multiple_items() {
    // Arrange
    let json = r#"{"retroRoomOrders":[{"retroRoomId":1,"orderIndex":1},{"retroRoomId":2,"orderIndex":2}]}"#;

    // Act
    let req: UpdateRetroRoomOrderRequest = serde_json::from_str(json).unwrap();

    // Assert
    assert_eq!(req.retro_room_orders.len(), 2);
}

// ============== 직렬화 테스트 ==============

#[test]
fn should_serialize_order_item_in_camel_case() {
    // Arrange
    let item = RetroRoomOrderItem {
        retro_room_id: 1,
        order_index: 5,
    };

    // Act
    let json = serde_json::to_string(&item).unwrap();

    // Assert
    assert!(json.contains("retroRoomId"));
    assert!(json.contains("orderIndex"));
    assert!(!json.contains("retro_room_id"));
}
