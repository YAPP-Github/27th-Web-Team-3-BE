use std::fmt;

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::domain::member::entity::member_retro::RetrospectStatus;
use crate::domain::retrospect::entity::retrospect::RetroCategory;

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
    /// 회고 유형
    pub retro_category: RetroCategory,
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
    pub retro_category: RetroCategory,
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

#[cfg(test)]
mod tests {
    use super::*;

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
                    retro_category: RetroCategory::Kpt,
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
        assert!(json["years"][0]["retrospects"][0]["retroCategory"].is_string());
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
            retro_category: RetroCategory::Kpt,
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
            retro_category: RetroCategory::Free,
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
            (RetroCategory::Kpt, "KPT"),
            (RetroCategory::FourL, "FOUR_L"),
            (RetroCategory::FiveF, "FIVE_F"),
            (RetroCategory::Pmi, "PMI"),
            (RetroCategory::Free, "FREE"),
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
