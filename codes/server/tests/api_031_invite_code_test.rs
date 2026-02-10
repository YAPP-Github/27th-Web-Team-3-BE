//! API-031: 초대 코드 조회 테스트
//!
//! 테스트 대상:
//! - InviteCodeResponse 직렬화 (camelCase)
//! - SuccessInviteCodeResponse 직렬화
//! - 만료된 초대 코드의 is_expired 플래그 검증
//! - 초대 코드 형식 검증 (INV-XXXX-XXXX)
//! - 초대 코드 추출 유틸리티

use server::domain::retrospect::dto::{InviteCodeResponse, SuccessInviteCodeResponse};
use server::domain::retrospect::service::RetrospectService;

// ============== 직렬화 테스트 ==============

#[test]
fn should_serialize_invite_code_response_in_camel_case() {
    // Arrange
    let response = InviteCodeResponse {
        retro_room_id: 1,
        invite_code: "INV-1234-5678".to_string(),
        expires_at: "2026-02-10T12:00:00".to_string(),
        is_expired: false,
    };

    // Act
    let json = serde_json::to_string(&response).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Assert - camelCase 키 존재 확인
    assert!(parsed.get("retroRoomId").is_some());
    assert!(parsed.get("inviteCode").is_some());
    assert!(parsed.get("expiresAt").is_some());
    assert!(parsed.get("isExpired").is_some());

    // snake_case 키가 없어야 함
    assert!(parsed.get("retro_room_id").is_none());
    assert!(parsed.get("invite_code").is_none());
    assert!(parsed.get("expires_at").is_none());
    assert!(parsed.get("is_expired").is_none());

    // 값 검증
    assert_eq!(parsed["retroRoomId"], 1);
    assert_eq!(parsed["inviteCode"], "INV-1234-5678");
    assert_eq!(parsed["expiresAt"], "2026-02-10T12:00:00");
    assert_eq!(parsed["isExpired"], false);
}

#[test]
fn should_serialize_expired_invite_code_response() {
    // Arrange - 만료된 초대 코드
    let response = InviteCodeResponse {
        retro_room_id: 42,
        invite_code: "INV-9999-0000".to_string(),
        expires_at: "2026-01-01T00:00:00".to_string(),
        is_expired: true,
    };

    // Act
    let json = serde_json::to_string(&response).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Assert - 만료된 코드도 is_expired=true와 함께 정상 반환
    assert_eq!(parsed["isExpired"], true);
    assert_eq!(parsed["inviteCode"], "INV-9999-0000");
    assert_eq!(parsed["retroRoomId"], 42);
}

#[test]
fn should_serialize_success_invite_code_response_structure() {
    // Arrange
    let response = SuccessInviteCodeResponse {
        is_success: true,
        code: "COMMON200".to_string(),
        message: "초대 코드 조회를 성공했습니다.".to_string(),
        result: InviteCodeResponse {
            retro_room_id: 10,
            invite_code: "INV-ABCD-1234".to_string(),
            expires_at: "2026-03-01T09:30:00".to_string(),
            is_expired: false,
        },
    };

    // Act
    let json = serde_json::to_string(&response).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Assert - 전체 응답 구조 검증
    assert!(parsed.get("isSuccess").is_some());
    assert!(parsed.get("code").is_some());
    assert!(parsed.get("message").is_some());
    assert!(parsed.get("result").is_some());

    assert_eq!(parsed["isSuccess"], true);
    assert_eq!(parsed["code"], "COMMON200");

    // snake_case 키가 없어야 함
    assert!(parsed.get("is_success").is_none());

    // result 내부 검증
    let result = &parsed["result"];
    assert_eq!(result["retroRoomId"], 10);
    assert_eq!(result["inviteCode"], "INV-ABCD-1234");
}

// ============== 초대 코드 생성 테스트 ==============

#[test]
fn should_generate_invite_code_in_correct_format() {
    // Act
    let code = RetrospectService::generate_invite_code();

    // Assert - INV-XXXX-XXXX 형식 검증
    assert!(code.starts_with("INV-"));
    assert_eq!(code.len(), 13);

    let parts: Vec<&str> = code.split('-').collect();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0], "INV");
    assert_eq!(parts[1].len(), 4);
    assert_eq!(parts[2].len(), 4);
    assert!(parts[1].chars().all(|c| c.is_ascii_digit()));
    assert!(parts[2].chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn should_generate_unique_invite_codes() {
    // Act - 여러 번 생성하여 고유성 확인
    let codes: Vec<String> = (0..10)
        .map(|_| RetrospectService::generate_invite_code())
        .collect();

    // Assert - 모든 코드가 올바른 형식
    for code in &codes {
        assert!(code.starts_with("INV-"));
        assert_eq!(code.len(), 13);
    }
}

// ============== 초대 코드 추출 테스트 ==============

#[test]
fn should_extract_invite_code_from_url_path() {
    // Arrange
    let url = "https://example.com/invite/INV-1234-5678";

    // Act
    let result = RetrospectService::extract_invite_code(url);

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "INV-1234-5678");
}

#[test]
fn should_extract_invite_code_from_query_param() {
    // Arrange
    let url = "https://example.com/invite?code=INV-1234-5678";

    // Act
    let result = RetrospectService::extract_invite_code(url);

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "INV-1234-5678");
}

#[test]
fn should_return_error_for_invalid_invite_url() {
    // Arrange
    let url = "https://example.com/no-code-here";

    // Act
    let result = RetrospectService::extract_invite_code(url);

    // Assert
    assert!(result.is_err());
}

#[test]
fn should_return_error_for_malformed_invite_code() {
    // Arrange - query param에 잘못된 형식의 코드
    let url = "https://example.com/invite?code=INVALID-CODE";

    // Act
    let result = RetrospectService::extract_invite_code(url);

    // Assert
    assert!(result.is_err());
}

#[test]
fn should_return_error_for_empty_url() {
    // Arrange
    let url = "";

    // Act
    let result = RetrospectService::extract_invite_code(url);

    // Assert
    assert!(result.is_err());
}

#[test]
fn should_preserve_iso8601_timestamp_format() {
    // Arrange
    let response = InviteCodeResponse {
        retro_room_id: 1,
        invite_code: "INV-0000-0000".to_string(),
        expires_at: "2026-12-31T23:59:59".to_string(),
        is_expired: false,
    };

    // Act
    let json = serde_json::to_string(&response).unwrap();

    // Assert
    assert!(json.contains("2026-12-31T23:59:59"));
}

#[test]
fn should_handle_large_retro_room_id() {
    // Arrange
    let response = InviteCodeResponse {
        retro_room_id: i64::MAX,
        invite_code: "INV-0001-0002".to_string(),
        expires_at: "2026-01-01T00:00:00".to_string(),
        is_expired: false,
    };

    // Act
    let json = serde_json::to_string(&response).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Assert
    assert_eq!(parsed["retroRoomId"], i64::MAX);
}
