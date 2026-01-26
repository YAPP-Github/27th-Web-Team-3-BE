//! 보관함 조회 API 통합 테스트 (API-019)
//!
//! GET /api/v1/retrospects/storage 엔드포인트에 대한 HTTP 통합 테스트입니다.
//! Mock 기반 테스트로 실제 DB 연결 없이 핸들러 동작을 검증합니다.

use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
    routing::get,
    Router,
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

mod storage_test_helpers {
    use super::*;

    /// API-019 테스트용 라우터 생성 (보관함 조회)
    pub fn create_storage_test_router() -> Router {
        async fn test_handler(
            headers: axum::http::HeaderMap,
            axum::extract::Query(params): axum::extract::Query<
                std::collections::HashMap<String, String>,
            >,
        ) -> Result<axum::Json<Value>, (StatusCode, axum::Json<Value>)> {
            // Authorization 헤더 검증
            let auth = headers.get(header::AUTHORIZATION);
            if auth.is_none() {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    axum::Json(json!({
                        "isSuccess": false,
                        "code": "AUTH4001",
                        "message": "인증 정보가 유효하지 않습니다.",
                        "result": null
                    })),
                ));
            }

            let auth_str = auth.and_then(|v| v.to_str().ok()).unwrap_or("");
            if !auth_str.starts_with("Bearer ") {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    axum::Json(json!({
                        "isSuccess": false,
                        "code": "AUTH4001",
                        "message": "인증 정보가 유효하지 않습니다.",
                        "result": null
                    })),
                ));
            }

            // range 파라미터 검증
            let range = params.get("range").map(|s| s.as_str()).unwrap_or("ALL");
            let valid_ranges = ["ALL", "3_MONTHS", "6_MONTHS", "1_YEAR"];
            if !valid_ranges.contains(&range) {
                return Err((
                    StatusCode::BAD_REQUEST,
                    axum::Json(json!({
                        "isSuccess": false,
                        "code": "COMMON400",
                        "message": "유효하지 않은 기간 필터입니다.",
                        "result": null
                    })),
                ));
            }

            // Mock 데이터 기반 응답
            match range {
                "ALL" => Ok(axum::Json(json!({
                    "isSuccess": true,
                    "code": "COMMON200",
                    "message": "보관함 조회를 성공했습니다.",
                    "result": {
                        "years": [
                            {
                                "yearLabel": "2026년",
                                "retrospects": [
                                    {
                                        "retrospectId": 124,
                                        "displayDate": "2026-01-24",
                                        "title": "API 명세 표준화 프로젝트",
                                        "retroCategory": "KPT",
                                        "memberCount": 5
                                    },
                                    {
                                        "retrospectId": 120,
                                        "displayDate": "2026-01-10",
                                        "title": "디자인 시스템 구축",
                                        "retroCategory": "FOUR_L",
                                        "memberCount": 3
                                    }
                                ]
                            },
                            {
                                "yearLabel": "2025년",
                                "retrospects": [
                                    {
                                        "retrospectId": 98,
                                        "displayDate": "2025-12-15",
                                        "title": "연말 회고",
                                        "retroCategory": "FIVE_F",
                                        "memberCount": 8
                                    }
                                ]
                            }
                        ]
                    }
                }))),
                "3_MONTHS" => Ok(axum::Json(json!({
                    "isSuccess": true,
                    "code": "COMMON200",
                    "message": "보관함 조회를 성공했습니다.",
                    "result": {
                        "years": [
                            {
                                "yearLabel": "2026년",
                                "retrospects": [
                                    {
                                        "retrospectId": 124,
                                        "displayDate": "2026-01-24",
                                        "title": "API 명세 표준화 프로젝트",
                                        "retroCategory": "KPT",
                                        "memberCount": 5
                                    }
                                ]
                            }
                        ]
                    }
                }))),
                _ => Ok(axum::Json(json!({
                    "isSuccess": true,
                    "code": "COMMON200",
                    "message": "보관함 조회를 성공했습니다.",
                    "result": {
                        "years": []
                    }
                }))),
            }
        }

        Router::new().route("/api/v1/retrospects/storage", get(test_handler))
    }

    /// 응답 본문을 JSON으로 파싱
    pub async fn parse_response_body(body: Body) -> Value {
        let bytes = body.collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }
}

// ============================================
// 인증 관련 테스트
// ============================================

/// [API-019] 인증 헤더 없이 요청 시 401 반환 테스트
#[tokio::test]
async fn api019_should_return_401_when_authorization_header_missing() {
    // Arrange
    let app = storage_test_helpers::create_storage_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/storage")
        // Authorization 헤더 없음
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = storage_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "AUTH4001");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("인증 정보가 유효하지 않습니다"));
}

/// [API-019] 잘못된 Authorization 헤더 형식 시 401 반환 테스트
#[tokio::test]
async fn api019_should_return_401_when_authorization_header_format_invalid() {
    // Arrange
    let app = storage_test_helpers::create_storage_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/storage")
        .header(header::AUTHORIZATION, "InvalidFormat token123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = storage_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "AUTH4001");
}

// ============================================
// 기간 필터 검증 테스트
// ============================================

/// [API-019] 유효하지 않은 range 파라미터 시 400 반환 테스트
#[tokio::test]
async fn api019_should_return_400_when_range_filter_is_invalid() {
    // Arrange
    let app = storage_test_helpers::create_storage_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/storage?range=INVALID_RANGE")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = storage_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "COMMON400");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("유효하지 않은 기간 필터"));
}

// ============================================
// 성공 케이스 테스트
// ============================================

/// [API-019] range 없이 요청 시 기본값(ALL) 200 성공 응답 테스트
#[tokio::test]
async fn api019_should_return_200_with_default_range_all() {
    // Arrange
    let app = storage_test_helpers::create_storage_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/storage")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = storage_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], true);
    assert_eq!(body["code"], "COMMON200");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("보관함 조회를 성공했습니다"));

    // result 구조 검증
    let result = &body["result"];
    assert!(result["years"].is_array());

    let years = result["years"].as_array().unwrap();
    assert_eq!(years.len(), 2);

    // 최신 연도 먼저
    assert_eq!(years[0]["yearLabel"], "2026년");
    assert_eq!(years[1]["yearLabel"], "2025년");
}

/// [API-019] 연도별 회고 아이템 필드 검증 테스트
#[tokio::test]
async fn api019_should_return_correct_retrospect_item_fields() {
    // Arrange
    let app = storage_test_helpers::create_storage_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/storage")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = storage_test_helpers::parse_response_body(response.into_body()).await;
    let first_retro = &body["result"]["years"][0]["retrospects"][0];

    // 필수 필드 존재 확인
    assert!(first_retro["retrospectId"].is_number());
    assert!(first_retro["displayDate"].is_string());
    assert!(first_retro["title"].is_string());
    assert!(first_retro["retroCategory"].is_string());
    assert!(first_retro["memberCount"].is_number());

    // 값 검증
    assert_eq!(first_retro["retrospectId"], 124);
    assert_eq!(first_retro["displayDate"], "2026-01-24");
    assert_eq!(first_retro["title"], "API 명세 표준화 프로젝트");
    assert_eq!(first_retro["retroCategory"], "KPT");
    assert_eq!(first_retro["memberCount"], 5);
}

/// [API-019] 연도별 그룹 내 최신순 정렬 검증 테스트
#[tokio::test]
async fn api019_should_sort_retrospects_by_date_descending_within_year() {
    // Arrange
    let app = storage_test_helpers::create_storage_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/storage")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = storage_test_helpers::parse_response_body(response.into_body()).await;
    let retrospects_2026 = body["result"]["years"][0]["retrospects"]
        .as_array()
        .unwrap();

    // 2026년 그룹 내에서 최신순 (01-24 > 01-10)
    assert_eq!(retrospects_2026.len(), 2);
    assert_eq!(retrospects_2026[0]["displayDate"], "2026-01-24");
    assert_eq!(retrospects_2026[1]["displayDate"], "2026-01-10");
}

/// [API-019] range=3_MONTHS 필터 적용 시 필터링된 결과 반환 테스트
#[tokio::test]
async fn api019_should_return_filtered_results_with_3_months_range() {
    // Arrange
    let app = storage_test_helpers::create_storage_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/storage?range=3_MONTHS")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = storage_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], true);
    assert_eq!(body["code"], "COMMON200");

    let years = body["result"]["years"].as_array().unwrap();
    assert_eq!(years.len(), 1);
    assert_eq!(years[0]["yearLabel"], "2026년");
}

/// [API-019] range=6_MONTHS 필터 시 빈 결과 반환 테스트
#[tokio::test]
async fn api019_should_return_empty_years_when_no_retrospects_in_range() {
    // Arrange
    let app = storage_test_helpers::create_storage_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/storage?range=6_MONTHS")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = storage_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], true);
    assert_eq!(body["code"], "COMMON200");

    let years = body["result"]["years"].as_array().unwrap();
    assert_eq!(years.len(), 0);
}

/// [API-019] range=1_YEAR 필터 시 빈 결과 반환 테스트
#[tokio::test]
async fn api019_should_return_empty_years_with_1_year_range() {
    // Arrange
    let app = storage_test_helpers::create_storage_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/storage?range=1_YEAR")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = storage_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], true);
    assert_eq!(body["code"], "COMMON200");
    assert!(body["result"]["years"].is_array());
}

/// [API-019] range=ALL 명시적 전달 시 전체 결과 반환 테스트
#[tokio::test]
async fn api019_should_return_all_results_with_explicit_all_range() {
    // Arrange
    let app = storage_test_helpers::create_storage_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/storage?range=ALL")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = storage_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], true);
    assert_eq!(body["code"], "COMMON200");

    let years = body["result"]["years"].as_array().unwrap();
    assert_eq!(years.len(), 2);
}

/// [API-019] 다양한 retroCategory 값 검증 테스트
#[tokio::test]
async fn api019_should_support_multiple_retro_categories() {
    // Arrange
    let app = storage_test_helpers::create_storage_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/storage")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = storage_test_helpers::parse_response_body(response.into_body()).await;
    let all_retrospects: Vec<&Value> = body["result"]["years"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|y| y["retrospects"].as_array().unwrap())
        .collect();

    // 여러 카테고리가 포함되어 있는지 확인
    let categories: Vec<&str> = all_retrospects
        .iter()
        .map(|r| r["retroCategory"].as_str().unwrap())
        .collect();

    assert!(categories.contains(&"KPT"));
    assert!(categories.contains(&"FOUR_L"));
    assert!(categories.contains(&"FIVE_F"));
}

/// [API-019] 연도별 그룹이 연도 내림차순으로 정렬되는지 검증
#[tokio::test]
async fn api019_should_sort_year_groups_descending() {
    // Arrange
    let app = storage_test_helpers::create_storage_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/storage")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = storage_test_helpers::parse_response_body(response.into_body()).await;
    let years = body["result"]["years"].as_array().unwrap();

    // 연도 내림차순 확인
    assert!(years.len() >= 2);
    let first_year = years[0]["yearLabel"].as_str().unwrap();
    let second_year = years[1]["yearLabel"].as_str().unwrap();
    assert!(first_year > second_year);
}
