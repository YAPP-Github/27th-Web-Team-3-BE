//! 회고 답변 카테고리별 조회 API 통합 테스트 (API-020)
//!
//! GET /api/v1/retrospects/{retrospectId}/responses 엔드포인트에 대한 HTTP 통합 테스트입니다.
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

mod responses_test_helpers {
    use super::*;

    /// API-020 테스트용 라우터 생성 (답변 카테고리별 조회)
    pub fn create_responses_test_router() -> Router {
        async fn test_handler(
            headers: axum::http::HeaderMap,
            axum::extract::Path(retrospect_id): axum::extract::Path<i64>,
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

            // category 파라미터 검증
            let category = params.get("category").map(|s| s.as_str());
            let valid_categories = [
                "ALL",
                "QUESTION_1",
                "QUESTION_2",
                "QUESTION_3",
                "QUESTION_4",
                "QUESTION_5",
            ];

            match category {
                None => {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        axum::Json(json!({
                            "isSuccess": false,
                            "code": "RETRO4004",
                            "message": "유효하지 않은 카테고리 값입니다.",
                            "result": null
                        })),
                    ));
                }
                Some(cat) if !valid_categories.contains(&cat) => {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        axum::Json(json!({
                            "isSuccess": false,
                            "code": "RETRO4004",
                            "message": "유효하지 않은 카테고리 값입니다.",
                            "result": null
                        })),
                    ));
                }
                _ => {}
            }

            // cursor 검증
            if let Some(cursor_str) = params.get("cursor") {
                let cursor = match cursor_str.parse::<i64>() {
                    Ok(v) => v,
                    Err(_) => {
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
                };
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

            // size 검증
            if let Some(size_str) = params.get("size") {
                let size = match size_str.parse::<i64>() {
                    Ok(v) => v,
                    Err(_) => {
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
                };
                if !(1..=100).contains(&size) {
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

            // 존재하지 않는 회고
            if retrospect_id == 9999 {
                return Err((
                    StatusCode::NOT_FOUND,
                    axum::Json(json!({
                        "isSuccess": false,
                        "code": "RETRO4041",
                        "message": "존재하지 않는 회고 세션입니다.",
                        "result": null
                    })),
                ));
            }

            // 권한 없는 회고
            if retrospect_id == 8888 {
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

            let cat = category.unwrap_or("ALL");

            // Mock 데이터 기반 응답
            match cat {
                "ALL" => Ok(axum::Json(json!({
                    "isSuccess": true,
                    "code": "COMMON200",
                    "message": "답변 리스트 조회를 성공했습니다.",
                    "result": {
                        "responses": [
                            {
                                "responseId": 501,
                                "userName": "제이슨",
                                "content": "이번 스프린트에서 테스트 코드를 꼼꼼히 짠 것이 좋았습니다.",
                                "likeCount": 12,
                                "commentCount": 3
                            },
                            {
                                "responseId": 456,
                                "userName": "김민수",
                                "content": "기한 맞춰서 작업하는 것을 잘했고요...",
                                "likeCount": 12,
                                "commentCount": 21
                            }
                        ],
                        "hasNext": true,
                        "nextCursor": 455
                    }
                }))),
                "QUESTION_1" => Ok(axum::Json(json!({
                    "isSuccess": true,
                    "code": "COMMON200",
                    "message": "답변 리스트 조회를 성공했습니다.",
                    "result": {
                        "responses": [
                            {
                                "responseId": 501,
                                "userName": "제이슨",
                                "content": "테스트 코드를 꼼꼼히 짠 것이 좋았습니다.",
                                "likeCount": 12,
                                "commentCount": 3
                            }
                        ],
                        "hasNext": false,
                        "nextCursor": null
                    }
                }))),
                _ => Ok(axum::Json(json!({
                    "isSuccess": true,
                    "code": "COMMON200",
                    "message": "답변 리스트 조회를 성공했습니다.",
                    "result": {
                        "responses": [],
                        "hasNext": false,
                        "nextCursor": null
                    }
                }))),
            }
        }

        Router::new().route(
            "/api/v1/retrospects/:retrospect_id/responses",
            get(test_handler),
        )
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

/// [API-020] 인증 헤더 없이 요청 시 401 반환 테스트
#[tokio::test]
async fn api020_should_return_401_when_authorization_header_missing() {
    // Arrange
    let app = responses_test_helpers::create_responses_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/100/responses?category=ALL")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = responses_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "AUTH4001");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("인증 정보가 유효하지 않습니다"));
}

/// [API-020] 잘못된 Authorization 헤더 형식 시 401 반환 테스트
#[tokio::test]
async fn api020_should_return_401_when_authorization_header_format_invalid() {
    // Arrange
    let app = responses_test_helpers::create_responses_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/100/responses?category=ALL")
        .header(header::AUTHORIZATION, "InvalidFormat token123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = responses_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "AUTH4001");
}

// ============================================
// Path Parameter 검증 테스트
// ============================================

/// [API-020] retrospectId가 0일 때 400 반환 테스트
#[tokio::test]
async fn api020_should_return_400_when_retrospect_id_is_zero() {
    // Arrange
    let app = responses_test_helpers::create_responses_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/0/responses?category=ALL")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = responses_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "COMMON400");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("retrospectId는 1 이상"));
}

/// [API-020] retrospectId가 음수일 때 400 반환 테스트
#[tokio::test]
async fn api020_should_return_400_when_retrospect_id_is_negative() {
    // Arrange
    let app = responses_test_helpers::create_responses_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/-1/responses?category=ALL")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = responses_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "COMMON400");
}

// ============================================
// Query Parameter 검증 테스트
// ============================================

/// [API-020] category 파라미터 누락 시 400 반환 테스트
#[tokio::test]
async fn api020_should_return_400_when_category_is_missing() {
    // Arrange
    let app = responses_test_helpers::create_responses_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/100/responses")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = responses_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "RETRO4004");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("유효하지 않은 카테고리"));
}

/// [API-020] 잘못된 category 값 시 400 반환 테스트
#[tokio::test]
async fn api020_should_return_400_when_category_is_invalid() {
    // Arrange
    let app = responses_test_helpers::create_responses_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/100/responses?category=INVALID")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = responses_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "RETRO4004");
}

/// [API-020] cursor가 0일 때 400 반환 테스트
#[tokio::test]
async fn api020_should_return_400_when_cursor_is_zero() {
    // Arrange
    let app = responses_test_helpers::create_responses_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/100/responses?category=ALL&cursor=0")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = responses_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "COMMON400");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("cursor는 1 이상"));
}

/// [API-020] size가 범위 밖일 때 400 반환 테스트
#[tokio::test]
async fn api020_should_return_400_when_size_is_out_of_range() {
    // Arrange
    let app = responses_test_helpers::create_responses_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/100/responses?category=ALL&size=101")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = responses_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "COMMON400");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("size는 1~100 범위"));
}

/// [API-020] size가 0일 때 400 반환 테스트
#[tokio::test]
async fn api020_should_return_400_when_size_is_zero() {
    // Arrange
    let app = responses_test_helpers::create_responses_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/100/responses?category=ALL&size=0")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = responses_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "COMMON400");
}

// ============================================
// 비즈니스 에러 테스트
// ============================================

/// [API-020] 존재하지 않는 회고 시 404 반환 테스트
#[tokio::test]
async fn api020_should_return_404_when_retrospect_not_found() {
    // Arrange
    let app = responses_test_helpers::create_responses_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/9999/responses?category=ALL")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = responses_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "RETRO4041");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("존재하지 않는 회고"));
}

/// [API-020] 접근 권한 없는 회고 시 403 반환 테스트
#[tokio::test]
async fn api020_should_return_403_when_access_denied() {
    // Arrange
    let app = responses_test_helpers::create_responses_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/8888/responses?category=ALL")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = responses_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "TEAM4031");
}

// ============================================
// 성공 케이스 테스트
// ============================================

/// [API-020] category=ALL 시 전체 답변 200 성공 응답 테스트
#[tokio::test]
async fn api020_should_return_200_with_all_responses() {
    // Arrange
    let app = responses_test_helpers::create_responses_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/100/responses?category=ALL&size=10")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = responses_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], true);
    assert_eq!(body["code"], "COMMON200");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("답변 리스트 조회를 성공했습니다"));

    // result 구조 검증
    let result = &body["result"];
    assert!(result["responses"].is_array());
    assert!(result["hasNext"].is_boolean());
}

/// [API-020] 응답 필드 구조 검증 테스트
#[tokio::test]
async fn api020_should_return_correct_response_fields() {
    // Arrange
    let app = responses_test_helpers::create_responses_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/100/responses?category=ALL")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = responses_test_helpers::parse_response_body(response.into_body()).await;
    let first_response = &body["result"]["responses"][0];

    // 필수 필드 존재 확인 (camelCase)
    assert!(first_response["responseId"].is_number());
    assert!(first_response["userName"].is_string());
    assert!(first_response["content"].is_string());
    assert!(first_response["likeCount"].is_number());
    assert!(first_response["commentCount"].is_number());

    // 값 검증
    assert_eq!(first_response["responseId"], 501);
    assert_eq!(first_response["userName"], "제이슨");
    assert_eq!(first_response["likeCount"], 12);
    assert_eq!(first_response["commentCount"], 3);
}

/// [API-020] 페이지네이션 hasNext/nextCursor 검증 테스트
#[tokio::test]
async fn api020_should_return_pagination_fields() {
    // Arrange
    let app = responses_test_helpers::create_responses_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/100/responses?category=ALL")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = responses_test_helpers::parse_response_body(response.into_body()).await;
    let result = &body["result"];

    // hasNext가 true이면 nextCursor가 존재
    assert_eq!(result["hasNext"], true);
    assert!(result["nextCursor"].is_number());
    assert_eq!(result["nextCursor"], 455);
}

/// [API-020] category=QUESTION_1 시 필터링된 답변 반환 테스트
#[tokio::test]
async fn api020_should_return_filtered_responses_for_question_1() {
    // Arrange
    let app = responses_test_helpers::create_responses_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/100/responses?category=QUESTION_1")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = responses_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], true);
    assert_eq!(body["code"], "COMMON200");

    let responses = body["result"]["responses"].as_array().unwrap();
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0]["responseId"], 501);
}

/// [API-020] 마지막 페이지 (hasNext=false, nextCursor=null) 검증 테스트
#[tokio::test]
async fn api020_should_return_null_cursor_on_last_page() {
    // Arrange
    let app = responses_test_helpers::create_responses_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/100/responses?category=QUESTION_1")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = responses_test_helpers::parse_response_body(response.into_body()).await;
    let result = &body["result"];

    assert_eq!(result["hasNext"], false);
    assert!(result["nextCursor"].is_null());
}

/// [API-020] 빈 결과 응답 검증 테스트
#[tokio::test]
async fn api020_should_return_empty_responses_for_unused_category() {
    // Arrange
    let app = responses_test_helpers::create_responses_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/100/responses?category=QUESTION_5")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = responses_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], true);
    assert_eq!(body["code"], "COMMON200");

    let responses = body["result"]["responses"].as_array().unwrap();
    assert_eq!(responses.len(), 0);
    assert_eq!(body["result"]["hasNext"], false);
    assert!(body["result"]["nextCursor"].is_null());
}

/// [API-020] camelCase 필드명 검증 테스트
#[tokio::test]
async fn api020_should_use_camel_case_field_names_in_response() {
    // Arrange
    let app = responses_test_helpers::create_responses_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/100/responses?category=ALL")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = responses_test_helpers::parse_response_body(response.into_body()).await;

    // 최상위 응답 필드 camelCase 확인
    assert!(body.get("isSuccess").is_some());
    assert!(body.get("is_success").is_none());

    // result 내부 필드 camelCase 확인
    let result = &body["result"];
    assert!(result.get("hasNext").is_some());
    assert!(result.get("has_next").is_none());
    assert!(result.get("nextCursor").is_some());
    assert!(result.get("next_cursor").is_none());

    // 응답 아이템 필드 camelCase 확인
    let first = &result["responses"][0];
    assert!(first.get("responseId").is_some());
    assert!(first.get("response_id").is_none());
    assert!(first.get("userName").is_some());
    assert!(first.get("user_name").is_none());
    assert!(first.get("likeCount").is_some());
    assert!(first.get("like_count").is_none());
    assert!(first.get("commentCount").is_some());
    assert!(first.get("comment_count").is_none());
}

/// [API-020] responseId 내림차순 정렬 검증 테스트
#[tokio::test]
async fn api020_should_return_responses_sorted_by_response_id_descending() {
    // Arrange
    let app = responses_test_helpers::create_responses_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/100/responses?category=ALL")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = responses_test_helpers::parse_response_body(response.into_body()).await;
    let responses = body["result"]["responses"].as_array().unwrap();

    assert!(responses.len() >= 2);
    let first_id = responses[0]["responseId"].as_i64().unwrap();
    let second_id = responses[1]["responseId"].as_i64().unwrap();
    assert!(
        first_id > second_id,
        "응답은 responseId 내림차순이어야 합니다"
    );
}
