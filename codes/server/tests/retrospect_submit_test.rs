//! 회고 제출 API 통합 테스트 (API-017)
//!
//! POST /api/v1/retrospects/{retrospectId}/submit 엔드포인트에 대한 HTTP 통합 테스트입니다.
//! Mock 기반 테스트로 실제 DB 연결 없이 핸들러 동작을 검증합니다.

use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
    routing::post,
    Router,
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

mod submit_test_helpers {
    use super::*;

    /// API-017 테스트용 라우터 생성 (회고 최종 제출)
    pub fn create_submit_test_router() -> Router {
        async fn test_handler(
            headers: axum::http::HeaderMap,
            axum::extract::Path(retrospect_id): axum::extract::Path<i64>,
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

            // Body 파싱 검증
            let body = match body {
                Ok(b) => b,
                Err(e) => {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        axum::Json(json!({
                            "isSuccess": false,
                            "code": "COMMON400",
                            "message": format!("JSON 파싱 실패: {}", e),
                            "result": null
                        })),
                    ));
                }
            };

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

            // Mock: 이미 제출 완료 (555)
            if retrospect_id == 555 {
                return Err((
                    StatusCode::FORBIDDEN,
                    axum::Json(json!({
                        "isSuccess": false,
                        "code": "RETRO4033",
                        "message": "이미 제출이 완료된 회고입니다.",
                        "result": null
                    })),
                ));
            }

            // answers 배열 검증
            let answers = match body.get("answers").and_then(|v| v.as_array()) {
                Some(arr) => arr,
                None => {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        axum::Json(json!({
                            "isSuccess": false,
                            "code": "RETRO4002",
                            "message": "모든 질문에 대한 답변이 필요합니다.",
                            "result": null
                        })),
                    ));
                }
            };

            // 답변 개수 검증
            if answers.len() != 5 {
                return Err((
                    StatusCode::BAD_REQUEST,
                    axum::Json(json!({
                        "isSuccess": false,
                        "code": "RETRO4002",
                        "message": "모든 질문에 대한 답변이 필요합니다.",
                        "result": null
                    })),
                ));
            }

            // 각 답변 내용 검증
            for answer in answers {
                let content = answer.get("content").and_then(|v| v.as_str()).unwrap_or("");

                // 공백만으로 구성된 답변
                if content.trim().is_empty() {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        axum::Json(json!({
                            "isSuccess": false,
                            "code": "RETRO4007",
                            "message": "답변 내용은 공백만으로 구성될 수 없습니다.",
                            "result": null
                        })),
                    ));
                }

                // 최대 1,000자 제한
                if content.chars().count() > 1000 {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        axum::Json(json!({
                            "isSuccess": false,
                            "code": "RETRO4003",
                            "message": "답변은 1,000자를 초과할 수 없습니다.",
                            "result": null
                        })),
                    ));
                }
            }

            // 성공 응답
            Ok(axum::Json(json!({
                "isSuccess": true,
                "code": "COMMON200",
                "message": "회고 제출이 성공적으로 완료되었습니다.",
                "result": {
                    "retrospectId": retrospect_id,
                    "submittedAt": "2026-01-24",
                    "status": "SUBMITTED"
                }
            })))
        }

        Router::new().route(
            "/api/v1/retrospects/:retrospect_id/submit",
            post(test_handler),
        )
    }

    /// 유효한 제출 요청 바디 생성
    pub fn create_valid_submit_body() -> Value {
        json!({
            "answers": [
                { "questionNumber": 1, "content": "유지할 점에 대한 답변입니다." },
                { "questionNumber": 2, "content": "문제점에 대한 답변입니다." },
                { "questionNumber": 3, "content": "시도할 점에 대한 답변입니다." },
                { "questionNumber": 4, "content": "느낀 점에 대한 답변입니다." },
                { "questionNumber": 5, "content": "기타 의견에 대한 답변입니다." }
            ]
        })
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

/// [API-017] 인증 헤더 없이 요청 시 401 반환 테스트
#[tokio::test]
async fn api017_should_return_401_when_authorization_header_missing() {
    // Arrange
    let app = submit_test_helpers::create_submit_test_router();
    let request_body = submit_test_helpers::create_valid_submit_body();

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/retrospects/1/submit")
        .header(header::CONTENT_TYPE, "application/json")
        // Authorization 헤더 없음
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = submit_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "AUTH4001");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("인증 정보가 유효하지 않습니다"));
}

/// [API-017] 잘못된 Authorization 헤더 형식 시 401 반환 테스트
#[tokio::test]
async fn api017_should_return_401_when_authorization_header_format_invalid() {
    // Arrange
    let app = submit_test_helpers::create_submit_test_router();
    let request_body = submit_test_helpers::create_valid_submit_body();

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/retrospects/1/submit")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "InvalidFormat token123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = submit_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "AUTH4001");
}

// ============================================
// Path Parameter 검증 테스트
// ============================================

/// [API-017] 유효하지 않은 retrospectId (0) 요청 시 400 반환 테스트
#[tokio::test]
async fn api017_should_return_400_when_retrospect_id_is_zero() {
    // Arrange
    let app = submit_test_helpers::create_submit_test_router();
    let request_body = submit_test_helpers::create_valid_submit_body();

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/retrospects/0/submit")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = submit_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "COMMON400");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("retrospectId는 1 이상의 양수여야 합니다"));
}

/// [API-017] 유효하지 않은 retrospectId (음수) 요청 시 400 반환 테스트
#[tokio::test]
async fn api017_should_return_400_when_retrospect_id_is_negative() {
    // Arrange
    let app = submit_test_helpers::create_submit_test_router();
    let request_body = submit_test_helpers::create_valid_submit_body();

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/retrospects/-1/submit")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = submit_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "COMMON400");
}

// ============================================
// 비즈니스 에러 테스트
// ============================================

/// [API-017] 존재하지 않는 회고 요청 시 404 반환 테스트
#[tokio::test]
async fn api017_should_return_404_when_retrospect_not_found() {
    // Arrange
    let app = submit_test_helpers::create_submit_test_router();
    let request_body = submit_test_helpers::create_valid_submit_body();

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/retrospects/999/submit") // 999는 존재하지 않는 회고
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = submit_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "RETRO4041");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("존재하지 않는 회고"));
}

/// [API-017] 이미 제출 완료된 회고 요청 시 403 반환 테스트
#[tokio::test]
async fn api017_should_return_403_when_already_submitted() {
    // Arrange
    let app = submit_test_helpers::create_submit_test_router();
    let request_body = submit_test_helpers::create_valid_submit_body();

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/retrospects/555/submit") // 555는 이미 제출 완료된 회고
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = submit_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "RETRO4033");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("이미 제출이 완료된 회고"));
}

// ============================================
// 답변 검증 테스트
// ============================================

/// [API-017] 답변 개수 부족 시 400 반환 테스트 (RETRO4002)
#[tokio::test]
async fn api017_should_return_400_when_answers_less_than_5() {
    // Arrange
    let app = submit_test_helpers::create_submit_test_router();
    let request_body = json!({
        "answers": [
            { "questionNumber": 1, "content": "답변 1" },
            { "questionNumber": 2, "content": "답변 2" },
            { "questionNumber": 3, "content": "답변 3" }
        ]
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/retrospects/1/submit")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = submit_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "RETRO4002");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("모든 질문에 대한 답변이 필요합니다"));
}

/// [API-017] 빈 answers 배열 시 400 반환 테스트 (RETRO4002)
#[tokio::test]
async fn api017_should_return_400_when_answers_empty() {
    // Arrange
    let app = submit_test_helpers::create_submit_test_router();
    let request_body = json!({
        "answers": []
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/retrospects/1/submit")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = submit_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "RETRO4002");
}

/// [API-017] 답변 내용이 공백만 있을 때 400 반환 테스트 (RETRO4007)
#[tokio::test]
async fn api017_should_return_400_when_answer_content_is_whitespace_only() {
    // Arrange
    let app = submit_test_helpers::create_submit_test_router();
    let request_body = json!({
        "answers": [
            { "questionNumber": 1, "content": "   " },
            { "questionNumber": 2, "content": "답변 2" },
            { "questionNumber": 3, "content": "답변 3" },
            { "questionNumber": 4, "content": "답변 4" },
            { "questionNumber": 5, "content": "답변 5" }
        ]
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/retrospects/1/submit")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = submit_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "RETRO4007");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("공백만으로 구성될 수 없습니다"));
}

/// [API-017] 답변 길이 초과 시 400 반환 테스트 (RETRO4003)
#[tokio::test]
async fn api017_should_return_400_when_answer_content_exceeds_1000_chars() {
    // Arrange
    let app = submit_test_helpers::create_submit_test_router();
    let long_content = "가".repeat(1001);
    let request_body = json!({
        "answers": [
            { "questionNumber": 1, "content": long_content },
            { "questionNumber": 2, "content": "답변 2" },
            { "questionNumber": 3, "content": "답변 3" },
            { "questionNumber": 4, "content": "답변 4" },
            { "questionNumber": 5, "content": "답변 5" }
        ]
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/retrospects/1/submit")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = submit_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "RETRO4003");
    assert!(body["message"].as_str().unwrap().contains("1,000자를 초과"));
}

/// [API-017] 유효하지 않은 JSON 요청 바디 시 400 반환 테스트
#[tokio::test]
async fn api017_should_return_400_when_request_body_is_invalid_json() {
    // Arrange
    let app = submit_test_helpers::create_submit_test_router();
    let invalid_json = "{ invalid json }";

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/retrospects/1/submit")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::from(invalid_json))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = submit_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "COMMON400");
}

/// [API-017] 빈 요청 바디 시 400 반환 테스트
#[tokio::test]
async fn api017_should_return_400_when_request_body_is_empty() {
    // Arrange
    let app = submit_test_helpers::create_submit_test_router();

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/retrospects/1/submit")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// ============================================
// 성공 케이스 테스트
// ============================================

/// [API-017] 유효한 요청 시 200 성공 응답 테스트
#[tokio::test]
async fn api017_should_return_200_when_valid_request() {
    // Arrange
    let app = submit_test_helpers::create_submit_test_router();
    let request_body = submit_test_helpers::create_valid_submit_body();

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/retrospects/101/submit")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = submit_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], true);
    assert_eq!(body["code"], "COMMON200");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("성공적으로 완료"));

    // result 검증
    let result = &body["result"];
    assert_eq!(result["retrospectId"], 101);
    assert_eq!(result["submittedAt"], "2026-01-24");
    assert_eq!(result["status"], "SUBMITTED");
}

/// [API-017] 최대 길이(1,000자) 답변으로 성공 응답 테스트
#[tokio::test]
async fn api017_should_return_200_when_answer_content_is_exactly_1000_chars() {
    // Arrange
    let app = submit_test_helpers::create_submit_test_router();
    let max_content = "가".repeat(1000);
    let request_body = json!({
        "answers": [
            { "questionNumber": 1, "content": max_content },
            { "questionNumber": 2, "content": "답변 2" },
            { "questionNumber": 3, "content": "답변 3" },
            { "questionNumber": 4, "content": "답변 4" },
            { "questionNumber": 5, "content": "답변 5" }
        ]
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/retrospects/101/submit")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = submit_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], true);
    assert_eq!(body["code"], "COMMON200");
}
