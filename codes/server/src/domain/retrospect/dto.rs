use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use super::entity::retrospect::{Model as RetrospectModel, RetrospectMethod};

/// 회고 생성 요청 DTO
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateRetrospectRequest {
    /// 회고가 속한 팀의 고유 ID
    #[validate(range(min = 1, message = "팀 ID는 1 이상이어야 합니다"))]
    pub team_id: i64,

    /// 프로젝트 이름 (최소 1자, 최대 20자)
    #[validate(length(
        min = 1,
        max = 20,
        message = "프로젝트 이름은 1자 이상 20자 이하여야 합니다"
    ))]
    pub project_name: String,

    /// 회고 날짜 (ISO 8601 형식: YYYY-MM-DD)
    #[validate(length(
        min = 10,
        max = 10,
        message = "날짜 형식이 올바르지 않습니다. (YYYY-MM-DD 형식 필요)"
    ))]
    pub retrospect_date: String,

    /// 회고 시간 (HH:mm 형식, 한국 시간 기준)
    #[validate(length(
        min = 5,
        max = 5,
        message = "시간 형식이 올바르지 않습니다. (HH:mm 형식 필요)"
    ))]
    pub retrospect_time: String,

    /// 회고 방식
    pub retrospect_method: RetrospectMethod,

    /// 참고 자료 URL 리스트 (최대 10개)
    #[validate(length(max = 10, message = "참고 URL은 최대 10개까지 등록 가능합니다"))]
    #[serde(default)]
    pub reference_urls: Vec<String>,
}

/// 회고 생성 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateRetrospectResponse {
    /// 생성된 회고 고유 ID
    pub retrospect_id: i64,
    /// 회고가 속한 팀의 고유 ID
    pub team_id: i64,
    /// 저장된 프로젝트 이름
    pub project_name: String,
}

/// Swagger용 성공 응답 타입
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessCreateRetrospectResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: CreateRetrospectResponse,
}

// ============================================
// API-010: 팀 회고 목록 조회 DTO
// ============================================

/// 팀 회고 목록 아이템 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TeamRetrospectListItem {
    /// 회고 고유 식별자
    pub retrospect_id: i64,
    /// 프로젝트 이름
    pub project_name: String,
    /// 회고 방식
    pub retrospect_method: RetrospectMethod,
    /// 회고 날짜 (yyyy-MM-dd)
    pub retrospect_date: String,
    /// 회고 시간 (HH:mm)
    pub retrospect_time: String,
}

impl From<RetrospectModel> for TeamRetrospectListItem {
    fn from(model: RetrospectModel) -> Self {
        Self {
            retrospect_id: model.retrospect_id,
            project_name: model.title,
            retrospect_method: model.retrospect_method,
            retrospect_date: model.start_time.format("%Y-%m-%d").to_string(),
            retrospect_time: model.start_time.format("%H:%M").to_string(),
        }
    }
}

/// Swagger용 팀 회고 목록 성공 응답 타입
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessTeamRetrospectListResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: Vec<TeamRetrospectListItem>,
}

// ============================================
// API-014: 회고 참석자 등록 DTO
// ============================================

/// 회고 참석 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateParticipantResponse {
    /// 참석자 등록 고유 식별자
    pub participant_id: i64,
    /// 참석한 유저의 고유 ID
    pub member_id: i64,
    /// 참석한 유저의 닉네임
    pub nickname: String,
}

/// Swagger용 회고 참석 성공 응답 타입
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessCreateParticipantResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: CreateParticipantResponse,
}

// ============================================
// API-018: 회고 참고자료 목록 조회 DTO
// ============================================

/// 참고자료 아이템 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceItem {
    /// 자료 고유 식별자
    pub reference_id: i64,
    /// 자료 별칭 (예: 깃허브 레포지토리)
    pub url_name: String,
    /// 참고자료 주소
    pub url: String,
}

/// Swagger용 참고자료 목록 성공 응답 타입
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessReferencesListResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: Vec<ReferenceItem>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    fn create_valid_request() -> CreateRetrospectRequest {
        CreateRetrospectRequest {
            team_id: 1,
            project_name: "테스트 프로젝트".to_string(),
            retrospect_date: "2025-01-25".to_string(),
            retrospect_time: "14:00".to_string(),
            retrospect_method: RetrospectMethod::Kpt,
            reference_urls: vec![],
        }
    }

    // ========================================
    // project_name 검증 테스트
    // ========================================

    #[test]
    fn should_fail_validation_when_project_name_is_empty() {
        // Arrange
        let request = CreateRetrospectRequest {
            project_name: "".to_string(),
            ..create_valid_request()
        };

        // Act
        let result = request.validate();

        // Assert
        assert!(result.is_err());
        let errors = result.unwrap_err();
        let field_errors = errors.field_errors();
        assert!(field_errors.contains_key("project_name"));
    }

    #[test]
    fn should_fail_validation_when_project_name_exceeds_20_chars() {
        // Arrange
        let request = CreateRetrospectRequest {
            project_name: "가".repeat(21), // 21자
            ..create_valid_request()
        };

        // Act
        let result = request.validate();

        // Assert
        assert!(result.is_err());
        let errors = result.unwrap_err();
        let field_errors = errors.field_errors();
        assert!(field_errors.contains_key("project_name"));
    }

    #[test]
    fn should_pass_validation_when_project_name_is_exactly_20_chars() {
        // Arrange
        let request = CreateRetrospectRequest {
            project_name: "가".repeat(20), // 정확히 20자
            ..create_valid_request()
        };

        // Act
        let result = request.validate();

        // Assert
        assert!(result.is_ok());
    }

    // ========================================
    // team_id 검증 테스트
    // ========================================

    #[test]
    fn should_fail_validation_when_team_id_is_zero() {
        // Arrange
        let request = CreateRetrospectRequest {
            team_id: 0,
            ..create_valid_request()
        };

        // Act
        let result = request.validate();

        // Assert
        assert!(result.is_err());
        let errors = result.unwrap_err();
        let field_errors = errors.field_errors();
        assert!(field_errors.contains_key("team_id"));
    }

    #[test]
    fn should_fail_validation_when_team_id_is_negative() {
        // Arrange
        let request = CreateRetrospectRequest {
            team_id: -1,
            ..create_valid_request()
        };

        // Act
        let result = request.validate();

        // Assert
        assert!(result.is_err());
        let errors = result.unwrap_err();
        let field_errors = errors.field_errors();
        assert!(field_errors.contains_key("team_id"));
    }

    #[test]
    fn should_pass_validation_when_team_id_is_positive() {
        // Arrange
        let request = CreateRetrospectRequest {
            team_id: 1,
            ..create_valid_request()
        };

        // Act
        let result = request.validate();

        // Assert
        assert!(result.is_ok());
    }

    // ========================================
    // retrospect_date 검증 테스트
    // ========================================

    #[test]
    fn should_fail_validation_when_retrospect_date_is_too_short() {
        // Arrange
        let request = CreateRetrospectRequest {
            retrospect_date: "2025-1-1".to_string(), // 8자 (형식 오류)
            ..create_valid_request()
        };

        // Act
        let result = request.validate();

        // Assert
        assert!(result.is_err());
        let errors = result.unwrap_err();
        let field_errors = errors.field_errors();
        assert!(field_errors.contains_key("retrospect_date"));
    }

    #[test]
    fn should_fail_validation_when_retrospect_date_is_too_long() {
        // Arrange
        let request = CreateRetrospectRequest {
            retrospect_date: "2025-01-251".to_string(), // 11자 (형식 오류)
            ..create_valid_request()
        };

        // Act
        let result = request.validate();

        // Assert
        assert!(result.is_err());
        let errors = result.unwrap_err();
        let field_errors = errors.field_errors();
        assert!(field_errors.contains_key("retrospect_date"));
    }

    #[test]
    fn should_pass_validation_when_retrospect_date_has_correct_format() {
        // Arrange
        let request = CreateRetrospectRequest {
            retrospect_date: "2025-01-25".to_string(), // 정확히 10자
            ..create_valid_request()
        };

        // Act
        let result = request.validate();

        // Assert
        assert!(result.is_ok());
    }

    // ========================================
    // reference_urls 검증 테스트
    // ========================================

    #[test]
    fn should_fail_validation_when_reference_urls_exceed_10() {
        // Arrange
        let urls: Vec<String> = (0..11)
            .map(|i| format!("https://example.com/{}", i))
            .collect();
        let request = CreateRetrospectRequest {
            reference_urls: urls, // 11개
            ..create_valid_request()
        };

        // Act
        let result = request.validate();

        // Assert
        assert!(result.is_err());
        let errors = result.unwrap_err();
        let field_errors = errors.field_errors();
        assert!(field_errors.contains_key("reference_urls"));
    }

    #[test]
    fn should_pass_validation_when_reference_urls_are_exactly_10() {
        // Arrange
        let urls: Vec<String> = (0..10)
            .map(|i| format!("https://example.com/{}", i))
            .collect();
        let request = CreateRetrospectRequest {
            reference_urls: urls, // 정확히 10개
            ..create_valid_request()
        };

        // Act
        let result = request.validate();

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_pass_validation_when_reference_urls_are_empty() {
        // Arrange
        let request = CreateRetrospectRequest {
            reference_urls: vec![],
            ..create_valid_request()
        };

        // Act
        let result = request.validate();

        // Assert
        assert!(result.is_ok());
    }

    // ========================================
    // retrospect_time 검증 테스트
    // ========================================

    #[test]
    fn should_fail_validation_when_retrospect_time_is_too_short() {
        // Arrange
        let request = CreateRetrospectRequest {
            retrospect_time: "9:00".to_string(), // 4자 (형식 오류)
            ..create_valid_request()
        };

        // Act
        let result = request.validate();

        // Assert
        assert!(result.is_err());
        let errors = result.unwrap_err();
        let field_errors = errors.field_errors();
        assert!(field_errors.contains_key("retrospect_time"));
    }

    #[test]
    fn should_fail_validation_when_retrospect_time_is_too_long() {
        // Arrange
        let request = CreateRetrospectRequest {
            retrospect_time: "14:00:00".to_string(), // 8자 (형식 오류)
            ..create_valid_request()
        };

        // Act
        let result = request.validate();

        // Assert
        assert!(result.is_err());
        let errors = result.unwrap_err();
        let field_errors = errors.field_errors();
        assert!(field_errors.contains_key("retrospect_time"));
    }

    #[test]
    fn should_pass_validation_when_retrospect_time_has_correct_format() {
        // Arrange
        let request = CreateRetrospectRequest {
            retrospect_time: "14:30".to_string(), // 정확히 5자
            ..create_valid_request()
        };

        // Act
        let result = request.validate();

        // Assert
        assert!(result.is_ok());
    }
}
