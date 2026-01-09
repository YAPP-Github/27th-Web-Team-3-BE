//! Handler 테스트
//!
//! axum-test를 사용한 HTTP 핸들러 레이어 테스트

use async_openai::types::ChatCompletionRequestMessage;
use axum_test::TestServer;
use serde_json::json;
use web3_server::{create_test_router_with_mock, error::AppError, AiClientTrait};

/// 테스트용 Mock AI 클라이언트 (성공 응답)
struct MockAiClientSuccess {
    response: String,
}

impl MockAiClientSuccess {
    fn new(response: &str) -> Self {
        Self {
            response: response.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl AiClientTrait for MockAiClientSuccess {
    async fn complete(
        &self,
        _messages: Vec<ChatCompletionRequestMessage>,
    ) -> Result<String, AppError> {
        Ok(self.response.clone())
    }

    async fn check_connectivity(&self) -> Result<(), AppError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<String, AppError> {
        Ok("ok".to_string())
    }
}

/// 테스트용 Mock AI 클라이언트 (에러 응답)
struct MockAiClientError {
    error_message: String,
}

impl MockAiClientError {
    fn new(message: &str) -> Self {
        Self {
            error_message: message.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl AiClientTrait for MockAiClientError {
    async fn complete(
        &self,
        _messages: Vec<ChatCompletionRequestMessage>,
    ) -> Result<String, AppError> {
        Err(AppError::OpenAiError(self.error_message.clone()))
    }

    async fn check_connectivity(&self) -> Result<(), AppError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<String, AppError> {
        Ok("ok".to_string())
    }
}

mod guide_handler {
    use super::*;

    const SECRET_KEY: &str = "test-secret-key";

    #[tokio::test]
    async fn should_return_200_for_valid_request() {
        // Arrange
        let mock = MockAiClientSuccess::new("좋은 시작이에요! 더 구체적으로 작성해보세요.");
        let app = create_test_router_with_mock(SECRET_KEY, mock);
        let server = TestServer::new(app).unwrap();

        // Act
        let response = server
            .post("/api/ai/retrospective/guide")
            .json(&json!({
                "currentContent": "오늘 프로젝트를 진행하면서...",
                "secretKey": SECRET_KEY
            }))
            .await;

        // Assert
        response.assert_status_ok();
        response.assert_json_contains(&json!({
            "isSuccess": true,
            "code": "COMMON200",
            "message": "성공입니다."
        }));

        let body: serde_json::Value = response.json();
        assert!(body["result"]["currentContent"]
            .as_str()
            .unwrap()
            .contains("오늘 프로젝트를 진행하면서..."));
        assert!(!body["result"]["guideMessage"].as_str().unwrap().is_empty());
    }

    #[tokio::test]
    async fn should_return_401_for_invalid_secret_key() {
        // Arrange
        let mock = MockAiClientSuccess::new("test");
        let app = create_test_router_with_mock(SECRET_KEY, mock);
        let server = TestServer::new(app).unwrap();

        // Act
        let response = server
            .post("/api/ai/retrospective/guide")
            .json(&json!({
                "currentContent": "테스트 내용",
                "secretKey": "wrong-key"
            }))
            .await;

        // Assert
        response.assert_status_unauthorized();
        response.assert_json_contains(&json!({
            "isSuccess": false,
            "code": "AI_001"
        }));
    }

    #[tokio::test]
    async fn should_return_400_for_empty_content() {
        // Arrange
        let mock = MockAiClientSuccess::new("test");
        let app = create_test_router_with_mock(SECRET_KEY, mock);
        let server = TestServer::new(app).unwrap();

        // Act
        let response = server
            .post("/api/ai/retrospective/guide")
            .json(&json!({
                "currentContent": "",
                "secretKey": SECRET_KEY
            }))
            .await;

        // Assert
        response.assert_status_bad_request();
        response.assert_json_contains(&json!({
            "isSuccess": false,
            "code": "COMMON400"
        }));
    }

    #[tokio::test]
    async fn should_return_400_for_missing_content() {
        // Arrange
        let mock = MockAiClientSuccess::new("test");
        let app = create_test_router_with_mock(SECRET_KEY, mock);
        let server = TestServer::new(app).unwrap();

        // Act
        let response = server
            .post("/api/ai/retrospective/guide")
            .json(&json!({
                "secretKey": SECRET_KEY
            }))
            .await;

        // Assert
        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn should_return_400_for_invalid_json() {
        // Arrange
        let mock = MockAiClientSuccess::new("test");
        let app = create_test_router_with_mock(SECRET_KEY, mock);
        let server = TestServer::new(app).unwrap();

        // Act
        let response = server
            .post("/api/ai/retrospective/guide")
            .content_type("application/json")
            .bytes("{invalid json}".as_bytes().into())
            .await;

        // Assert
        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn should_return_500_when_openai_fails() {
        // Arrange
        let mock = MockAiClientError::new("API Error");
        let app = create_test_router_with_mock(SECRET_KEY, mock);
        let server = TestServer::new(app).unwrap();

        // Act
        let response = server
            .post("/api/ai/retrospective/guide")
            .json(&json!({
                "currentContent": "테스트 내용",
                "secretKey": SECRET_KEY
            }))
            .await;

        // Assert
        response.assert_status(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        response.assert_json_contains(&json!({
            "isSuccess": false,
            "code": "AI_006"
        }));
    }
}

mod refine_handler {
    use super::*;

    const SECRET_KEY: &str = "test-secret-key";

    #[tokio::test]
    async fn should_return_200_for_kind_style() {
        // Arrange
        let mock = MockAiClientSuccess::new("오늘 일이 많이 힘들었어요.");
        let app = create_test_router_with_mock(SECRET_KEY, mock);
        let server = TestServer::new(app).unwrap();

        // Act
        let response = server
            .post("/api/ai/retrospective/refine")
            .json(&json!({
                "content": "오늘 일 힘들었음",
                "toneStyle": "KIND",
                "secretKey": SECRET_KEY
            }))
            .await;

        // Assert
        response.assert_status_ok();
        response.assert_json_contains(&json!({
            "isSuccess": true,
            "code": "COMMON200"
        }));

        let body: serde_json::Value = response.json();
        assert_eq!(body["result"]["originalContent"], "오늘 일 힘들었음");
        assert_eq!(body["result"]["toneStyle"], "KIND");
    }

    #[tokio::test]
    async fn should_return_200_for_polite_style() {
        // Arrange
        let mock = MockAiClientSuccess::new("오늘 일이 많이 힘들었습니다.");
        let app = create_test_router_with_mock(SECRET_KEY, mock);
        let server = TestServer::new(app).unwrap();

        // Act
        let response = server
            .post("/api/ai/retrospective/refine")
            .json(&json!({
                "content": "오늘 일 힘들었음",
                "toneStyle": "POLITE",
                "secretKey": SECRET_KEY
            }))
            .await;

        // Assert
        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        assert_eq!(body["result"]["toneStyle"], "POLITE");
    }

    #[tokio::test]
    async fn should_return_401_for_invalid_secret_key() {
        // Arrange
        let mock = MockAiClientSuccess::new("test");
        let app = create_test_router_with_mock(SECRET_KEY, mock);
        let server = TestServer::new(app).unwrap();

        // Act
        let response = server
            .post("/api/ai/retrospective/refine")
            .json(&json!({
                "content": "테스트",
                "toneStyle": "KIND",
                "secretKey": "wrong-key"
            }))
            .await;

        // Assert
        response.assert_status_unauthorized();
        response.assert_json_contains(&json!({
            "isSuccess": false,
            "code": "AI_001"
        }));
    }

    #[tokio::test]
    async fn should_return_400_for_empty_content() {
        // Arrange
        let mock = MockAiClientSuccess::new("test");
        let app = create_test_router_with_mock(SECRET_KEY, mock);
        let server = TestServer::new(app).unwrap();

        // Act
        let response = server
            .post("/api/ai/retrospective/refine")
            .json(&json!({
                "content": "",
                "toneStyle": "KIND",
                "secretKey": SECRET_KEY
            }))
            .await;

        // Assert
        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn should_return_400_for_invalid_tone_style() {
        // Arrange
        let mock = MockAiClientSuccess::new("test");
        let app = create_test_router_with_mock(SECRET_KEY, mock);
        let server = TestServer::new(app).unwrap();

        // Act
        let response = server
            .post("/api/ai/retrospective/refine")
            .json(&json!({
                "content": "테스트",
                "toneStyle": "INVALID",
                "secretKey": SECRET_KEY
            }))
            .await;

        // Assert
        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn should_return_400_for_lowercase_tone_style() {
        // Arrange
        let mock = MockAiClientSuccess::new("test");
        let app = create_test_router_with_mock(SECRET_KEY, mock);
        let server = TestServer::new(app).unwrap();

        // Act
        let response = server
            .post("/api/ai/retrospective/refine")
            .json(&json!({
                "content": "테스트",
                "toneStyle": "kind",
                "secretKey": SECRET_KEY
            }))
            .await;

        // Assert
        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn should_return_500_when_openai_fails() {
        // Arrange
        let mock = MockAiClientError::new("API Error");
        let app = create_test_router_with_mock(SECRET_KEY, mock);
        let server = TestServer::new(app).unwrap();

        // Act
        let response = server
            .post("/api/ai/retrospective/refine")
            .json(&json!({
                "content": "테스트",
                "toneStyle": "KIND",
                "secretKey": SECRET_KEY
            }))
            .await;

        // Assert
        response.assert_status(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }
}

mod response_format {
    use super::*;

    const SECRET_KEY: &str = "test-secret-key";

    #[tokio::test]
    async fn success_response_should_use_camel_case() {
        // Arrange
        let mock = MockAiClientSuccess::new("가이드 메시지");
        let app = create_test_router_with_mock(SECRET_KEY, mock);
        let server = TestServer::new(app).unwrap();

        // Act
        let response = server
            .post("/api/ai/retrospective/guide")
            .json(&json!({
                "currentContent": "테스트",
                "secretKey": SECRET_KEY
            }))
            .await;

        // Assert
        let body: serde_json::Value = response.json();

        // Top-level fields should be camelCase
        assert!(body.get("isSuccess").is_some());
        assert!(body.get("code").is_some());
        assert!(body.get("message").is_some());
        assert!(body.get("result").is_some());

        // Result fields should be camelCase
        assert!(body["result"].get("currentContent").is_some());
        assert!(body["result"].get("guideMessage").is_some());

        // No snake_case fields
        assert!(body.get("is_success").is_none());
        assert!(body["result"].get("current_content").is_none());
        assert!(body["result"].get("guide_message").is_none());
    }

    #[tokio::test]
    async fn error_response_should_have_null_result() {
        // Arrange
        let mock = MockAiClientSuccess::new("test");
        let app = create_test_router_with_mock(SECRET_KEY, mock);
        let server = TestServer::new(app).unwrap();

        // Act
        let response = server
            .post("/api/ai/retrospective/guide")
            .json(&json!({
                "currentContent": "",
                "secretKey": SECRET_KEY
            }))
            .await;

        // Assert
        let body: serde_json::Value = response.json();
        assert!(body["result"].is_null());
    }
}

// ===== Task 1.5: 동시성 테스트 =====
mod concurrency {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    const SECRET_KEY: &str = "test-secret-key";

    /// 동시 요청을 처리하는 Mock 클라이언트
    struct MockAiClientConcurrent {
        call_count: Arc<AtomicUsize>,
    }

    impl MockAiClientConcurrent {
        fn new() -> Self {
            Self {
                call_count: Arc::new(AtomicUsize::new(0)),
            }
        }

        fn get_call_count(&self) -> Arc<AtomicUsize> {
            self.call_count.clone()
        }
    }

    #[async_trait::async_trait]
    impl AiClientTrait for MockAiClientConcurrent {
        async fn complete(
            &self,
            _messages: Vec<ChatCompletionRequestMessage>,
        ) -> Result<String, AppError> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            // 약간의 지연을 추가하여 동시성 테스트 효과 증대
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            Ok("응답 메시지".to_string())
        }

        async fn check_connectivity(&self) -> Result<(), AppError> {
            Ok(())
        }

        async fn health_check(&self) -> Result<String, AppError> {
            Ok("ok".to_string())
        }
    }

    #[tokio::test]
    async fn should_handle_sequential_guide_requests() {
        // Arrange
        let mock = MockAiClientConcurrent::new();
        let call_count = mock.get_call_count();
        let app = create_test_router_with_mock(SECRET_KEY, mock);
        let server = TestServer::new(app).unwrap();

        // Act - 10개의 순차 요청
        for i in 0..10 {
            let response = server
                .post("/api/ai/retrospective/guide")
                .json(&json!({
                    "currentContent": format!("테스트 내용 {}", i),
                    "secretKey": SECRET_KEY
                }))
                .await;
            response.assert_status_ok();
        }

        // Assert - 10번의 API 호출이 이루어졌는지 확인
        assert_eq!(call_count.load(Ordering::SeqCst), 10);
    }

    #[tokio::test]
    async fn should_handle_sequential_refine_requests() {
        // Arrange
        let mock = MockAiClientConcurrent::new();
        let call_count = mock.get_call_count();
        let app = create_test_router_with_mock(SECRET_KEY, mock);
        let server = TestServer::new(app).unwrap();

        // Act - 10개의 순차 요청 (KIND와 POLITE 혼합)
        for i in 0..10 {
            let tone_style = if i % 2 == 0 { "KIND" } else { "POLITE" };
            let response = server
                .post("/api/ai/retrospective/refine")
                .json(&json!({
                    "content": format!("테스트 내용 {}", i),
                    "toneStyle": tone_style,
                    "secretKey": SECRET_KEY
                }))
                .await;
            response.assert_status_ok();
        }

        // Assert
        assert_eq!(call_count.load(Ordering::SeqCst), 10);
    }

    #[tokio::test]
    async fn should_handle_mixed_sequential_requests() {
        // Arrange
        let mock = MockAiClientConcurrent::new();
        let call_count = mock.get_call_count();
        let app = create_test_router_with_mock(SECRET_KEY, mock);
        let server = TestServer::new(app).unwrap();

        // Act - guide와 refine 요청을 번갈아 보냄
        for i in 0..5 {
            // Guide 요청
            let response = server
                .post("/api/ai/retrospective/guide")
                .json(&json!({
                    "currentContent": format!("가이드 테스트 {}", i),
                    "secretKey": SECRET_KEY
                }))
                .await;
            response.assert_status_ok();

            // Refine 요청
            let response = server
                .post("/api/ai/retrospective/refine")
                .json(&json!({
                    "content": format!("정제 테스트 {}", i),
                    "toneStyle": "KIND",
                    "secretKey": SECRET_KEY
                }))
                .await;
            response.assert_status_ok();
        }

        // Assert
        assert_eq!(call_count.load(Ordering::SeqCst), 10);
    }

    #[tokio::test]
    async fn should_maintain_state_across_requests() {
        // Arrange - 여러 요청 간 서비스 상태 유지 확인
        let mock = MockAiClientConcurrent::new();
        let call_count = mock.get_call_count();
        let app = create_test_router_with_mock(SECRET_KEY, mock);
        let server = TestServer::new(app).unwrap();

        // Act - 여러 요청
        let response1 = server
            .post("/api/ai/retrospective/guide")
            .json(&json!({
                "currentContent": "첫 번째 요청",
                "secretKey": SECRET_KEY
            }))
            .await;

        let response2 = server
            .post("/api/ai/retrospective/guide")
            .json(&json!({
                "currentContent": "두 번째 요청",
                "secretKey": SECRET_KEY
            }))
            .await;

        // Assert
        response1.assert_status_ok();
        response2.assert_status_ok();
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn should_handle_concurrent_guide_requests() {
        use axum::body::Body;
        use axum::http::Request;
        use axum::response::Response;
        use std::convert::Infallible;
        use tower::ServiceExt;

        // Arrange
        let mock = MockAiClientConcurrent::new();
        let call_count = mock.get_call_count();
        let app = create_test_router_with_mock(SECRET_KEY, mock);

        // Act - 10개의 병렬 요청을 위한 request 생성
        let requests: Vec<Request<Body>> = (0..10)
            .map(|i| {
                Request::builder()
                    .method("POST")
                    .uri("/api/ai/retrospective/guide")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "currentContent": format!("병렬 테스트 {}", i),
                            "secretKey": SECRET_KEY
                        })
                        .to_string(),
                    ))
                    .unwrap()
            })
            .collect();

        // 병렬 요청 실행
        let handles: Vec<_> = requests
            .into_iter()
            .map(|req| {
                let app = app.clone();
                tokio::spawn(async move {
                    let result: Result<Response, Infallible> = app.oneshot(req).await;
                    result
                })
            })
            .collect();

        let results = futures::future::join_all(handles).await;

        // Assert - 모든 요청이 성공해야 함
        for result in results {
            let response = result.expect("Task should not panic").unwrap();
            assert!(response.status().is_success());
        }

        // 10번의 API 호출이 이루어졌는지 확인
        assert_eq!(call_count.load(Ordering::SeqCst), 10);
    }

    #[tokio::test]
    async fn should_handle_concurrent_mixed_requests() {
        use axum::body::Body;
        use axum::http::Request;
        use axum::response::Response;
        use std::convert::Infallible;
        use tower::ServiceExt;

        // Arrange
        let mock = MockAiClientConcurrent::new();
        let call_count = mock.get_call_count();
        let app = create_test_router_with_mock(SECRET_KEY, mock);

        // Act - guide와 refine을 병렬로 요청
        let mut requests: Vec<Request<Body>> = Vec::new();

        // 5개의 guide 요청
        for i in 0..5 {
            requests.push(
                Request::builder()
                    .method("POST")
                    .uri("/api/ai/retrospective/guide")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "currentContent": format!("가이드 {}", i),
                            "secretKey": SECRET_KEY
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            );
        }

        // 5개의 refine 요청
        for i in 0..5 {
            let tone_style = if i % 2 == 0 { "KIND" } else { "POLITE" };
            requests.push(
                Request::builder()
                    .method("POST")
                    .uri("/api/ai/retrospective/refine")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "content": format!("정제 {}", i),
                            "toneStyle": tone_style,
                            "secretKey": SECRET_KEY
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            );
        }

        // 병렬 요청 실행
        let handles: Vec<_> = requests
            .into_iter()
            .map(|req| {
                let app = app.clone();
                tokio::spawn(async move {
                    let result: Result<Response, Infallible> = app.oneshot(req).await;
                    result
                })
            })
            .collect();

        let results = futures::future::join_all(handles).await;

        // Assert - 모든 요청이 성공해야 함
        for result in results {
            let response = result.expect("Task should not panic").unwrap();
            assert!(response.status().is_success());
        }

        // 10번의 API 호출이 이루어졌는지 확인
        assert_eq!(call_count.load(Ordering::SeqCst), 10);
    }

    #[tokio::test]
    async fn should_handle_high_concurrency() {
        use axum::body::Body;
        use axum::http::Request;
        use axum::response::Response;
        use std::convert::Infallible;
        use tower::ServiceExt;

        // Arrange - 높은 동시성 테스트 (50개 요청)
        let mock = MockAiClientConcurrent::new();
        let call_count = mock.get_call_count();
        let app = create_test_router_with_mock(SECRET_KEY, mock);

        // Act - 50개의 병렬 요청
        let requests: Vec<Request<Body>> = (0..50)
            .map(|i| {
                Request::builder()
                    .method("POST")
                    .uri("/api/ai/retrospective/guide")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "currentContent": format!("고부하 테스트 {}", i),
                            "secretKey": SECRET_KEY
                        })
                        .to_string(),
                    ))
                    .unwrap()
            })
            .collect();

        // 병렬 요청 실행
        let handles: Vec<_> = requests
            .into_iter()
            .map(|req| {
                let app = app.clone();
                tokio::spawn(async move {
                    let result: Result<Response, Infallible> = app.oneshot(req).await;
                    result
                })
            })
            .collect();

        let results: Vec<Result<Result<Response, Infallible>, _>> =
            futures::future::join_all(handles).await;

        // Assert - 모든 요청이 성공해야 함
        let success_count = results
            .iter()
            .filter(|r| r.is_ok())
            .filter(|r| r.as_ref().unwrap().as_ref().unwrap().status().is_success())
            .count();

        assert_eq!(success_count, 50);
        assert_eq!(call_count.load(Ordering::SeqCst), 50);
    }
}

// ===== Task 1.6: 엣지 케이스 테스트 =====
mod edge_cases {
    use super::*;

    const SECRET_KEY: &str = "test-secret-key";

    #[tokio::test]
    async fn should_handle_unicode_content() {
        // Arrange
        let mock = MockAiClientSuccess::new("응답 메시지");
        let app = create_test_router_with_mock(SECRET_KEY, mock);
        let server = TestServer::new(app).unwrap();

        // Act - 유니코드 (한글, 이모지, 일본어)
        let response = server
            .post("/api/ai/retrospective/guide")
            .json(&json!({
                "currentContent": "한글 테스트 🎉 日本語 émoji",
                "secretKey": SECRET_KEY
            }))
            .await;

        // Assert
        response.assert_status_ok();
    }

    #[tokio::test]
    async fn should_handle_very_long_content() {
        // Arrange
        let mock = MockAiClientSuccess::new("응답 메시지");
        let app = create_test_router_with_mock(SECRET_KEY, mock);
        let server = TestServer::new(app).unwrap();

        // Act - 5000자 이상의 긴 내용
        let long_content = "가".repeat(5000);
        let response = server
            .post("/api/ai/retrospective/guide")
            .json(&json!({
                "currentContent": long_content,
                "secretKey": SECRET_KEY
            }))
            .await;

        // Assert
        response.assert_status_ok();
    }

    #[tokio::test]
    async fn should_handle_special_characters() {
        // Arrange
        let mock = MockAiClientSuccess::new("응답 메시지");
        let app = create_test_router_with_mock(SECRET_KEY, mock);
        let server = TestServer::new(app).unwrap();

        // Act - 특수 문자 (XSS 시도 포함)
        let response = server
            .post("/api/ai/retrospective/guide")
            .json(&json!({
                "currentContent": "<script>alert('xss')</script> & \"quotes\" 'apostrophes'",
                "secretKey": SECRET_KEY
            }))
            .await;

        // Assert - 서버는 정상 처리해야 함 (XSS는 클라이언트에서 이스케이프)
        response.assert_status_ok();
    }

    #[tokio::test]
    async fn should_handle_whitespace_only_content() {
        // Arrange
        let mock = MockAiClientSuccess::new("응답 메시지");
        let app = create_test_router_with_mock(SECRET_KEY, mock);
        let server = TestServer::new(app).unwrap();

        // Act - 공백만 있는 내용 (validation 통과)
        let response = server
            .post("/api/ai/retrospective/guide")
            .json(&json!({
                "currentContent": "   \t\n   ",
                "secretKey": SECRET_KEY
            }))
            .await;

        // Assert - 현재는 통과 (최소 길이 1만 검증)
        // Phase 3에서 trim 후 검증 추가 예정
        response.assert_status_ok();
    }

    #[tokio::test]
    async fn should_handle_newlines_in_content() {
        // Arrange
        let mock = MockAiClientSuccess::new("응답 메시지");
        let app = create_test_router_with_mock(SECRET_KEY, mock);
        let server = TestServer::new(app).unwrap();

        // Act - 여러 줄의 내용
        let response = server
            .post("/api/ai/retrospective/guide")
            .json(&json!({
                "currentContent": "첫 번째 줄\n두 번째 줄\n세 번째 줄",
                "secretKey": SECRET_KEY
            }))
            .await;

        // Assert
        response.assert_status_ok();
    }

    #[tokio::test]
    async fn should_handle_json_special_chars_in_content() {
        // Arrange
        let mock = MockAiClientSuccess::new("응답 메시지");
        let app = create_test_router_with_mock(SECRET_KEY, mock);
        let server = TestServer::new(app).unwrap();

        // Act - JSON 특수 문자
        let response = server
            .post("/api/ai/retrospective/guide")
            .json(&json!({
                "currentContent": r#"{"key": "value"} [array] \n \t \\"#,
                "secretKey": SECRET_KEY
            }))
            .await;

        // Assert
        response.assert_status_ok();
    }

    #[tokio::test]
    async fn should_handle_sql_injection_attempt() {
        // Arrange
        let mock = MockAiClientSuccess::new("응답 메시지");
        let app = create_test_router_with_mock(SECRET_KEY, mock);
        let server = TestServer::new(app).unwrap();

        // Act - SQL 인젝션 시도 (실제로는 DB 사용 안 함)
        let response = server
            .post("/api/ai/retrospective/guide")
            .json(&json!({
                "currentContent": "'; DROP TABLE users; --",
                "secretKey": SECRET_KEY
            }))
            .await;

        // Assert - 단순히 문자열로 처리되어야 함
        response.assert_status_ok();
    }

    #[tokio::test]
    async fn should_handle_binary_like_content() {
        // Arrange
        let mock = MockAiClientSuccess::new("응답 메시지");
        let app = create_test_router_with_mock(SECRET_KEY, mock);
        let server = TestServer::new(app).unwrap();

        // Act - 바이너리처럼 보이는 문자열
        let response = server
            .post("/api/ai/retrospective/guide")
            .json(&json!({
                "currentContent": "\u{0000}\u{0001}\u{0002}",
                "secretKey": SECRET_KEY
            }))
            .await;

        // Assert - 서버가 처리할 수 있어야 함
        response.assert_status_ok();
    }
}
