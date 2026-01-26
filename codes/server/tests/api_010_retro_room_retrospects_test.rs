//! API-010: 레트로룸 내 회고 목록 조회 테스트
//!
//! 테스트 대상:
//! - GET /api/v1/retro-rooms/{retro_room_id}/retrospects
//! - RetrospectListItem 직렬화
//! - SuccessRetrospectListResponse 직렬화

use server::domain::retrospect::dto::{RetrospectListItem, SuccessRetrospectListResponse};

// ============== 직렬화 테스트 ==============

#[test]
fn should_serialize_retrospect_list_item_in_camel_case() {
    // Arrange
    let item = RetrospectListItem {
        retrospect_id: 1,
        project_name: "프로젝트".to_string(),
        retrospect_method: "KPT".to_string(),
        retrospect_date: "2026-01-26".to_string(),
        retrospect_time: "10:00".to_string(),
    };

    // Act
    let json = serde_json::to_string(&item).unwrap();

    // Assert
    assert!(json.contains("retrospectId"));
    assert!(json.contains("projectName"));
    assert!(json.contains("retrospectMethod"));
    assert!(json.contains("retrospectDate"));
    assert!(json.contains("retrospectTime"));
    assert!(!json.contains("retrospect_id"));
}

#[test]
fn should_serialize_empty_retrospect_list() {
    // Arrange
    let response = SuccessRetrospectListResponse {
        is_success: true,
        code: "COMMON200".to_string(),
        message: "성공입니다.".to_string(),
        result: vec![],
    };

    // Act
    let json = serde_json::to_string(&response).unwrap();

    // Assert
    assert!(json.contains("\"result\":[]"));
}

#[test]
fn should_serialize_list_with_multiple_retrospects() {
    // Arrange
    let response = SuccessRetrospectListResponse {
        is_success: true,
        code: "COMMON200".to_string(),
        message: "성공입니다.".to_string(),
        result: vec![
            RetrospectListItem {
                retrospect_id: 1,
                project_name: "프로젝트1".to_string(),
                retrospect_method: "KPT".to_string(),
                retrospect_date: "2026-01-26".to_string(),
                retrospect_time: "10:00".to_string(),
            },
            RetrospectListItem {
                retrospect_id: 2,
                project_name: "프로젝트2".to_string(),
                retrospect_method: "FOUR_L".to_string(),
                retrospect_date: "2026-01-27".to_string(),
                retrospect_time: "14:00".to_string(),
            },
        ],
    };

    // Act
    let json = serde_json::to_string(&response).unwrap();

    // Assert
    assert!(json.contains("프로젝트1"));
    assert!(json.contains("프로젝트2"));
    assert!(json.contains("KPT"));
    assert!(json.contains("FOUR_L"));
}

#[test]
fn should_preserve_retrospect_method_values() {
    // Arrange - 모든 회고 방식 테스트
    let methods = vec!["KPT", "FOUR_L", "FIVE_F", "PMI", "FREE"];

    for method in methods {
        let item = RetrospectListItem {
            retrospect_id: 1,
            project_name: "테스트".to_string(),
            retrospect_method: method.to_string(),
            retrospect_date: "2026-01-26".to_string(),
            retrospect_time: "10:00".to_string(),
        };

        // Act
        let json = serde_json::to_string(&item).unwrap();

        // Assert
        assert!(json.contains(method));
    }
}

#[test]
fn should_preserve_date_format() {
    // Arrange
    let item = RetrospectListItem {
        retrospect_id: 1,
        project_name: "테스트".to_string(),
        retrospect_method: "KPT".to_string(),
        retrospect_date: "2026-12-31".to_string(),
        retrospect_time: "23:59".to_string(),
    };

    // Act
    let json = serde_json::to_string(&item).unwrap();

    // Assert
    assert!(json.contains("2026-12-31"));
    assert!(json.contains("23:59"));
}
