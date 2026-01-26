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
}
