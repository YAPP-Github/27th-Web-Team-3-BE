//! API-009: 레트로룸 삭제 테스트
//!
//! 테스트 대상:
//! - DELETE /api/v1/retro-rooms/{retro_room_id}
//! - DeleteRetroRoomResponse 직렬화

use server::domain::retrospect::dto::{DeleteRetroRoomResponse, SuccessDeleteRetroRoomResponse};

// ============== 직렬화 테스트 ==============

#[test]
fn should_serialize_delete_response_in_camel_case() {
    // Arrange
    let response = DeleteRetroRoomResponse {
        retro_room_id: 123,
        deleted_at: "2026-01-26T15:00:00".to_string(),
    };

    // Act
    let json = serde_json::to_string(&response).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Assert - JSON 파싱으로 키 존재 여부 확인
    assert!(parsed.get("retroRoomId").is_some());
    assert!(parsed.get("deletedAt").is_some());
    assert_eq!(parsed["retroRoomId"], 123);
    // snake_case 키가 없어야 함
    assert!(parsed.get("retro_room_id").is_none());
    assert!(parsed.get("deleted_at").is_none());
}

#[test]
fn should_serialize_success_delete_response() {
    // Arrange
    let response = SuccessDeleteRetroRoomResponse {
        is_success: true,
        code: "COMMON200".to_string(),
        message: "성공입니다.".to_string(),
        result: DeleteRetroRoomResponse {
            retro_room_id: 456,
            deleted_at: "2026-01-26T16:00:00".to_string(),
        },
    };

    // Act
    let json = serde_json::to_string(&response).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Assert - JSON 파싱으로 검증
    assert_eq!(parsed["isSuccess"], true);
    assert_eq!(parsed["code"], "COMMON200");
    assert!(parsed.get("result").is_some());
    assert_eq!(parsed["result"]["retroRoomId"], 456);
}

#[test]
fn should_preserve_timestamp_format() {
    // Arrange
    let response = DeleteRetroRoomResponse {
        retro_room_id: 1,
        deleted_at: "2026-12-31T23:59:59".to_string(),
    };

    // Act
    let json = serde_json::to_string(&response).unwrap();

    // Assert
    assert!(json.contains("2026-12-31T23:59:59"));
}
