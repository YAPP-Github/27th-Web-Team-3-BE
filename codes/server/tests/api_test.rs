use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;
use web3_server::create_test_router;

const TEST_SECRET_KEY: &str = "test-secret-key";

// ===== Helper Functions =====

async fn parse_response_body(body: Body) -> Value {
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

fn create_json_request(method: &str, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("Content-Type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

// ===== Health Check Tests =====

mod health {
    use super::*;

    #[tokio::test]
    async fn should_return_ok() {
        let app = create_test_router(TEST_SECRET_KEY);

        let request = Request::builder()
            .method("GET")
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}

// ===== Guide API Tests =====

mod guide_api {
    use super::*;

    const GUIDE_URI: &str = "/api/ai/retrospective/guide";

    #[tokio::test]
    async fn should_return_401_with_invalid_secret_key() {
        let app = create_test_router(TEST_SECRET_KEY);

        let request = create_json_request(
            "POST",
            GUIDE_URI,
            json!({
                "currentContent": "오늘 프로젝트를 진행하면서...",
                "secretKey": "wrong-key"
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body = parse_response_body(response.into_body()).await;
        assert_eq!(body["isSuccess"], false);
        assert_eq!(body["code"], "AI_001");
        assert!(body["message"].as_str().unwrap().contains("비밀 키"));
    }

    #[tokio::test]
    async fn should_return_400_with_empty_content() {
        let app = create_test_router(TEST_SECRET_KEY);

        let request = create_json_request(
            "POST",
            GUIDE_URI,
            json!({
                "currentContent": "",
                "secretKey": TEST_SECRET_KEY
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = parse_response_body(response.into_body()).await;
        assert_eq!(body["isSuccess"], false);
        assert_eq!(body["code"], "COMMON400");
    }

    #[tokio::test]
    async fn should_return_400_with_missing_content() {
        let app = create_test_router(TEST_SECRET_KEY);

        let request = create_json_request(
            "POST",
            GUIDE_URI,
            json!({
                "secretKey": TEST_SECRET_KEY
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = parse_response_body(response.into_body()).await;
        assert_eq!(body["isSuccess"], false);
        assert_eq!(body["code"], "COMMON400");
    }

    #[tokio::test]
    async fn should_return_400_with_missing_secret_key() {
        let app = create_test_router(TEST_SECRET_KEY);

        let request = create_json_request(
            "POST",
            GUIDE_URI,
            json!({
                "currentContent": "오늘 프로젝트를 진행하면서..."
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = parse_response_body(response.into_body()).await;
        assert_eq!(body["isSuccess"], false);
        assert_eq!(body["code"], "COMMON400");
    }

    #[tokio::test]
    async fn should_return_400_with_invalid_json() {
        let app = create_test_router(TEST_SECRET_KEY);

        let request = Request::builder()
            .method("POST")
            .uri(GUIDE_URI)
            .header("Content-Type", "application/json")
            .body(Body::from("{ invalid json }"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = parse_response_body(response.into_body()).await;
        assert_eq!(body["isSuccess"], false);
        assert_eq!(body["code"], "COMMON400");
    }
}

// ===== Refine API Tests =====

mod refine_api {
    use super::*;

    const REFINE_URI: &str = "/api/ai/retrospective/refine";

    #[tokio::test]
    async fn should_return_401_with_invalid_secret_key() {
        let app = create_test_router(TEST_SECRET_KEY);

        let request = create_json_request(
            "POST",
            REFINE_URI,
            json!({
                "content": "오늘 힘들었음",
                "toneStyle": "KIND",
                "secretKey": "wrong-key"
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body = parse_response_body(response.into_body()).await;
        assert_eq!(body["isSuccess"], false);
        assert_eq!(body["code"], "AI_001");
    }

    #[tokio::test]
    async fn should_return_400_with_invalid_tone_style() {
        let app = create_test_router(TEST_SECRET_KEY);

        let request = create_json_request(
            "POST",
            REFINE_URI,
            json!({
                "content": "오늘 힘들었음",
                "toneStyle": "INVALID",
                "secretKey": TEST_SECRET_KEY
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = parse_response_body(response.into_body()).await;
        assert_eq!(body["isSuccess"], false);
        assert_eq!(body["code"], "AI_002");
        assert!(body["message"]
            .as_str()
            .unwrap()
            .contains("KIND 또는 POLITE"));
    }

    #[tokio::test]
    async fn should_return_400_with_empty_content() {
        let app = create_test_router(TEST_SECRET_KEY);

        let request = create_json_request(
            "POST",
            REFINE_URI,
            json!({
                "content": "",
                "toneStyle": "KIND",
                "secretKey": TEST_SECRET_KEY
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = parse_response_body(response.into_body()).await;
        assert_eq!(body["isSuccess"], false);
        assert_eq!(body["code"], "COMMON400");
    }

    #[tokio::test]
    async fn should_return_400_with_missing_content() {
        let app = create_test_router(TEST_SECRET_KEY);

        let request = create_json_request(
            "POST",
            REFINE_URI,
            json!({
                "toneStyle": "KIND",
                "secretKey": TEST_SECRET_KEY
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = parse_response_body(response.into_body()).await;
        assert_eq!(body["isSuccess"], false);
        assert_eq!(body["code"], "COMMON400");
    }

    #[tokio::test]
    async fn should_return_400_with_missing_tone_style() {
        let app = create_test_router(TEST_SECRET_KEY);

        let request = create_json_request(
            "POST",
            REFINE_URI,
            json!({
                "content": "오늘 힘들었음",
                "secretKey": TEST_SECRET_KEY
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = parse_response_body(response.into_body()).await;
        assert_eq!(body["isSuccess"], false);
        // toneStyle 누락 시 AI_002 (ToneStyle 관련 에러)
        assert_eq!(body["code"], "AI_002");
    }

    #[tokio::test]
    async fn should_return_400_with_lowercase_tone_style() {
        let app = create_test_router(TEST_SECRET_KEY);

        // "kind" (소문자)는 유효하지 않음 - "KIND" (대문자)여야 함
        let request = create_json_request(
            "POST",
            REFINE_URI,
            json!({
                "content": "오늘 힘들었음",
                "toneStyle": "kind",
                "secretKey": TEST_SECRET_KEY
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = parse_response_body(response.into_body()).await;
        assert_eq!(body["isSuccess"], false);
        assert_eq!(body["code"], "AI_002");
    }

    #[tokio::test]
    async fn should_return_400_with_invalid_json() {
        let app = create_test_router(TEST_SECRET_KEY);

        let request = Request::builder()
            .method("POST")
            .uri(REFINE_URI)
            .header("Content-Type", "application/json")
            .body(Body::from("not valid json"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = parse_response_body(response.into_body()).await;
        assert_eq!(body["isSuccess"], false);
        assert_eq!(body["code"], "COMMON400");
    }
}

// ===== Response Format Tests =====

mod response_format {
    use super::*;

    #[tokio::test]
    async fn error_response_should_use_camel_case() {
        let app = create_test_router(TEST_SECRET_KEY);

        let request = create_json_request(
            "POST",
            "/api/ai/retrospective/guide",
            json!({
                "currentContent": "test",
                "secretKey": "wrong-key"
            }),
        );

        let response = app.oneshot(request).await.unwrap();
        let body = parse_response_body(response.into_body()).await;

        // camelCase 필드 확인
        assert!(body.get("isSuccess").is_some());
        assert!(body.get("code").is_some());
        assert!(body.get("message").is_some());
        assert!(body.get("result").is_some());

        // snake_case 필드가 없어야 함
        assert!(body.get("is_success").is_none());
    }

    #[tokio::test]
    async fn error_response_should_have_null_result() {
        let app = create_test_router(TEST_SECRET_KEY);

        let request = create_json_request(
            "POST",
            "/api/ai/retrospective/guide",
            json!({
                "currentContent": "test",
                "secretKey": "wrong-key"
            }),
        );

        let response = app.oneshot(request).await.unwrap();
        let body = parse_response_body(response.into_body()).await;

        assert!(body["result"].is_null());
    }
}
