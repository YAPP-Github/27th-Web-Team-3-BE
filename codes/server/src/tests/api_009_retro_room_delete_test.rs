//! API-009: 레트로룸 삭제 테스트
//!
//! 테스트 대상:
//! - DELETE /api/v1/retro-rooms/{retro_room_id}
//! - DeleteRetroRoomResponse 직렬화

#[cfg(test)]
mod tests {
    use crate::domain::retrospect::dto::{DeleteRetroRoomResponse, SuccessDeleteRetroRoomResponse};

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

        // Assert
        assert!(json.contains("retroRoomId"));
        assert!(json.contains("deletedAt"));
        assert!(!json.contains("retro_room_id"));
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

        // Assert
        assert!(json.contains("\"isSuccess\":true"));
        assert!(json.contains("COMMON200"));
        assert!(json.contains("retroRoomId"));
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
}
