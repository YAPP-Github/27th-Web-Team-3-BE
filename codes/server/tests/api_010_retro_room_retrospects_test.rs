//! API-010: 레트로룸 내 회고 목록 조회 테스트
//!
//! 테스트 대상:
//! - GET /api/v1/retro-rooms/{retro_room_id}/retrospects
//! - RetrospectListItem 직렬화
//! - SuccessRetrospectListResponse 직렬화
//! - RetrospectListStatus 직렬화

use server::domain::retrospect::dto::{
    RetrospectListItem, RetrospectListStatus, SuccessRetrospectListResponse,
};

// ============== 직렬화 테스트 ==============

#[test]
fn should_serialize_retrospect_list_item_in_camel_case() {
    // Arrange
    let item = RetrospectListItem {
        retrospect_id: 1,
        project_name: "프로젝트".to_string(),
        retrospect_method: "KPT".to_string(),
        retrospect_date: "2026-01-26".to_string(),
        participant_count: 5,
        status: RetrospectListStatus::InProgress,
    };

    // Act
    let json = serde_json::to_string(&item).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Assert - JSON 파싱으로 키 존재 여부 확인
    assert!(parsed.get("retrospectId").is_some());
    assert!(parsed.get("projectName").is_some());
    assert!(parsed.get("retrospectMethod").is_some());
    assert!(parsed.get("retrospectDate").is_some());
    assert!(parsed.get("participantCount").is_some());
    assert!(parsed.get("status").is_some());
    assert_eq!(parsed["retrospectId"], 1);
    assert_eq!(parsed["projectName"], "프로젝트");
    assert_eq!(parsed["participantCount"], 5);
    assert_eq!(parsed["status"], "IN_PROGRESS");
    // snake_case 키가 없어야 함
    assert!(parsed.get("retrospect_id").is_none());
    assert!(parsed.get("project_name").is_none());
    assert!(parsed.get("participant_count").is_none());
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
                participant_count: 3,
                status: RetrospectListStatus::InProgress,
            },
            RetrospectListItem {
                retrospect_id: 2,
                project_name: "프로젝트2".to_string(),
                retrospect_method: "FOUR_L".to_string(),
                retrospect_date: "2026-01-27".to_string(),
                participant_count: 5,
                status: RetrospectListStatus::Draft,
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
    assert!(json.contains("\"participantCount\":3"));
    assert!(json.contains("\"participantCount\":5"));
    assert!(json.contains("\"status\":\"IN_PROGRESS\""));
    assert!(json.contains("\"status\":\"DRAFT\""));
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
            participant_count: 2,
            status: RetrospectListStatus::InProgress,
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
        participant_count: 4,
        status: RetrospectListStatus::Completed,
    };

    // Act
    let json = serde_json::to_string(&item).unwrap();

    // Assert
    assert!(json.contains("2026-12-31"));
}

// ============== RetrospectListStatus 직렬화 테스트 ==============

#[test]
fn should_serialize_status_in_progress_as_screaming_snake_case() {
    // Arrange
    let item = RetrospectListItem {
        retrospect_id: 1,
        project_name: "테스트".to_string(),
        retrospect_method: "KPT".to_string(),
        retrospect_date: "2026-01-26".to_string(),
        participant_count: 0,
        status: RetrospectListStatus::InProgress,
    };

    // Act
    let json = serde_json::to_string(&item).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Assert
    assert_eq!(parsed["status"], "IN_PROGRESS");
}

#[test]
fn should_serialize_status_draft_as_screaming_snake_case() {
    // Arrange
    let item = RetrospectListItem {
        retrospect_id: 1,
        project_name: "테스트".to_string(),
        retrospect_method: "KPT".to_string(),
        retrospect_date: "2026-01-26".to_string(),
        participant_count: 0,
        status: RetrospectListStatus::Draft,
    };

    // Act
    let json = serde_json::to_string(&item).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Assert
    assert_eq!(parsed["status"], "DRAFT");
}

#[test]
fn should_serialize_status_completed_as_screaming_snake_case() {
    // Arrange
    let item = RetrospectListItem {
        retrospect_id: 1,
        project_name: "테스트".to_string(),
        retrospect_method: "KPT".to_string(),
        retrospect_date: "2026-01-26".to_string(),
        participant_count: 0,
        status: RetrospectListStatus::Completed,
    };

    // Act
    let json = serde_json::to_string(&item).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Assert
    assert_eq!(parsed["status"], "COMPLETED");
}

#[test]
fn should_serialize_all_status_values() {
    // Arrange - 모든 상태 테스트
    let statuses = vec![
        (RetrospectListStatus::InProgress, "IN_PROGRESS"),
        (RetrospectListStatus::Draft, "DRAFT"),
        (RetrospectListStatus::Completed, "COMPLETED"),
    ];

    for (status, expected) in statuses {
        let item = RetrospectListItem {
            retrospect_id: 1,
            project_name: "테스트".to_string(),
            retrospect_method: "KPT".to_string(),
            retrospect_date: "2026-01-26".to_string(),
            participant_count: 0,
            status,
        };

        // Act
        let json = serde_json::to_string(&item).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Assert
        assert_eq!(parsed["status"], expected);
    }
}

#[test]
fn should_display_status_as_screaming_snake_case() {
    // Assert
    assert_eq!(RetrospectListStatus::InProgress.to_string(), "IN_PROGRESS");
    assert_eq!(RetrospectListStatus::Draft.to_string(), "DRAFT");
    assert_eq!(RetrospectListStatus::Completed.to_string(), "COMPLETED");
}
