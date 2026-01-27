use std::borrow::Cow;
use std::fmt;

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use super::entity::retrospect::{Model as RetrospectModel, RetrospectMethod};
use crate::domain::member::entity::member_retro::RetrospectStatus;

/// 참고 URL 최대 길이 (개별 URL당)
pub const REFERENCE_URL_MAX_LENGTH: usize = 2048;

/// 참고 URL 개별 길이 검증
fn validate_reference_url_items(urls: &[String]) -> Result<(), validator::ValidationError> {
    for url in urls {
        if url.len() > REFERENCE_URL_MAX_LENGTH {
            let mut err = validator::ValidationError::new("url_too_long");
            err.message = Some(Cow::Borrowed("각 URL은 최대 2048자까지 허용됩니다"));
            return Err(err);
        }
    }
    Ok(())
}

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

    /// 참고 자료 URL 리스트 (최대 10개, 각 URL 최대 2048자)
    #[validate(
        length(max = 10, message = "참고 URL은 최대 10개까지 등록 가능합니다"),
        custom(function = "validate_reference_url_items")
    )]
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
// API-016: 회고 답변 임시 저장 DTO
// ============================================

/// 회고 답변 임시 저장 요청 DTO
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DraftSaveRequest {
    /// 임시 저장할 답변 데이터 리스트 (최소 1개, 최대 5개)
    pub drafts: Vec<DraftItem>,
}

/// 임시 저장 답변 아이템
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DraftItem {
    /// 질문 번호 (1~5)
    pub question_number: i32,
    /// 답변 내용 (최대 1,000자, null 또는 빈 문자열 허용)
    pub content: Option<String>,
}

/// 회고 답변 임시 저장 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DraftSaveResponse {
    /// 해당 회고의 고유 ID
    pub retrospect_id: i64,
    /// 최종 저장 날짜 (YYYY-MM-DD)
    pub updated_at: String,
}

/// Swagger용 회고 답변 임시 저장 성공 응답 타입
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessDraftSaveResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: DraftSaveResponse,
}

// ============================================
// API-017: 회고 최종 제출 DTO
// ============================================

/// 회고 제출 요청 DTO
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubmitRetrospectRequest {
    /// 제출할 답변 리스트 (정확히 5개, 서비스 레이어에서 검증)
    pub answers: Vec<SubmitAnswerItem>,
}

/// 제출 답변 아이템
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubmitAnswerItem {
    /// 질문 번호 (1~5)
    pub question_number: i32,
    /// 답변 내용 (1~1,000자)
    pub content: String,
}

/// 회고 제출 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubmitRetrospectResponse {
    /// 제출된 회고의 고유 ID
    pub retrospect_id: i64,
    /// 최종 제출 날짜 (YYYY-MM-DD)
    pub submitted_at: String,
    /// 현재 회고 상태
    pub status: RetrospectStatus,
}

/// Swagger용 회고 제출 성공 응답 타입
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessSubmitRetrospectResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: SubmitRetrospectResponse,
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

// ============================================
// API-019: 보관함 조회 DTO
// ============================================

/// 보관함 조회 기간 필터
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, ToSchema)]
pub enum StorageRangeFilter {
    /// 전체 기간
    #[serde(rename = "ALL")]
    #[default]
    All,
    /// 최근 3개월
    #[serde(rename = "3_MONTHS")]
    ThreeMonths,
    /// 최근 6개월
    #[serde(rename = "6_MONTHS")]
    SixMonths,
    /// 최근 1년
    #[serde(rename = "1_YEAR")]
    OneYear,
}

impl fmt::Display for StorageRangeFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageRangeFilter::All => write!(f, "ALL"),
            StorageRangeFilter::ThreeMonths => write!(f, "3_MONTHS"),
            StorageRangeFilter::SixMonths => write!(f, "6_MONTHS"),
            StorageRangeFilter::OneYear => write!(f, "1_YEAR"),
        }
    }
}

impl StorageRangeFilter {
    /// 필터에 해당하는 일수 반환 (None이면 전체 기간)
    pub fn days(&self) -> Option<i64> {
        match self {
            StorageRangeFilter::All => None,
            StorageRangeFilter::ThreeMonths => Some(90),
            StorageRangeFilter::SixMonths => Some(180),
            StorageRangeFilter::OneYear => Some(365),
        }
    }
}

/// 보관함 조회 쿼리 파라미터
#[derive(Debug, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct StorageQueryParams {
    /// 기간 필터 (기본값: ALL)
    pub range: Option<StorageRangeFilter>,
}

/// 보관함 내 개별 회고 아이템
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StorageRetrospectItem {
    /// 회고 고유 ID
    pub retrospect_id: i64,
    /// 표시 날짜 (YYYY-MM-DD)
    pub display_date: String,
    /// 회고 제목 (프로젝트명)
    pub title: String,
    /// 회고 방식
    pub retrospect_method: RetrospectMethod,
    /// 참여자 수
    pub member_count: i64,
}

/// 연도별 회고 그룹
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StorageYearGroup {
    /// 연도 레이블 (예: "2026년")
    pub year_label: String,
    /// 해당 연도의 회고 리스트 (최신순 정렬)
    pub retrospects: Vec<StorageRetrospectItem>,
}

/// 보관함 조회 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StorageResponse {
    /// 연도별 회고 그룹 리스트 (최신 연도 순)
    pub years: Vec<StorageYearGroup>,
}

/// Swagger용 보관함 조회 성공 응답 타입
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessStorageResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: StorageResponse,
}

// ============================================
// API-012: 회고 상세 정보 조회 DTO
// ============================================

/// 회고 상세 정보 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetrospectDetailResponse {
    /// 회고가 속한 팀의 고유 ID
    pub team_id: i64,
    /// 회고 제목 (프로젝트명)
    pub title: String,
    /// 회고 시작 날짜 (YYYY-MM-DD)
    pub start_time: String,
    /// 회고 유형
    pub retro_category: RetrospectMethod,
    /// 참여 멤버 리스트 (참석 등록일 기준 오름차순 정렬)
    pub members: Vec<RetrospectMemberItem>,
    /// 회고 전체 좋아요 합계
    pub total_like_count: i64,
    /// 회고 전체 댓글 합계
    pub total_comment_count: i64,
    /// 해당 회고의 질문 리스트 (index 기준 오름차순 정렬, 최대 5개)
    pub questions: Vec<RetrospectQuestionItem>,
}

/// 회고 참여 멤버 아이템
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetrospectMemberItem {
    /// 멤버 고유 식별자
    pub member_id: i64,
    /// 멤버 이름 (닉네임)
    pub user_name: String,
}

/// 회고 질문 아이템
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetrospectQuestionItem {
    /// 질문 순서 (1~5)
    pub index: i32,
    /// 질문 내용
    pub content: String,
}

/// Swagger용 회고 상세 정보 조회 성공 응답 타입
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessRetrospectDetailResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: RetrospectDetailResponse,
}

// ============================================
// API-022: 회고 분석 DTO
// ============================================

/// 감정 랭킹 아이템
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EmotionRankItem {
    /// 순위 (1부터 시작, 감정 빈도 기준 내림차순)
    pub rank: i32,
    /// 감정 키워드 (예: "피로", "뿌듯")
    pub label: String,
    /// 해당 감정에 대한 상세 설명 및 원인 분석
    pub description: String,
    /// 해당 감정을 선택/언급한 횟수
    pub count: i32,
}

/// 개인 미션 아이템
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MissionItem {
    /// 개인 미션 제목 (예: "감정 표현 적극적으로 하기")
    pub mission_title: String,
    /// 개인 미션 상세 설명 및 인사이트
    pub mission_desc: String,
}

/// 사용자별 개인 미션 아이템
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PersonalMissionItem {
    /// 사용자 고유 ID
    pub user_id: i64,
    /// 사용자 이름
    pub user_name: String,
    /// 해당 사용자의 개인 미션 리스트 (정확히 3개)
    pub missions: Vec<MissionItem>,
}

/// 회고 분석 응답 데이터
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisResponse {
    /// 팀 전체를 위한 AI 분석 메시지
    pub team_insight: String,
    /// 감정 키워드 순위 리스트 (내림차순 정렬, 정확히 3개)
    pub emotion_rank: Vec<EmotionRankItem>,
    /// 사용자별 개인 맞춤 미션 리스트 (userId 오름차순 정렬)
    pub personal_missions: Vec<PersonalMissionItem>,
}

/// Swagger용 회고 분석 성공 응답 타입
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessAnalysisResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: AnalysisResponse,
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

    #[test]
    fn should_fail_validation_when_individual_url_exceeds_max_length() {
        // Arrange
        let long_url = format!("https://example.com/{}", "a".repeat(2050));
        let request = CreateRetrospectRequest {
            reference_urls: vec![long_url],
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
    fn should_pass_validation_when_url_is_within_max_length() {
        // Arrange
        let valid_url = format!("https://example.com/{}", "a".repeat(2020));
        let request = CreateRetrospectRequest {
            reference_urls: vec![valid_url],
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

    // ========================================
    // API-016: DraftItem 직렬화/역직렬화 테스트
    // ========================================

    #[test]
    fn should_deserialize_draft_save_request() {
        // Arrange
        let json = r#"{
            "drafts": [
                { "questionNumber": 1, "content": "첫 번째 답변" },
                { "questionNumber": 3, "content": "세 번째 답변" }
            ]
        }"#;

        // Act
        let req: DraftSaveRequest = serde_json::from_str(json).unwrap();

        // Assert
        assert_eq!(req.drafts.len(), 2);
        assert_eq!(req.drafts[0].question_number, 1);
        assert_eq!(req.drafts[0].content.as_deref(), Some("첫 번째 답변"));
        assert_eq!(req.drafts[1].question_number, 3);
        assert_eq!(req.drafts[1].content.as_deref(), Some("세 번째 답변"));
    }

    #[test]
    fn should_deserialize_draft_item_with_null_content() {
        // Arrange
        let json = r#"{
            "drafts": [
                { "questionNumber": 2, "content": null }
            ]
        }"#;

        // Act
        let req: DraftSaveRequest = serde_json::from_str(json).unwrap();

        // Assert
        assert_eq!(req.drafts.len(), 1);
        assert_eq!(req.drafts[0].question_number, 2);
        assert!(req.drafts[0].content.is_none());
    }

    #[test]
    fn should_deserialize_draft_item_with_empty_content() {
        // Arrange
        let json = r#"{
            "drafts": [
                { "questionNumber": 1, "content": "" }
            ]
        }"#;

        // Act
        let req: DraftSaveRequest = serde_json::from_str(json).unwrap();

        // Assert
        assert_eq!(req.drafts[0].content.as_deref(), Some(""));
    }

    #[test]
    fn should_deserialize_draft_item_without_content_field() {
        // Arrange - content 필드가 아예 없는 경우
        let json = r#"{
            "drafts": [
                { "questionNumber": 1 }
            ]
        }"#;

        // Act
        let req: DraftSaveRequest = serde_json::from_str(json).unwrap();

        // Assert
        assert!(req.drafts[0].content.is_none());
    }

    #[test]
    fn should_serialize_draft_save_response_in_camel_case() {
        // Arrange
        let response = DraftSaveResponse {
            retrospect_id: 101,
            updated_at: "2026-01-24".to_string(),
        };

        // Act
        let json = serde_json::to_value(&response).unwrap();

        // Assert
        assert_eq!(json["retrospectId"], 101);
        assert_eq!(json["updatedAt"], "2026-01-24");
        // snake_case 키가 없는지 확인
        assert!(json.get("retrospect_id").is_none());
        assert!(json.get("updated_at").is_none());
    }

    #[test]
    fn should_serialize_success_draft_save_response_in_camel_case() {
        // Arrange
        let response = SuccessDraftSaveResponse {
            is_success: true,
            code: "COMMON200".to_string(),
            message: "임시 저장이 완료되었습니다.".to_string(),
            result: DraftSaveResponse {
                retrospect_id: 101,
                updated_at: "2026-01-24".to_string(),
            },
        };

        // Act
        let json = serde_json::to_value(&response).unwrap();

        // Assert
        assert_eq!(json["isSuccess"], true);
        assert_eq!(json["code"], "COMMON200");
        assert_eq!(json["message"], "임시 저장이 완료되었습니다.");
        assert_eq!(json["result"]["retrospectId"], 101);
        assert_eq!(json["result"]["updatedAt"], "2026-01-24");
    }

    // ========================================
    // API-019: StorageRangeFilter 테스트
    // ========================================

    #[test]
    fn should_deserialize_all_range_filter() {
        // Arrange & Act
        let filter: StorageRangeFilter = serde_json::from_str("\"ALL\"").unwrap();

        // Assert
        assert_eq!(filter, StorageRangeFilter::All);
        assert!(filter.days().is_none());
    }

    #[test]
    fn should_deserialize_3_months_range_filter() {
        // Arrange & Act
        let filter: StorageRangeFilter = serde_json::from_str("\"3_MONTHS\"").unwrap();

        // Assert
        assert_eq!(filter, StorageRangeFilter::ThreeMonths);
        assert_eq!(filter.days(), Some(90));
    }

    #[test]
    fn should_deserialize_6_months_range_filter() {
        // Arrange & Act
        let filter: StorageRangeFilter = serde_json::from_str("\"6_MONTHS\"").unwrap();

        // Assert
        assert_eq!(filter, StorageRangeFilter::SixMonths);
        assert_eq!(filter.days(), Some(180));
    }

    #[test]
    fn should_deserialize_1_year_range_filter() {
        // Arrange & Act
        let filter: StorageRangeFilter = serde_json::from_str("\"1_YEAR\"").unwrap();

        // Assert
        assert_eq!(filter, StorageRangeFilter::OneYear);
        assert_eq!(filter.days(), Some(365));
    }

    #[test]
    fn should_fail_deserialize_invalid_range_filter() {
        // Arrange & Act
        let result: Result<StorageRangeFilter, _> = serde_json::from_str("\"INVALID\"");

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn should_default_to_all() {
        // Arrange & Act
        let filter = StorageRangeFilter::default();

        // Assert
        assert_eq!(filter, StorageRangeFilter::All);
    }

    #[test]
    fn should_display_range_filter_correctly() {
        // Assert
        assert_eq!(StorageRangeFilter::All.to_string(), "ALL");
        assert_eq!(StorageRangeFilter::ThreeMonths.to_string(), "3_MONTHS");
        assert_eq!(StorageRangeFilter::SixMonths.to_string(), "6_MONTHS");
        assert_eq!(StorageRangeFilter::OneYear.to_string(), "1_YEAR");
    }

    #[test]
    fn should_serialize_storage_response_in_camel_case() {
        // Arrange
        let response = StorageResponse {
            years: vec![StorageYearGroup {
                year_label: "2026년".to_string(),
                retrospects: vec![StorageRetrospectItem {
                    retrospect_id: 1,
                    display_date: "2026-01-24".to_string(),
                    title: "테스트 프로젝트".to_string(),
                    retrospect_method: RetrospectMethod::Kpt,
                    member_count: 5,
                }],
            }],
        };

        // Act
        let json = serde_json::to_value(&response).unwrap();

        // Assert
        assert!(json["years"][0]["yearLabel"].is_string());
        assert!(json["years"][0]["retrospects"][0]["retrospectId"].is_number());
        assert!(json["years"][0]["retrospects"][0]["displayDate"].is_string());
        assert!(json["years"][0]["retrospects"][0]["retrospectMethod"].is_string());
        assert!(json["years"][0]["retrospects"][0]["memberCount"].is_number());
    }

    #[test]
    fn should_serialize_empty_storage_response() {
        // Arrange
        let response = StorageResponse { years: vec![] };

        // Act
        let json = serde_json::to_value(&response).unwrap();

        // Assert
        assert_eq!(json["years"].as_array().unwrap().len(), 0);
    }

    // ========================================
    // API-012: RetrospectDetailResponse 테스트
    // ========================================

    #[test]
    fn should_serialize_retrospect_detail_response_in_camel_case() {
        // Arrange
        let response = RetrospectDetailResponse {
            team_id: 789,
            title: "3차 스프린트 회고".to_string(),
            start_time: "2026-01-24".to_string(),
            retro_category: RetrospectMethod::Kpt,
            members: vec![
                RetrospectMemberItem {
                    member_id: 1,
                    user_name: "김민철".to_string(),
                },
                RetrospectMemberItem {
                    member_id: 2,
                    user_name: "카이".to_string(),
                },
            ],
            total_like_count: 156,
            total_comment_count: 42,
            questions: vec![
                RetrospectQuestionItem {
                    index: 1,
                    content: "계속 유지하고 싶은 좋은 점은 무엇인가요?".to_string(),
                },
                RetrospectQuestionItem {
                    index: 2,
                    content: "개선이 필요한 문제점은 무엇인가요?".to_string(),
                },
                RetrospectQuestionItem {
                    index: 3,
                    content: "다음에 시도해보고 싶은 것은 무엇인가요?".to_string(),
                },
            ],
        };

        // Act
        let json = serde_json::to_value(&response).unwrap();

        // Assert
        assert_eq!(json["teamId"], 789);
        assert_eq!(json["title"], "3차 스프린트 회고");
        assert_eq!(json["startTime"], "2026-01-24");
        assert_eq!(json["retroCategory"], "KPT");
        assert_eq!(json["totalLikeCount"], 156);
        assert_eq!(json["totalCommentCount"], 42);

        // members 검증
        let members = json["members"].as_array().unwrap();
        assert_eq!(members.len(), 2);
        assert_eq!(members[0]["memberId"], 1);
        assert_eq!(members[0]["userName"], "김민철");
        assert_eq!(members[1]["memberId"], 2);
        assert_eq!(members[1]["userName"], "카이");

        // questions 검증
        let questions = json["questions"].as_array().unwrap();
        assert_eq!(questions.len(), 3);
        assert_eq!(questions[0]["index"], 1);
        assert!(questions[0]["content"].as_str().unwrap().contains("유지"));
        assert_eq!(questions[1]["index"], 2);
        assert_eq!(questions[2]["index"], 3);
    }

    #[test]
    fn should_serialize_retrospect_detail_with_empty_members_and_questions() {
        // Arrange
        let response = RetrospectDetailResponse {
            team_id: 1,
            title: "빈 회고".to_string(),
            start_time: "2026-01-01".to_string(),
            retro_category: RetrospectMethod::Free,
            members: vec![],
            total_like_count: 0,
            total_comment_count: 0,
            questions: vec![],
        };

        // Act
        let json = serde_json::to_value(&response).unwrap();

        // Assert
        assert_eq!(json["members"].as_array().unwrap().len(), 0);
        assert_eq!(json["questions"].as_array().unwrap().len(), 0);
        assert_eq!(json["totalLikeCount"], 0);
        assert_eq!(json["totalCommentCount"], 0);
        assert_eq!(json["retroCategory"], "FREE");
    }

    #[test]
    fn should_serialize_all_retro_categories_correctly() {
        // Arrange & Act & Assert
        let categories = vec![
            (RetrospectMethod::Kpt, "KPT"),
            (RetrospectMethod::FourL, "FOUR_L"),
            (RetrospectMethod::FiveF, "FIVE_F"),
            (RetrospectMethod::Pmi, "PMI"),
            (RetrospectMethod::Free, "FREE"),
        ];

        for (category, expected) in categories {
            let response = RetrospectDetailResponse {
                team_id: 1,
                title: "테스트".to_string(),
                start_time: "2026-01-01".to_string(),
                retro_category: category,
                members: vec![],
                total_like_count: 0,
                total_comment_count: 0,
                questions: vec![],
            };

            let json = serde_json::to_value(&response).unwrap();
            assert_eq!(json["retroCategory"], expected);
        }
    }

    #[test]
    fn should_serialize_member_item_in_camel_case() {
        // Arrange
        let member = RetrospectMemberItem {
            member_id: 42,
            user_name: "테스트유저".to_string(),
        };

        // Act
        let json = serde_json::to_value(&member).unwrap();

        // Assert
        assert_eq!(json["memberId"], 42);
        assert_eq!(json["userName"], "테스트유저");
        // snake_case 키가 없는지 확인
        assert!(json.get("member_id").is_none());
        assert!(json.get("user_name").is_none());
    }

    #[test]
    fn should_serialize_question_item_in_camel_case() {
        // Arrange
        let question = RetrospectQuestionItem {
            index: 3,
            content: "테스트 질문입니다".to_string(),
        };

        // Act
        let json = serde_json::to_value(&question).unwrap();

        // Assert
        assert_eq!(json["index"], 3);
        assert_eq!(json["content"], "테스트 질문입니다");
    }
}
