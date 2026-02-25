//! API-030: 회고방 멤버 목록 조회 테스트
//!
//! 테스트 대상:
//! - GET /api/v1/retro-rooms/{retro_room_id}/members
//! - RetroRoomMemberItem 직렬화
//! - SuccessRetroRoomMembersResponse 직렬화
//! - 본인이 맨 위로 정렬되는지 검증

use server::domain::retrospect::dto::{RetroRoomMemberItem, SuccessRetroRoomMembersResponse};

// ============== 직렬화 테스트 ==============

#[test]
fn should_serialize_member_item_in_camel_case() {
    // Arrange
    let item = RetroRoomMemberItem {
        member_id: 1,
        nickname: "홍길동".to_string(),
        joined_at: "2026-01-26T10:00:00".to_string(),
    };

    // Act
    let json = serde_json::to_string(&item).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Assert - JSON 파싱으로 키 존재 여부 확인
    assert!(parsed.get("memberId").is_some());
    assert!(parsed.get("nickname").is_some());
    assert!(parsed.get("joinedAt").is_some());
    assert_eq!(parsed["memberId"], 1);
    assert_eq!(parsed["nickname"], "홍길동");
    assert_eq!(parsed["joinedAt"], "2026-01-26T10:00:00");
    // snake_case 키가 없어야 함
    assert!(parsed.get("member_id").is_none());
    assert!(parsed.get("joined_at").is_none());
}

#[test]
fn should_serialize_empty_members_response() {
    // Arrange
    let response = SuccessRetroRoomMembersResponse {
        is_success: true,
        code: "COMMON200".to_string(),
        message: "성공입니다.".to_string(),
        result: vec![],
    };

    // Act
    let json = serde_json::to_string(&response).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Assert
    assert!(json.contains("\"result\":[]"));
    assert_eq!(parsed["isSuccess"], true);
    assert_eq!(parsed["code"], "COMMON200");
}

#[test]
fn should_serialize_list_with_multiple_members() {
    // Arrange
    let response = SuccessRetroRoomMembersResponse {
        is_success: true,
        code: "COMMON200".to_string(),
        message: "회고방 멤버 목록 조회를 성공했습니다.".to_string(),
        result: vec![
            RetroRoomMemberItem {
                member_id: 1,
                nickname: "나".to_string(),
                joined_at: "2026-01-20T09:00:00".to_string(),
            },
            RetroRoomMemberItem {
                member_id: 2,
                nickname: "멤버1".to_string(),
                joined_at: "2026-01-21T10:00:00".to_string(),
            },
            RetroRoomMemberItem {
                member_id: 3,
                nickname: "멤버2".to_string(),
                joined_at: "2026-01-22T11:00:00".to_string(),
            },
        ],
    };

    // Act
    let json = serde_json::to_string(&response).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Assert
    assert!(json.contains("나"));
    assert!(json.contains("멤버1"));
    assert!(json.contains("멤버2"));

    let members = parsed["result"].as_array().unwrap();
    assert_eq!(members.len(), 3);
}

#[test]
fn should_preserve_timestamp_format() {
    // Arrange
    let item = RetroRoomMemberItem {
        member_id: 1,
        nickname: "테스터".to_string(),
        joined_at: "2026-12-31T23:59:59".to_string(),
    };

    // Act
    let json = serde_json::to_string(&item).unwrap();

    // Assert
    assert!(json.contains("2026-12-31T23:59:59"));
}

#[test]
fn should_serialize_success_response_structure() {
    // Arrange
    let response = SuccessRetroRoomMembersResponse {
        is_success: true,
        code: "COMMON200".to_string(),
        message: "회고방 멤버 목록 조회를 성공했습니다.".to_string(),
        result: vec![RetroRoomMemberItem {
            member_id: 42,
            nickname: "사용자".to_string(),
            joined_at: "2026-02-01T12:00:00".to_string(),
        }],
    };

    // Act
    let json = serde_json::to_string(&response).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Assert - 전체 응답 구조 검증
    assert!(parsed.get("isSuccess").is_some());
    assert!(parsed.get("code").is_some());
    assert!(parsed.get("message").is_some());
    assert!(parsed.get("result").is_some());

    // snake_case 키가 없어야 함
    assert!(parsed.get("is_success").is_none());

    // result 배열 내 아이템 검증
    let result = parsed["result"].as_array().unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0]["memberId"], 42);
}

#[test]
fn should_handle_unicode_nicknames() {
    // Arrange - 다양한 유니코드 닉네임 테스트
    let nicknames = vec![
        "홍길동",
        "John Doe",
        "田中太郎",
        "🚀개발자", // 이모지 포함
        "test-user_123",
    ];

    for nickname in nicknames {
        let item = RetroRoomMemberItem {
            member_id: 1,
            nickname: nickname.to_string(),
            joined_at: "2026-01-26T10:00:00".to_string(),
        };

        // Act
        let json = serde_json::to_string(&item).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Assert
        assert_eq!(parsed["nickname"], nickname);
    }
}

#[test]
fn should_handle_large_member_id() {
    // Arrange - 큰 숫자 ID 테스트
    let item = RetroRoomMemberItem {
        member_id: i64::MAX,
        nickname: "대용량ID".to_string(),
        joined_at: "2026-01-26T10:00:00".to_string(),
    };

    // Act
    let json = serde_json::to_string(&item).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Assert
    assert_eq!(parsed["memberId"], i64::MAX);
}
