//! API-005: 레트로룸 참여 테스트
//!
//! 테스트 대상:
//! - POST /api/v1/retro-rooms/join
//! - JoinRetroRoomRequest 유효성 검증
//! - JoinRetroRoomResponse 직렬화
//! - 초대 코드 추출 로직

use server::domain::retrospect::dto::{JoinRetroRoomRequest, JoinRetroRoomResponse};
use server::domain::retrospect::service::RetrospectService;
use validator::Validate;

// ============== 유효성 검증 테스트 ==============

#[test]
fn should_validate_join_request_with_valid_url() {
    // Arrange
    let req = JoinRetroRoomRequest {
        invite_url: "https://service.com/invite/INV-TEST-1234".to_string(),
    };

    // Act & Assert
    assert!(req.validate().is_ok());
}

#[test]
fn should_fail_validation_with_invalid_url_format() {
    // Arrange
    let req = JoinRetroRoomRequest {
        invite_url: "not-a-valid-url".to_string(),
    };

    // Act
    let result = req.validate();

    // Assert
    assert!(result.is_err());
}

#[test]
fn should_validate_join_request_with_query_param_url() {
    // Arrange
    let req = JoinRetroRoomRequest {
        invite_url: "https://service.com/join?code=INV-TEST-1234".to_string(),
    };

    // Act & Assert
    assert!(req.validate().is_ok());
}

// ============== 직렬화 테스트 ==============

#[test]
fn should_serialize_join_response_in_camel_case() {
    // Arrange
    let response = JoinRetroRoomResponse {
        retro_room_id: 456,
        title: "프로젝트".to_string(),
        joined_at: "2026-01-26T10:00:00".to_string(),
    };

    // Act
    let json = serde_json::to_string(&response).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Assert - JSON 파싱으로 키 존재 여부 확인
    assert!(parsed.get("retroRoomId").is_some());
    assert!(parsed.get("title").is_some());
    assert!(parsed.get("joinedAt").is_some());
    // snake_case 키가 없어야 함
    assert!(parsed.get("retro_room_id").is_none());
    assert!(parsed.get("joined_at").is_none());
}

// ============== 초대 코드 추출 테스트 ==============

#[test]
fn should_extract_invite_code_from_path_segment() {
    // Arrange
    let url = "https://service.com/invite/INV-A1B2-C3D4";

    // Act
    let result = RetrospectService::extract_invite_code(url);

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "INV-A1B2-C3D4");
}

#[test]
fn should_extract_invite_code_from_query_parameter() {
    // Arrange
    let url = "https://service.com/join?code=INV-A1B2-C3D4";

    // Act
    let result = RetrospectService::extract_invite_code(url);

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "INV-A1B2-C3D4");
}

#[test]
fn should_extract_invite_code_from_query_with_multiple_params() {
    // Arrange
    let url = "https://service.com/join?ref=abc&code=INV-TEST-1234&foo=bar";

    // Act
    let result = RetrospectService::extract_invite_code(url);

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "INV-TEST-1234");
}

#[test]
fn should_return_error_for_invalid_url() {
    // Arrange
    let url = "https://service.com/invalid/path";

    // Act
    let result = RetrospectService::extract_invite_code(url);

    // Assert
    assert!(result.is_err());
}

#[test]
fn should_return_error_for_empty_code() {
    // Arrange
    let url = "https://service.com/join?code=";

    // Act
    let result = RetrospectService::extract_invite_code(url);

    // Assert
    assert!(result.is_err());
}

#[test]
fn should_generate_valid_invite_code() {
    // Act
    let code = RetrospectService::generate_invite_code();

    // Assert - INV-XXXX-XXXX 형식 검증 (정확히 13자)
    // 인덱스: I(0) N(1) V(2) -(3) X(4) X(5) X(6) X(7) -(8) X(9) X(10) X(11) X(12)
    assert_eq!(code.len(), 13, "초대 코드는 정확히 13자여야 함");
    assert!(code.starts_with("INV-"), "INV- 접두사 필수");
    assert_eq!(
        code.chars().nth(3),
        Some('-'),
        "4번째 문자(인덱스 3)는 '-'여야 함"
    );
    assert_eq!(
        code.chars().nth(8),
        Some('-'),
        "9번째 문자(인덱스 8)는 '-'여야 함"
    );

    // 숫자 부분 검증 (XXXX-XXXX)
    let parts: Vec<&str> = code.split('-').collect();
    assert_eq!(parts.len(), 3, "하이픈으로 구분된 3개 파트");
    assert_eq!(parts[0], "INV");
    assert_eq!(parts[1].len(), 4, "첫 번째 숫자 부분은 4자리");
    assert_eq!(parts[2].len(), 4, "두 번째 숫자 부분은 4자리");
    assert!(
        parts[1].chars().all(|c| c.is_ascii_digit()),
        "첫 번째 부분은 숫자만"
    );
    assert!(
        parts[2].chars().all(|c| c.is_ascii_digit()),
        "두 번째 부분은 숫자만"
    );
}
