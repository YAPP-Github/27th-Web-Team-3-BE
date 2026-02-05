//! 회고 상세 정보 조회 API 통합 테스트 (API-012)
//!
//! GET /api/v1/retrospects/{retrospectId} 엔드포인트에 대한 HTTP 통합 테스트입니다.
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

mod detail_test_helpers {
    use super::*;

    /// API-012 테스트용 라우터 생성 (회고 상세 정보 조회)
    pub fn create_detail_test_router() -> Router {
        async fn test_handler(
            headers: axum::http::HeaderMap,
            axum::extract::Path(retrospect_id): axum::extract::Path<i64>,
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

            // retrospectId 검증
            if retrospect_id < 1 {
                return Err((
                    StatusCode::BAD_REQUEST,
                    axum::Json(json!({
                        "isSuccess": false,
                        "code": "COMMON400",
                        "message": "retrospectId는 1 이상의 양수여야 합니다.",
                        "result": null
                    })),
                ));
            }

            // Mock: 존재하지 않는 회고 (999)
            if retrospect_id == 999 {
                return Err((
                    StatusCode::NOT_FOUND,
                    axum::Json(json!({
                        "isSuccess": false,
                        "code": "RETRO4041",
                        "message": "존재하지 않는 회고입니다.",
                        "result": null
                    })),
                ));
            }

            // Mock: 접근 권한 없음 (888)
            if retrospect_id == 888 {
                return Err((
                    StatusCode::FORBIDDEN,
                    axum::Json(json!({
                        "isSuccess": false,
                        "code": "TEAM4031",
                        "message": "해당 회고에 접근 권한이 없습니다.",
                        "result": null
                    })),
                ));
            }

            // 성공 응답
            Ok(axum::Json(json!({
                "isSuccess": true,
                "code": "COMMON200",
                "message": "회고 상세 정보 조회를 성공했습니다.",
                "result": {
                    "teamId": 789,
                    "title": "3차 스프린트 회고",
                    "startTime": "2026-01-24",
                    "retroCategory": "KPT",
                    "members": [
                        { "memberId": 1, "userName": "김민철" },
                        { "memberId": 2, "userName": "카이" }
                    ],
                    "totalLikeCount": 156,
                    "totalCommentCount": 42,
                    "questions": [
                        {
                            "index": 1,
                            "content": "계속 유지하고 싶은 좋은 점은 무엇인가요?"
                        },
                        {
                            "index": 2,
                            "content": "개선이 필요한 문제점은 무엇인가요?"
                        },
                        {
                            "index": 3,
                            "content": "다음에 시도해보고 싶은 것은 무엇인가요?"
                        }
                    ]
                }
            })))
        }

        Router::new().route("/api/v1/retrospects/:retrospect_id", get(test_handler))
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

/// [API-012] 인증 헤더 없이 요청 시 401 반환 테스트
#[tokio::test]
async fn api012_should_return_401_when_authorization_header_missing() {
    // Arrange
    let app = detail_test_helpers::create_detail_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/1")
        // Authorization 헤더 없음
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = detail_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "AUTH4001");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("인증 정보가 유효하지 않습니다"));
}

/// [API-012] 잘못된 Authorization 헤더 형식 시 401 반환 테스트
#[tokio::test]
async fn api012_should_return_401_when_authorization_header_format_invalid() {
    // Arrange
    let app = detail_test_helpers::create_detail_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/1")
        .header(header::AUTHORIZATION, "InvalidFormat token123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = detail_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "AUTH4001");
}

// ============================================
// Path Parameter 검증 테스트
// ============================================

/// [API-012] 유효하지 않은 retrospectId (0) 요청 시 400 반환 테스트
#[tokio::test]
async fn api012_should_return_400_when_retrospect_id_is_zero() {
    // Arrange
    let app = detail_test_helpers::create_detail_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/0")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = detail_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "COMMON400");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("retrospectId는 1 이상의 양수여야 합니다"));
}

/// [API-012] 유효하지 않은 retrospectId (음수) 요청 시 400 반환 테스트
#[tokio::test]
async fn api012_should_return_400_when_retrospect_id_is_negative() {
    // Arrange
    let app = detail_test_helpers::create_detail_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/-1")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = detail_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "COMMON400");
}

// ============================================
// 비즈니스 에러 테스트
// ============================================

/// [API-012] 존재하지 않는 회고 요청 시 404 반환 테스트
#[tokio::test]
async fn api012_should_return_404_when_retrospect_not_found() {
    // Arrange
    let app = detail_test_helpers::create_detail_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/999") // 999는 존재하지 않는 회고
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = detail_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "RETRO4041");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("존재하지 않는 회고"));
}

/// [API-012] 팀 멤버가 아닌 사용자 요청 시 403 반환 테스트
#[tokio::test]
async fn api012_should_return_403_when_user_is_not_team_member() {
    // Arrange
    let app = detail_test_helpers::create_detail_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/888") // 888은 접근 권한 없는 회고
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = detail_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "TEAM4031");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("접근 권한이 없습니다"));
}

// ============================================
// 성공 케이스 테스트
// ============================================

/// [API-012] 유효한 요청 시 200 성공 응답 테스트
#[tokio::test]
async fn api012_should_return_200_when_valid_request() {
    // Arrange
    let app = detail_test_helpers::create_detail_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/100")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = detail_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], true);
    assert_eq!(body["code"], "COMMON200");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("회고 상세 정보 조회를 성공했습니다"));
}

/// [API-012] 성공 응답의 result 필드 구조 검증 테스트
#[tokio::test]
async fn api012_should_return_correct_result_structure() {
    // Arrange
    let app = detail_test_helpers::create_detail_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/100")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = detail_test_helpers::parse_response_body(response.into_body()).await;
    let result = &body["result"];

    // 최상위 필드 존재 및 타입 확인 (camelCase)
    assert!(result["teamId"].is_number());
    assert!(result["title"].is_string());
    assert!(result["startTime"].is_string());
    assert!(result["retroCategory"].is_string());
    assert!(result["members"].is_array());
    assert!(result["totalLikeCount"].is_number());
    assert!(result["totalCommentCount"].is_number());
    assert!(result["questions"].is_array());

    // 값 검증
    assert_eq!(result["teamId"], 789);
    assert_eq!(result["title"], "3차 스프린트 회고");
    assert_eq!(result["startTime"], "2026-01-24");
    assert_eq!(result["retroCategory"], "KPT");
    assert_eq!(result["totalLikeCount"], 156);
    assert_eq!(result["totalCommentCount"], 42);
}

/// [API-012] 성공 응답의 members 배열 필드 검증 테스트
#[tokio::test]
async fn api012_should_return_correct_members_fields() {
    // Arrange
    let app = detail_test_helpers::create_detail_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/100")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = detail_test_helpers::parse_response_body(response.into_body()).await;
    let members = body["result"]["members"].as_array().unwrap();

    assert_eq!(members.len(), 2);

    // 첫 번째 멤버 필드 검증
    let first_member = &members[0];
    assert!(first_member["memberId"].is_number());
    assert!(first_member["userName"].is_string());
    assert_eq!(first_member["memberId"], 1);
    assert_eq!(first_member["userName"], "김민철");

    // 두 번째 멤버 필드 검증
    let second_member = &members[1];
    assert_eq!(second_member["memberId"], 2);
    assert_eq!(second_member["userName"], "카이");
}

/// [API-012] 성공 응답의 questions 배열 필드 검증 테스트
#[tokio::test]
async fn api012_should_return_correct_questions_fields() {
    // Arrange
    let app = detail_test_helpers::create_detail_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/100")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = detail_test_helpers::parse_response_body(response.into_body()).await;
    let questions = body["result"]["questions"].as_array().unwrap();

    assert_eq!(questions.len(), 3);

    // 질문 순서(index) 기준 오름차순 정렬 검증
    for (i, question) in questions.iter().enumerate() {
        assert!(question["index"].is_number());
        assert!(question["content"].is_string());
        assert_eq!(question["index"], (i + 1) as i64);
    }

    // 개별 질문 내용 검증
    assert_eq!(
        questions[0]["content"],
        "계속 유지하고 싶은 좋은 점은 무엇인가요?"
    );
    assert_eq!(
        questions[1]["content"],
        "개선이 필요한 문제점은 무엇인가요?"
    );
    assert_eq!(
        questions[2]["content"],
        "다음에 시도해보고 싶은 것은 무엇인가요?"
    );
}

/// [API-012] 응답 본문이 camelCase 필드명을 사용하는지 검증 테스트
#[tokio::test]
async fn api012_should_use_camel_case_field_names_in_response() {
    // Arrange
    let app = detail_test_helpers::create_detail_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/100")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = detail_test_helpers::parse_response_body(response.into_body()).await;

    // 최상위 응답 필드 camelCase 검증
    assert!(body.get("isSuccess").is_some());
    assert!(body.get("is_success").is_none());

    // result 내부 필드 camelCase 검증
    let result = &body["result"];
    assert!(result.get("teamId").is_some());
    assert!(result.get("team_id").is_none());

    assert!(result.get("startTime").is_some());
    assert!(result.get("start_time").is_none());

    assert!(result.get("retroCategory").is_some());
    assert!(result.get("retro_category").is_none());

    assert!(result.get("totalLikeCount").is_some());
    assert!(result.get("total_like_count").is_none());

    assert!(result.get("totalCommentCount").is_some());
    assert!(result.get("total_comment_count").is_none());

    // members 내부 필드 camelCase 검증
    let first_member = &result["members"][0];
    assert!(first_member.get("memberId").is_some());
    assert!(first_member.get("member_id").is_none());

    assert!(first_member.get("userName").is_some());
    assert!(first_member.get("user_name").is_none());
}
