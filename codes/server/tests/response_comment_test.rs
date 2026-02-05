//! 회고 답변 댓글 API 통합 테스트
//!
//! 이 테스트 모듈은 회고 답변 댓글 관련 엔드포인트에 대한 HTTP 통합 테스트를 포함합니다.
//! - API-026: GET /api/v1/responses/{responseId}/comments (댓글 목록 조회)
//! - API-027: POST /api/v1/responses/{responseId}/comments (댓글 작성)
//! Mock 기반 테스트로 실제 DB 연결 없이 핸들러 동작을 검증합니다.

use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
    routing::{get, post},
    Router,
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

mod test_helpers {
    use super::*;

    /// 응답 본문을 JSON으로 파싱
    pub async fn parse_response_body(body: Body) -> Value {
        let bytes = body.collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    /// API-026 테스트용 라우터 생성 (댓글 목록 조회)
    pub fn create_list_comments_test_router() -> Router {
        async fn test_handler(
            headers: axum::http::HeaderMap,
            axum::extract::Path(response_id): axum::extract::Path<i64>,
            axum::extract::Query(query): axum::extract::Query<
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

            // responseId 검증
            if response_id < 1 {
                return Err((
                    StatusCode::BAD_REQUEST,
                    axum::Json(json!({
                        "isSuccess": false,
                        "code": "COMMON400",
                        "message": "responseId는 1 이상의 양수여야 합니다.",
                        "result": null
                    })),
                ));
            }

            // cursor 검증
            if let Some(cursor_str) = query.get("cursor") {
                if let Ok(cursor) = cursor_str.parse::<i64>() {
                    if cursor < 1 {
                        return Err((
                            StatusCode::BAD_REQUEST,
                            axum::Json(json!({
                                "isSuccess": false,
                                "code": "COMMON400",
                                "message": "cursor는 1 이상의 양수여야 합니다.",
                                "result": null
                            })),
                        ));
                    }
                }
            }

            // size 검증
            if let Some(size_str) = query.get("size") {
                if let Ok(size) = size_str.parse::<i32>() {
                    if size < 1 || size > 100 {
                        return Err((
                            StatusCode::BAD_REQUEST,
                            axum::Json(json!({
                                "isSuccess": false,
                                "code": "COMMON400",
                                "message": "size는 1~100 범위의 정수여야 합니다.",
                                "result": null
                            })),
                        ));
                    }
                }
            }

            // Mock: 존재하지 않는 답변 (999)
            if response_id == 999 {
                return Err((
                    StatusCode::NOT_FOUND,
                    axum::Json(json!({
                        "isSuccess": false,
                        "code": "RES4041",
                        "message": "존재하지 않는 회고 답변입니다.",
                        "result": null
                    })),
                ));
            }

            // Mock: 팀 멤버가 아님 (888)
            if response_id == 888 {
                return Err((
                    StatusCode::FORBIDDEN,
                    axum::Json(json!({
                        "isSuccess": false,
                        "code": "TEAM4031",
                        "message": "해당 리소스에 접근 권한이 없습니다.",
                        "result": null
                    })),
                ));
            }

            // Mock: 댓글이 없는 답변 (555)
            if response_id == 555 {
                return Ok(axum::Json(json!({
                    "isSuccess": true,
                    "code": "COMMON200",
                    "message": "댓글 조회를 성공했습니다.",
                    "result": {
                        "comments": [],
                        "hasNext": false,
                        "nextCursor": null
                    }
                })));
            }

            // Mock: 다음 페이지가 있는 경우 (cursor 없음)
            let has_cursor = query.contains_key("cursor");
            if !has_cursor {
                return Ok(axum::Json(json!({
                    "isSuccess": true,
                    "code": "COMMON200",
                    "message": "댓글 조회를 성공했습니다.",
                    "result": {
                        "comments": [
                            {
                                "commentId": 789,
                                "memberId": 12,
                                "userName": "김민수",
                                "content": "이 의견에 전적으로 동의합니다! 저도 비슷한 생각을 했어요.",
                                "createdAt": "2026-01-24T16:30:15"
                            },
                            {
                                "commentId": 788,
                                "memberId": 15,
                                "userName": "이영희",
                                "content": "좋은 의견 감사합니다!",
                                "createdAt": "2026-01-24T16:25:10"
                            }
                        ],
                        "hasNext": true,
                        "nextCursor": 787
                    }
                })));
            }

            // Mock: 커서 이후 마지막 페이지
            Ok(axum::Json(json!({
                "isSuccess": true,
                "code": "COMMON200",
                "message": "댓글 조회를 성공했습니다.",
                "result": {
                    "comments": [
                        {
                            "commentId": 787,
                            "memberId": 20,
                            "userName": "박철수",
                            "content": "감사합니다.",
                            "createdAt": "2026-01-24T16:20:00"
                        }
                    ],
                    "hasNext": false,
                    "nextCursor": null
                }
            })))
        }

        Router::new().route("/api/v1/responses/:response_id/comments", get(test_handler))
    }

    /// API-027 테스트용 라우터 생성 (댓글 작성)
    pub fn create_comment_test_router() -> Router {
        async fn test_handler(
            headers: axum::http::HeaderMap,
            axum::extract::Path(response_id): axum::extract::Path<i64>,
            body: Result<axum::Json<Value>, axum::extract::rejection::JsonRejection>,
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

            // responseId 검증
            if response_id < 1 {
                return Err((
                    StatusCode::BAD_REQUEST,
                    axum::Json(json!({
                        "isSuccess": false,
                        "code": "COMMON400",
                        "message": "responseId는 1 이상의 양수여야 합니다.",
                        "result": null
                    })),
                ));
            }

            // Body 파싱 검증
            let body = match body {
                Ok(b) => b,
                Err(e) => {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        axum::Json(json!({
                            "isSuccess": false,
                            "code": "COMMON400",
                            "message": format!("요청 본문의 필수 필드가 누락되었거나 유효하지 않습니다: {}", e),
                            "result": null
                        })),
                    ));
                }
            };

            // content 필드 검증
            let content = body.get("content").and_then(|v| v.as_str());
            if content.is_none() || content.map(|s| s.is_empty()).unwrap_or(true) {
                return Err((
                    StatusCode::BAD_REQUEST,
                    axum::Json(json!({
                        "isSuccess": false,
                        "code": "COMMON400",
                        "message": "요청 본문의 필수 필드가 누락되었거나 유효하지 않습니다.",
                        "result": null
                    })),
                ));
            }

            let content_str = content.unwrap();

            // content 길이 검증 (200자 초과)
            if content_str.chars().count() > 200 {
                return Err((
                    StatusCode::BAD_REQUEST,
                    axum::Json(json!({
                        "isSuccess": false,
                        "code": "RES4001",
                        "message": "댓글은 최대 200자까지만 입력 가능합니다.",
                        "result": null
                    })),
                ));
            }

            // Mock: 존재하지 않는 답변 (999)
            if response_id == 999 {
                return Err((
                    StatusCode::NOT_FOUND,
                    axum::Json(json!({
                        "isSuccess": false,
                        "code": "RES4041",
                        "message": "존재하지 않는 회고 답변입니다.",
                        "result": null
                    })),
                ));
            }

            // Mock: 팀 멤버가 아님 (888)
            if response_id == 888 {
                return Err((
                    StatusCode::FORBIDDEN,
                    axum::Json(json!({
                        "isSuccess": false,
                        "code": "TEAM4031",
                        "message": "댓글 작성 권한이 없습니다.",
                        "result": null
                    })),
                ));
            }

            // 성공 응답
            Ok(axum::Json(json!({
                "isSuccess": true,
                "code": "COMMON200",
                "message": "댓글이 성공적으로 등록되었습니다.",
                "result": {
                    "commentId": 789,
                    "responseId": response_id,
                    "content": content_str,
                    "createdAt": "2026-01-24T15:48:21"
                }
            })))
        }

        Router::new().route(
            "/api/v1/responses/:response_id/comments",
            post(test_handler),
        )
    }
}

// ============================================
// API-026: 회고 답변 댓글 목록 조회 통합 테스트
// ============================================

/// [API-026] 인증 헤더 없이 요청 시 401 반환 테스트
#[tokio::test]
async fn api026_should_return_401_when_authorization_header_missing() {
    // Arrange
    let app = test_helpers::create_list_comments_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/responses/1/comments")
        // Authorization 헤더 없음
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "AUTH4001");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("인증 정보가 유효하지 않습니다"));
}

/// [API-026] 잘못된 Authorization 헤더 형식 시 401 반환 테스트
#[tokio::test]
async fn api026_should_return_401_when_authorization_header_format_invalid() {
    // Arrange
    let app = test_helpers::create_list_comments_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/responses/1/comments")
        .header(header::AUTHORIZATION, "InvalidFormat token123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "AUTH4001");
}

/// [API-026] 유효하지 않은 responseId (0) 요청 시 400 반환 테스트
#[tokio::test]
async fn api026_should_return_400_when_response_id_is_zero() {
    // Arrange
    let app = test_helpers::create_list_comments_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/responses/0/comments")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "COMMON400");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("responseId는 1 이상의 양수여야 합니다"));
}

/// [API-026] 유효하지 않은 responseId (음수) 요청 시 400 반환 테스트
#[tokio::test]
async fn api026_should_return_400_when_response_id_is_negative() {
    // Arrange
    let app = test_helpers::create_list_comments_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/responses/-1/comments")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "COMMON400");
}

/// [API-026] 유효하지 않은 cursor (0) 요청 시 400 반환 테스트
#[tokio::test]
async fn api026_should_return_400_when_cursor_is_zero() {
    // Arrange
    let app = test_helpers::create_list_comments_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/responses/1/comments?cursor=0")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "COMMON400");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("cursor는 1 이상의 양수여야 합니다"));
}

/// [API-026] 유효하지 않은 size (0) 요청 시 400 반환 테스트
#[tokio::test]
async fn api026_should_return_400_when_size_is_zero() {
    // Arrange
    let app = test_helpers::create_list_comments_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/responses/1/comments?size=0")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "COMMON400");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("size는 1~100 범위의 정수여야 합니다"));
}

/// [API-026] 유효하지 않은 size (101) 요청 시 400 반환 테스트
#[tokio::test]
async fn api026_should_return_400_when_size_exceeds_100() {
    // Arrange
    let app = test_helpers::create_list_comments_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/responses/1/comments?size=101")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "COMMON400");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("size는 1~100 범위의 정수여야 합니다"));
}

/// [API-026] 존재하지 않는 답변 요청 시 404 반환 테스트
#[tokio::test]
async fn api026_should_return_404_when_response_not_found() {
    // Arrange
    let app = test_helpers::create_list_comments_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/responses/999/comments") // 999는 존재하지 않는 답변
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "RES4041");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("존재하지 않는 회고 답변"));
}

/// [API-026] 팀 멤버가 아닌 경우 403 반환 테스트
#[tokio::test]
async fn api026_should_return_403_when_not_team_member() {
    // Arrange
    let app = test_helpers::create_list_comments_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/responses/888/comments") // 888은 팀 멤버가 아닌 케이스
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "TEAM4031");
    assert!(body["message"].as_str().unwrap().contains("접근 권한"));
}

/// [API-026] 댓글이 없는 경우 빈 배열 반환 테스트
#[tokio::test]
async fn api026_should_return_200_with_empty_array_when_no_comments() {
    // Arrange
    let app = test_helpers::create_list_comments_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/responses/555/comments") // 555는 댓글이 없는 답변
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], true);
    assert_eq!(body["code"], "COMMON200");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("댓글 조회를 성공했습니다"));

    // 빈 배열 확인
    let result = &body["result"];
    assert!(result["comments"].as_array().unwrap().is_empty());
    assert_eq!(result["hasNext"], false);
    assert!(result["nextCursor"].is_null());
}

/// [API-026] 유효한 요청 시 댓글 목록 반환 테스트 (첫 페이지)
#[tokio::test]
async fn api026_should_return_200_with_comments_list_when_valid_request() {
    // Arrange
    let app = test_helpers::create_list_comments_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/responses/1/comments?size=20")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], true);
    assert_eq!(body["code"], "COMMON200");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("댓글 조회를 성공했습니다"));

    let result = &body["result"];
    let comments = result["comments"].as_array().unwrap();
    assert_eq!(comments.len(), 2);

    // 첫 번째 댓글 확인 (최신순 - commentId 789가 먼저)
    let first = &comments[0];
    assert_eq!(first["commentId"], 789);
    assert_eq!(first["memberId"], 12);
    assert_eq!(first["userName"], "김민수");
    assert!(!first["content"].as_str().unwrap().is_empty());
    assert_eq!(first["createdAt"], "2026-01-24T16:30:15");

    // 두 번째 댓글 확인
    let second = &comments[1];
    assert_eq!(second["commentId"], 788);
    assert_eq!(second["memberId"], 15);
    assert_eq!(second["userName"], "이영희");

    // 페이지네이션 정보 확인
    assert_eq!(result["hasNext"], true);
    assert_eq!(result["nextCursor"], 787);
}

/// [API-026] 커서 기반 다음 페이지 조회 테스트
#[tokio::test]
async fn api026_should_return_200_with_next_page_when_cursor_provided() {
    // Arrange
    let app = test_helpers::create_list_comments_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/responses/1/comments?cursor=788&size=20")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], true);
    assert_eq!(body["code"], "COMMON200");

    let result = &body["result"];
    let comments = result["comments"].as_array().unwrap();
    assert_eq!(comments.len(), 1);

    // 마지막 페이지 확인
    assert_eq!(result["hasNext"], false);
    assert!(result["nextCursor"].is_null());
}

/// [API-026] 기본 size 값 (20) 적용 테스트
#[tokio::test]
async fn api026_should_use_default_size_when_not_provided() {
    // Arrange
    let app = test_helpers::create_list_comments_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/responses/1/comments") // size 파라미터 없음
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], true);
    assert_eq!(body["code"], "COMMON200");
}

// ============================================
// API-027: 회고 답변 댓글 작성 통합 테스트
// ============================================

/// [API-027] 인증 헤더 없이 요청 시 401 반환 테스트
#[tokio::test]
async fn api027_should_return_401_when_authorization_header_missing() {
    // Arrange
    let app = test_helpers::create_comment_test_router();

    let request_body = json!({
        "content": "테스트 댓글입니다."
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/responses/1/comments")
        .header(header::CONTENT_TYPE, "application/json")
        // Authorization 헤더 없음
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "AUTH4001");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("인증 정보가 유효하지 않습니다"));
}

/// [API-027] 잘못된 Authorization 헤더 형식 시 401 반환 테스트
#[tokio::test]
async fn api027_should_return_401_when_authorization_header_format_invalid() {
    // Arrange
    let app = test_helpers::create_comment_test_router();

    let request_body = json!({
        "content": "테스트 댓글입니다."
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/responses/1/comments")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "InvalidFormat token123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "AUTH4001");
}

/// [API-027] 유효하지 않은 responseId (0) 요청 시 400 반환 테스트
#[tokio::test]
async fn api027_should_return_400_when_response_id_is_zero() {
    // Arrange
    let app = test_helpers::create_comment_test_router();

    let request_body = json!({
        "content": "테스트 댓글입니다."
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/responses/0/comments")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "COMMON400");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("responseId는 1 이상의 양수여야 합니다"));
}

/// [API-027] 유효하지 않은 responseId (음수) 요청 시 400 반환 테스트
#[tokio::test]
async fn api027_should_return_400_when_response_id_is_negative() {
    // Arrange
    let app = test_helpers::create_comment_test_router();

    let request_body = json!({
        "content": "테스트 댓글입니다."
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/responses/-1/comments")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "COMMON400");
}

/// [API-027] content 필드 누락 시 400 반환 테스트
#[tokio::test]
async fn api027_should_return_400_when_content_missing() {
    // Arrange
    let app = test_helpers::create_comment_test_router();

    let request_body = json!({}); // content 필드 없음

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/responses/1/comments")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "COMMON400");
}

/// [API-027] content가 빈 문자열일 때 400 반환 테스트
#[tokio::test]
async fn api027_should_return_400_when_content_is_empty() {
    // Arrange
    let app = test_helpers::create_comment_test_router();

    let request_body = json!({
        "content": ""
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/responses/1/comments")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "COMMON400");
}

/// [API-027] content가 200자 초과 시 400 반환 테스트 (RES4001)
#[tokio::test]
async fn api027_should_return_400_when_content_exceeds_200_chars() {
    // Arrange
    let app = test_helpers::create_comment_test_router();

    let long_content = "가".repeat(201); // 201자 - 최대 200자 초과

    let request_body = json!({
        "content": long_content
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/responses/1/comments")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "RES4001");
    assert!(body["message"].as_str().unwrap().contains("200자"));
}

/// [API-027] content가 정확히 200자일 때 성공 테스트
#[tokio::test]
async fn api027_should_return_200_when_content_is_exactly_200_chars() {
    // Arrange
    let app = test_helpers::create_comment_test_router();

    let exact_content = "가".repeat(200); // 정확히 200자

    let request_body = json!({
        "content": exact_content
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/responses/1/comments")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], true);
    assert_eq!(body["code"], "COMMON200");
}

/// [API-027] 존재하지 않는 답변에 댓글 작성 시 404 반환 테스트
#[tokio::test]
async fn api027_should_return_404_when_response_not_found() {
    // Arrange
    let app = test_helpers::create_comment_test_router();

    let request_body = json!({
        "content": "테스트 댓글입니다."
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/responses/999/comments") // 999는 존재하지 않는 답변
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "RES4041");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("존재하지 않는 회고 답변"));
}

/// [API-027] 팀 멤버가 아닌 경우 403 반환 테스트
#[tokio::test]
async fn api027_should_return_403_when_not_team_member() {
    // Arrange
    let app = test_helpers::create_comment_test_router();

    let request_body = json!({
        "content": "테스트 댓글입니다."
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/responses/888/comments") // 888은 팀 멤버가 아닌 케이스
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "TEAM4031");
    assert!(body["message"].as_str().unwrap().contains("권한"));
}

/// [API-027] 유효한 요청 시 댓글 작성 성공 테스트
#[tokio::test]
async fn api027_should_return_200_when_valid_request() {
    // Arrange
    let app = test_helpers::create_comment_test_router();

    let request_body = json!({
        "content": "이 부분 정말 공감되네요! 고생 많으셨습니다."
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/responses/456/comments")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], true);
    assert_eq!(body["code"], "COMMON200");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("성공적으로 등록"));

    // result 검증
    let result = &body["result"];
    assert!(result["commentId"].is_i64());
    assert_eq!(result["responseId"], 456);
    assert_eq!(
        result["content"],
        "이 부분 정말 공감되네요! 고생 많으셨습니다."
    );
    assert!(!result["createdAt"].as_str().unwrap().is_empty());
}

/// [API-027] 유효하지 않은 JSON 요청 바디 시 400 반환 테스트
#[tokio::test]
async fn api027_should_return_400_when_request_body_is_invalid_json() {
    // Arrange
    let app = test_helpers::create_comment_test_router();
    let invalid_json = "{ invalid json }";

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/responses/1/comments")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::from(invalid_json))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "COMMON400");
}

/// [API-027] 빈 요청 바디 시 400 반환 테스트
#[tokio::test]
async fn api027_should_return_400_when_request_body_is_empty() {
    // Arrange
    let app = test_helpers::create_comment_test_router();

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/responses/1/comments")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
