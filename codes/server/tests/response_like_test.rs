//! API-025: 회고 답변 좋아요 토글 테스트
//!
//! 테스트 대상:
//! - POST /api/v1/responses/{responseId}/like
//! - 응답 필드 및 에러 응답 검증

use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
    routing::post,
    Router,
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

mod like_test_helpers {
    use super::*;

    /// API-025 테스트용 라우터 생성 (좋아요 토글)
    pub fn create_like_test_router() -> Router {
        async fn test_handler(
            headers: axum::http::HeaderMap,
            axum::extract::Path(response_id): axum::extract::Path<i64>,
        ) -> Result<axum::Json<Value>, (StatusCode, axum::Json<Value>)> {
            // Authorization 헤더 검증
            let auth = headers.get(header::AUTHORIZATION);
            if auth.is_none() {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    axum::Json(json!({
                        "isSuccess": false,
                        "code": "AUTH4001",
                        "message": "로그인이 필요합니다.",
                        "result": null
                    })),
                ));
            }

            let auth_str = auth.unwrap().to_str().unwrap_or("");
            if !auth_str.starts_with("Bearer ") {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    axum::Json(json!({
                        "isSuccess": false,
                        "code": "AUTH4001",
                        "message": "토큰 형식이 올바르지 않습니다.",
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

            // Mock: 존재하지 않는 답변 (999999)
            if response_id == 999999 {
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

            // Mock: 권한 없음 (888888)
            if response_id == 888888 {
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

            // 정상 응답 (토글)
            Ok(axum::Json(json!({
                "isSuccess": true,
                "code": "COMMON200",
                "message": "좋아요 상태가 성공적으로 업데이트되었습니다.",
                "result": {
                    "responseId": response_id,
                    "isLiked": true,
                    "totalLikes": 1
                }
            })))
        }

        Router::new().route("/api/v1/responses/:response_id/like", post(test_handler))
    }
}

// ============== 인증 테스트 ==============

#[tokio::test]
async fn api025_should_return_401_when_authorization_header_missing() {
    // Arrange
    let app = like_test_helpers::create_like_test_router();
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/responses/1/like")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["isSuccess"], false);
    assert_eq!(json["code"], "AUTH4001");
}

#[tokio::test]
async fn api025_should_return_401_when_token_format_invalid() {
    // Arrange
    let app = like_test_helpers::create_like_test_router();
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/responses/1/like")
        .header(header::AUTHORIZATION, "InvalidToken")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ============== 입력 검증 테스트 ==============

#[tokio::test]
async fn api025_should_return_400_when_response_id_is_zero() {
    // Arrange
    let app = like_test_helpers::create_like_test_router();
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/responses/0/like")
        .header(header::AUTHORIZATION, "Bearer valid_token")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["isSuccess"], false);
    assert_eq!(json["code"], "COMMON400");
}

#[tokio::test]
async fn api025_should_return_400_when_response_id_is_negative() {
    // Arrange
    let app = like_test_helpers::create_like_test_router();
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/responses/-1/like")
        .header(header::AUTHORIZATION, "Bearer valid_token")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// ============== 리소스 없음 테스트 (404) ==============

#[tokio::test]
async fn api025_should_return_404_when_response_not_found() {
    // Arrange
    let app = like_test_helpers::create_like_test_router();
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/responses/999999/like")
        .header(header::AUTHORIZATION, "Bearer valid_token")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["isSuccess"], false);
    assert_eq!(json["code"], "RES4041");
    assert!(json["message"].as_str().unwrap().contains("존재하지 않는"));
}

// ============== 권한 없음 테스트 (403) ==============

#[tokio::test]
async fn api025_should_return_403_when_not_team_member() {
    // Arrange
    let app = like_test_helpers::create_like_test_router();
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/responses/888888/like")
        .header(header::AUTHORIZATION, "Bearer valid_token")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["isSuccess"], false);
    assert_eq!(json["code"], "TEAM4031");
    assert!(json["message"].as_str().unwrap().contains("접근 권한"));
}

// ============== 성공 응답 테스트 ==============

#[tokio::test]
async fn api025_should_return_200_on_successful_like_toggle() {
    // Arrange
    let app = like_test_helpers::create_like_test_router();
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/responses/123/like")
        .header(header::AUTHORIZATION, "Bearer valid_token")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["isSuccess"], true);
    assert_eq!(json["code"], "COMMON200");
    assert!(json.get("result").is_some());
}

#[tokio::test]
async fn api025_should_return_correct_response_fields() {
    // Arrange
    let app = like_test_helpers::create_like_test_router();
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/responses/456/like")
        .header(header::AUTHORIZATION, "Bearer valid_token")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // camelCase 필드 확인
    let result = &json["result"];
    assert!(result.get("responseId").is_some());
    assert!(result.get("isLiked").is_some());
    assert!(result.get("totalLikes").is_some());

    // snake_case가 없어야 함
    assert!(result.get("response_id").is_none());
    assert!(result.get("is_liked").is_none());
    assert!(result.get("total_likes").is_none());
}

#[tokio::test]
async fn api025_should_return_response_id_in_result() {
    // Arrange
    let app = like_test_helpers::create_like_test_router();
    let response_id = 789;
    let request = Request::builder()
        .method(Method::POST)
        .uri(format!("/api/v1/responses/{}/like", response_id))
        .header(header::AUTHORIZATION, "Bearer valid_token")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["result"]["responseId"], response_id);
}
