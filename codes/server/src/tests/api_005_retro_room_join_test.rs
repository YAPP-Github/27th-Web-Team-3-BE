//! API-005: 레트로룸 참여 테스트
//!
//! 테스트 대상:
//! - POST /api/v1/retro-rooms/join
//! - JoinRetroRoomRequest 유효성 검증
//! - JoinRetroRoomResponse 직렬화
//! - 초대 코드 추출 로직

#[cfg(test)]
mod tests {
    use crate::domain::retrospect::dto::{JoinRetroRoomRequest, JoinRetroRoomResponse};
    use crate::domain::retrospect::service::RetrospectService;
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

        // Assert
        assert!(json.contains("retroRoomId"));
        assert!(json.contains("joinedAt"));
        assert!(!json.contains("retro_room_id"));
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

        // Assert
        assert!(code.starts_with("INV-"));
        assert!(code.len() > 4);
    }
}
