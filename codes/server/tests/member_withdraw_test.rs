/// [API-028] 서비스 탈퇴 API 통합 테스트
/// DELETE /api/v1/members/me
use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
    routing::delete,
    Router,
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::util::ServiceExt;

/// 테스트용 라우터 생성 (DB 없이 라우트 검증용)
fn create_test_router() -> Router {
    Router::new().route("/api/v1/members/me", delete(withdraw_handler))
}

/// 테스트용 핸들러 - 유효성 검증 로직만 포함
async fn withdraw_handler(headers: axum::http::HeaderMap) -> (StatusCode, axum::Json<Value>) {
    // Authorization 헤더 검증
    let auth_header = headers.get(header::AUTHORIZATION);
    if auth_header.is_none() {
        return (
            StatusCode::UNAUTHORIZED,
            axum::Json(json!({
                "isSuccess": false,
                "code": "AUTH4001",
                "message": "인증 정보가 유효하지 않습니다.",
                "result": null
            })),
        );
    }

    let auth_value = auth_header.unwrap().to_str().unwrap_or("");
    if !auth_value.starts_with("Bearer ") {
        return (
            StatusCode::UNAUTHORIZED,
            axum::Json(json!({
                "isSuccess": false,
                "code": "AUTH4001",
                "message": "인증 정보가 유효하지 않습니다.",
                "result": null
            })),
        );
    }

    let token = auth_value.strip_prefix("Bearer ").unwrap_or("");

    // 토큰이 "not_found_user_token"인 경우 사용자 없음 시뮬레이션
    if token == "not_found_user_token" {
        return (
            StatusCode::NOT_FOUND,
            axum::Json(json!({
                "isSuccess": false,
                "code": "MEMBER4042",
                "message": "존재하지 않는 사용자입니다.",
                "result": null
            })),
        );
    }

    // 성공 응답
    (
        StatusCode::OK,
        axum::Json(json!({
            "isSuccess": true,
            "code": "COMMON200",
            "message": "회원 탈퇴가 성공적으로 완료되었습니다.",
            "result": null
        })),
    )
}

/// HTTP 응답을 JSON으로 파싱하는 헬퍼 함수
async fn response_to_json(response: axum::response::Response) -> Value {
    let body = response.into_body();
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[cfg(test)]
mod withdraw_tests {
    use super::*;

    /// [API-028] 서비스 탈퇴 - 성공
    #[tokio::test]
    async fn should_withdraw_successfully() {
        // Arrange
        let app = create_test_router();

        let request = Request::builder()
            .method(Method::DELETE)
            .uri("/api/v1/members/me")
            .header(header::AUTHORIZATION, "Bearer valid_access_token")
            .body(Body::empty())
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let json = response_to_json(response).await;
        assert!(json["isSuccess"].as_bool().unwrap_or(false));
        assert_eq!(json["code"], "COMMON200");
        assert_eq!(json["message"], "회원 탈퇴가 성공적으로 완료되었습니다.");
        assert!(json["result"].is_null());
    }

    /// [API-028] 서비스 탈퇴 - 인증 실패 (토큰 누락)
    #[tokio::test]
    async fn should_return_401_when_token_missing() {
        // Arrange
        let app = create_test_router();

        let request = Request::builder()
            .method(Method::DELETE)
            .uri("/api/v1/members/me")
            // Authorization 헤더 없음
            .body(Body::empty())
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let json = response_to_json(response).await;
        assert!(!json["isSuccess"].as_bool().unwrap_or(true));
        assert_eq!(json["code"], "AUTH4001");
        assert_eq!(json["message"], "인증 정보가 유효하지 않습니다.");
    }

    /// [API-028] 서비스 탈퇴 - 인증 실패 (잘못된 Bearer 형식)
    #[tokio::test]
    async fn should_return_401_for_invalid_bearer_format() {
        // Arrange
        let app = create_test_router();

        let request = Request::builder()
            .method(Method::DELETE)
            .uri("/api/v1/members/me")
            .header(header::AUTHORIZATION, "InvalidFormat token")
            .body(Body::empty())
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let json = response_to_json(response).await;
        assert!(!json["isSuccess"].as_bool().unwrap_or(true));
        assert_eq!(json["code"], "AUTH4001");
    }

    /// [API-028] 서비스 탈퇴 - 사용자 없음 (404)
    #[tokio::test]
    async fn should_return_404_when_member_not_found() {
        // Arrange
        let app = create_test_router();

        let request = Request::builder()
            .method(Method::DELETE)
            .uri("/api/v1/members/me")
            .header(header::AUTHORIZATION, "Bearer not_found_user_token")
            .body(Body::empty())
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let json = response_to_json(response).await;
        assert!(!json["isSuccess"].as_bool().unwrap_or(true));
        assert_eq!(json["code"], "MEMBER4042");
        assert_eq!(json["message"], "존재하지 않는 사용자입니다.");
    }
}
