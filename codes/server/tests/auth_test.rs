/// AUTH API 통합 테스트
/// API-001: POST /api/v1/auth/social-login
/// API-002: POST /api/v1/auth/signup
/// API-003: POST /api/v1/auth/token/refresh
/// API-004: POST /api/v1/auth/logout

use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
    routing::{get, post},
    Router,
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::util::ServiceExt;

/// 테스트용 라우터 생성 (DB 없이 라우트 검증용)
fn create_test_router() -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/api/v1/auth/social-login", post(social_login_handler))
        .route("/api/v1/auth/signup", post(signup_handler))
        .route("/api/v1/auth/token/refresh", post(refresh_handler))
        .route("/api/v1/auth/logout", post(logout_handler))
}

// 테스트용 핸들러들 - 유효성 검증 로직만 포함
async fn health_handler() -> axum::Json<Value> {
    axum::Json(json!({
        "isSuccess": true,
        "code": "COMMON200",
        "message": "성공입니다.",
        "result": { "status": "healthy" }
    }))
}

async fn social_login_handler(
    body: Option<axum::Json<Value>>,
) -> (StatusCode, axum::Json<Value>) {
    let body = match body {
        Some(b) => b.0,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(json!({
                    "isSuccess": false,
                    "code": "COMMON400",
                    "message": "요청 본문이 없습니다.",
                    "result": null
                })),
            );
        }
    };

    // provider 검증
    if body.get("provider").is_none() {
        return (
            StatusCode::BAD_REQUEST,
            axum::Json(json!({
                "isSuccess": false,
                "code": "COMMON400",
                "message": "필수 파라미터가 누락되었습니다.",
                "result": null
            })),
        );
    }

    // accessToken 검증
    if body.get("accessToken").is_none() {
        return (
            StatusCode::BAD_REQUEST,
            axum::Json(json!({
                "isSuccess": false,
                "code": "COMMON400",
                "message": "필수 파라미터가 누락되었습니다.",
                "result": null
            })),
        );
    }

    // 테스트용: 소셜 토큰 검증 시뮬레이션
    let access_token = body["accessToken"].as_str().unwrap_or("");
    if access_token == "invalid_token" {
        return (
            StatusCode::UNAUTHORIZED,
            axum::Json(json!({
                "isSuccess": false,
                "code": "AUTH4002",
                "message": "유효하지 않은 소셜 토큰입니다.",
                "result": null
            })),
        );
    }

    // 성공 응답 (신규 회원)
    (
        StatusCode::OK,
        axum::Json(json!({
            "isSuccess": true,
            "code": "AUTH2001",
            "message": "신규 회원입니다. 가입 절차를 진행해 주세요.",
            "result": {
                "isNewMember": true,
                "email": "user@example.com",
                "signupToken": "signup_token_xxx"
            }
        })),
    )
}

async fn signup_handler(
    headers: axum::http::HeaderMap,
    body: Option<axum::Json<Value>>,
) -> (StatusCode, axum::Json<Value>) {
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

    let body = match body {
        Some(b) => b.0,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(json!({
                    "isSuccess": false,
                    "code": "COMMON400",
                    "message": "요청 본문이 없습니다.",
                    "result": null
                })),
            );
        }
    };

    // 닉네임 검증 (한글 등 멀티바이트 문자를 위해 chars().count() 사용)
    let nickname = body["nickname"].as_str().unwrap_or("");
    if nickname.is_empty() || nickname.chars().count() > 20 {
        return (
            StatusCode::BAD_REQUEST,
            axum::Json(json!({
                "isSuccess": false,
                "code": "COMMON400",
                "message": "닉네임은 1~20자 이내로 입력해야 합니다.",
                "result": null
            })),
        );
    }

    // 닉네임 중복 시뮬레이션
    if nickname == "이미존재하는닉네임" {
        return (
            StatusCode::CONFLICT,
            axum::Json(json!({
                "isSuccess": false,
                "code": "MEMBER4091",
                "message": "이미 사용 중인 닉네임입니다.",
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
            "message": "회원가입이 성공적으로 완료되었습니다.",
            "result": {
                "memberId": 505,
                "nickname": nickname,
                "accessToken": "service_access_token_xxx",
                "refreshToken": "service_refresh_token_xxx"
            }
        })),
    )
}

async fn refresh_handler(body: Option<axum::Json<Value>>) -> (StatusCode, axum::Json<Value>) {
    let body = match body {
        Some(b) => b.0,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(json!({
                    "isSuccess": false,
                    "code": "COMMON400",
                    "message": "필수 파라미터가 누락되었습니다.",
                    "result": null
                })),
            );
        }
    };

    // refreshToken 검증
    if body.get("refreshToken").is_none() {
        return (
            StatusCode::BAD_REQUEST,
            axum::Json(json!({
                "isSuccess": false,
                "code": "COMMON400",
                "message": "필수 파라미터가 누락되었습니다.",
                "result": null
            })),
        );
    }

    let refresh_token = body["refreshToken"].as_str().unwrap_or("");
    if refresh_token == "expired_refresh_token" {
        return (
            StatusCode::UNAUTHORIZED,
            axum::Json(json!({
                "isSuccess": false,
                "code": "AUTH4004",
                "message": "유효하지 않거나 만료된 Refresh Token입니다.",
                "result": null
            })),
        );
    }

    if refresh_token == "logged_out_refresh_token" {
        return (
            StatusCode::UNAUTHORIZED,
            axum::Json(json!({
                "isSuccess": false,
                "code": "AUTH4005",
                "message": "로그아웃 처리된 토큰입니다. 다시 로그인해 주세요.",
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
            "message": "토큰이 성공적으로 갱신되었습니다.",
            "result": {
                "accessToken": "new_access_token_xxx",
                "refreshToken": "new_refresh_token_xxx"
            }
        })),
    )
}

async fn logout_handler(
    headers: axum::http::HeaderMap,
    body: Option<axum::Json<Value>>,
) -> (StatusCode, axum::Json<Value>) {
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

    let body = match body {
        Some(b) => b.0,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(json!({
                    "isSuccess": false,
                    "code": "COMMON400",
                    "message": "요청 본문이 없습니다.",
                    "result": null
                })),
            );
        }
    };

    let refresh_token = body["refreshToken"].as_str().unwrap_or("");
    if refresh_token == "invalid_or_already_logged_out_token" {
        return (
            StatusCode::BAD_REQUEST,
            axum::Json(json!({
                "isSuccess": false,
                "code": "AUTH4003",
                "message": "이미 로그아웃되었거나 유효하지 않은 토큰입니다.",
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
            "message": "로그아웃이 성공적으로 처리되었습니다.",
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
mod social_login_tests {
    use super::*;

    /// [API-001] 소셜 로그인 - 신규 회원 (가입 필요)
    #[tokio::test]
    async fn should_return_signup_token_for_new_member() {
        // Arrange
        let app = create_test_router();
        let request_body = json!({
            "provider": "GOOGLE",
            "accessToken": "valid_google_token_456"
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/auth/social-login")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let json = response_to_json(response).await;
        assert!(json["isSuccess"].as_bool().unwrap_or(false));
        assert_eq!(json["code"], "AUTH2001");
        assert!(json["result"]["isNewMember"].as_bool().unwrap_or(false));
        assert!(json["result"]["signupToken"].is_string());
    }

    /// [API-001] 소셜 로그인 - 필수 파라미터 누락 (provider 없음)
    #[tokio::test]
    async fn should_return_400_when_provider_missing() {
        // Arrange
        let app = create_test_router();
        let request_body = json!({
            "accessToken": "some_token"
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/auth/social-login")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = response_to_json(response).await;
        assert!(!json["isSuccess"].as_bool().unwrap_or(true));
        assert_eq!(json["code"], "COMMON400");
    }

    /// [API-001] 소셜 로그인 - 유효하지 않은 소셜 토큰
    #[tokio::test]
    async fn should_return_401_for_invalid_social_token() {
        // Arrange
        let app = create_test_router();
        let request_body = json!({
            "provider": "KAKAO",
            "accessToken": "invalid_token"
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/auth/social-login")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let json = response_to_json(response).await;
        assert!(!json["isSuccess"].as_bool().unwrap_or(true));
        assert_eq!(json["code"], "AUTH4002");
    }
}

#[cfg(test)]
mod signup_tests {
    use super::*;

    /// [API-002] 회원가입 - 성공
    #[tokio::test]
    async fn should_complete_signup_successfully() {
        // Arrange
        let app = create_test_router();
        let request_body = json!({
            "email": "user@example.com",
            "nickname": "제이슨"
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/auth/signup")
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::AUTHORIZATION, "Bearer valid_signup_token")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let json = response_to_json(response).await;
        assert!(json["isSuccess"].as_bool().unwrap_or(false));
        assert_eq!(json["code"], "COMMON200");
        assert!(json["result"]["memberId"].is_number());
        assert!(json["result"]["accessToken"].is_string());
        assert!(json["result"]["refreshToken"].is_string());
    }

    /// [API-002] 회원가입 - 닉네임 유효성 검증 실패 (빈 닉네임)
    #[tokio::test]
    async fn should_return_400_for_empty_nickname() {
        // Arrange
        let app = create_test_router();
        let request_body = json!({
            "email": "user@example.com",
            "nickname": ""
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/auth/signup")
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::AUTHORIZATION, "Bearer valid_signup_token")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = response_to_json(response).await;
        assert!(!json["isSuccess"].as_bool().unwrap_or(true));
        assert_eq!(json["code"], "COMMON400");
    }

    /// [API-002] 회원가입 - 닉네임 중복
    #[tokio::test]
    async fn should_return_409_for_duplicate_nickname() {
        // Arrange
        let app = create_test_router();
        let request_body = json!({
            "email": "user@example.com",
            "nickname": "이미존재하는닉네임"
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/auth/signup")
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::AUTHORIZATION, "Bearer valid_signup_token")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::CONFLICT);

        let json = response_to_json(response).await;
        assert!(!json["isSuccess"].as_bool().unwrap_or(true));
        assert_eq!(json["code"], "MEMBER4091");
    }

    /// [API-002] 회원가입 - 인증 실패 (signupToken 누락)
    #[tokio::test]
    async fn should_return_401_when_signup_token_missing() {
        // Arrange
        let app = create_test_router();
        let request_body = json!({
            "email": "user@example.com",
            "nickname": "제이슨"
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/auth/signup")
            .header(header::CONTENT_TYPE, "application/json")
            // Authorization 헤더 없음
            .body(Body::from(request_body.to_string()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let json = response_to_json(response).await;
        assert!(!json["isSuccess"].as_bool().unwrap_or(true));
        assert_eq!(json["code"], "AUTH4001");
    }
}

#[cfg(test)]
mod token_refresh_tests {
    use super::*;

    /// [API-003] 토큰 갱신 - 성공
    #[tokio::test]
    async fn should_refresh_tokens_successfully() {
        // Arrange
        let app = create_test_router();
        let request_body = json!({
            "refreshToken": "valid_refresh_token_xxx"
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/auth/token/refresh")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let json = response_to_json(response).await;
        assert!(json["isSuccess"].as_bool().unwrap_or(false));
        assert_eq!(json["code"], "COMMON200");
        assert!(json["result"]["accessToken"].is_string());
        assert!(json["result"]["refreshToken"].is_string());
    }

    /// [API-003] 토큰 갱신 - 필수 파라미터 누락
    #[tokio::test]
    async fn should_return_400_when_refresh_token_missing() {
        // Arrange
        let app = create_test_router();
        let request_body = json!({});

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/auth/token/refresh")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = response_to_json(response).await;
        assert!(!json["isSuccess"].as_bool().unwrap_or(true));
        assert_eq!(json["code"], "COMMON400");
    }

    /// [API-003] 토큰 갱신 - 만료된 Refresh Token
    #[tokio::test]
    async fn should_return_401_for_expired_refresh_token() {
        // Arrange
        let app = create_test_router();
        let request_body = json!({
            "refreshToken": "expired_refresh_token"
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/auth/token/refresh")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let json = response_to_json(response).await;
        assert!(!json["isSuccess"].as_bool().unwrap_or(true));
        assert_eq!(json["code"], "AUTH4004");
    }

    /// [API-003] 토큰 갱신 - 로그아웃된 토큰
    #[tokio::test]
    async fn should_return_401_for_logged_out_token() {
        // Arrange
        let app = create_test_router();
        let request_body = json!({
            "refreshToken": "logged_out_refresh_token"
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/auth/token/refresh")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let json = response_to_json(response).await;
        assert!(!json["isSuccess"].as_bool().unwrap_or(true));
        assert_eq!(json["code"], "AUTH4005");
    }
}

#[cfg(test)]
mod logout_tests {
    use super::*;

    /// [API-004] 로그아웃 - 성공
    #[tokio::test]
    async fn should_logout_successfully() {
        // Arrange
        let app = create_test_router();
        let request_body = json!({
            "refreshToken": "valid_refresh_token_xxx"
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/auth/logout")
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::AUTHORIZATION, "Bearer valid_access_token")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let json = response_to_json(response).await;
        assert!(json["isSuccess"].as_bool().unwrap_or(false));
        assert_eq!(json["code"], "COMMON200");
    }

    /// [API-004] 로그아웃 - 인증 실패 (accessToken 누락)
    #[tokio::test]
    async fn should_return_401_when_access_token_missing() {
        // Arrange
        let app = create_test_router();
        let request_body = json!({
            "refreshToken": "valid_refresh_token_xxx"
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/auth/logout")
            .header(header::CONTENT_TYPE, "application/json")
            // Authorization 헤더 없음
            .body(Body::from(request_body.to_string()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let json = response_to_json(response).await;
        assert!(!json["isSuccess"].as_bool().unwrap_or(true));
        assert_eq!(json["code"], "AUTH4001");
    }

    /// [API-004] 로그아웃 - 유효하지 않은 refreshToken
    #[tokio::test]
    async fn should_return_400_for_invalid_refresh_token() {
        // Arrange
        let app = create_test_router();
        let request_body = json!({
            "refreshToken": "invalid_or_already_logged_out_token"
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/auth/logout")
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::AUTHORIZATION, "Bearer valid_access_token")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = response_to_json(response).await;
        assert!(!json["isSuccess"].as_bool().unwrap_or(true));
        assert_eq!(json["code"], "AUTH4003");
    }
}

#[cfg(test)]
mod dto_tests {
    use serde::{Deserialize, Serialize};

    /// Social Login Request DTO 직렬화 테스트
    #[test]
    fn should_serialize_social_login_request_with_camel_case() {
        // API 스펙에 따라 provider, accessToken 필드 사용
        #[derive(Debug, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct SocialLoginRequest {
            provider: String,
            access_token: String,
        }

        // Arrange
        let request = SocialLoginRequest {
            provider: "KAKAO".to_string(),
            access_token: "token_123".to_string(),
        };

        // Act
        let json_str = serde_json::to_string(&request).unwrap();

        // Assert
        assert!(json_str.contains("\"provider\""));
        assert!(json_str.contains("\"accessToken\""));
        assert!(!json_str.contains("\"access_token\""));
    }

    /// Social Login Response DTO 직렬화 테스트 (기존 회원)
    #[test]
    fn should_serialize_existing_member_response() {
        #[derive(Debug, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct SocialLoginResponse {
            is_new_member: bool,
            #[serde(skip_serializing_if = "Option::is_none")]
            access_token: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            refresh_token: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            email: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            signup_token: Option<String>,
        }

        // Arrange - 기존 회원
        let response = SocialLoginResponse {
            is_new_member: false,
            access_token: Some("access_token_xxx".to_string()),
            refresh_token: Some("refresh_token_xxx".to_string()),
            email: None,
            signup_token: None,
        };

        // Act
        let json_value: serde_json::Value = serde_json::to_value(&response).unwrap();

        // Assert
        assert_eq!(json_value["isNewMember"], false);
        assert!(json_value["accessToken"].is_string());
        assert!(json_value["refreshToken"].is_string());
        assert!(json_value.get("email").is_none());
        assert!(json_value.get("signupToken").is_none());
    }

    /// Social Login Response DTO 직렬화 테스트 (신규 회원)
    #[test]
    fn should_serialize_new_member_response() {
        #[derive(Debug, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct SocialLoginResponse {
            is_new_member: bool,
            #[serde(skip_serializing_if = "Option::is_none")]
            access_token: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            refresh_token: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            email: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            signup_token: Option<String>,
        }

        // Arrange - 신규 회원
        let response = SocialLoginResponse {
            is_new_member: true,
            access_token: None,
            refresh_token: None,
            email: Some("user@example.com".to_string()),
            signup_token: Some("signup_token_xxx".to_string()),
        };

        // Act
        let json_value: serde_json::Value = serde_json::to_value(&response).unwrap();

        // Assert
        assert_eq!(json_value["isNewMember"], true);
        assert!(json_value.get("accessToken").is_none());
        assert!(json_value.get("refreshToken").is_none());
        assert_eq!(json_value["email"], "user@example.com");
        assert!(json_value["signupToken"].is_string());
    }

    /// Signup Request DTO 직렬화 테스트
    #[test]
    fn should_serialize_signup_request_with_camel_case() {
        #[derive(Debug, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct SignupRequest {
            email: String,
            nickname: String,
        }

        // Arrange
        let request = SignupRequest {
            email: "user@example.com".to_string(),
            nickname: "제이슨".to_string(),
        };

        // Act
        let json_str = serde_json::to_string(&request).unwrap();

        // Assert
        assert!(json_str.contains("\"email\""));
        assert!(json_str.contains("\"nickname\""));
    }

    /// Signup Response DTO 직렬화 테스트
    #[test]
    fn should_serialize_signup_response_with_camel_case() {
        #[derive(Debug, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct SignupResponse {
            member_id: i64,
            nickname: String,
            access_token: String,
            refresh_token: String,
        }

        // Arrange
        let response = SignupResponse {
            member_id: 505,
            nickname: "제이슨".to_string(),
            access_token: "access_token_xxx".to_string(),
            refresh_token: "refresh_token_xxx".to_string(),
        };

        // Act
        let json_value: serde_json::Value = serde_json::to_value(&response).unwrap();

        // Assert
        assert_eq!(json_value["memberId"], 505);
        assert_eq!(json_value["nickname"], "제이슨");
        assert!(json_value["accessToken"].is_string());
        assert!(json_value["refreshToken"].is_string());
    }

    /// Token Refresh Request DTO 직렬화 테스트
    #[test]
    fn should_serialize_token_refresh_request_with_camel_case() {
        #[derive(Debug, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct TokenRefreshRequest {
            refresh_token: String,
        }

        // Arrange
        let request = TokenRefreshRequest {
            refresh_token: "refresh_token_xxx".to_string(),
        };

        // Act
        let json_str = serde_json::to_string(&request).unwrap();

        // Assert
        assert!(json_str.contains("\"refreshToken\""));
        assert!(!json_str.contains("\"refresh_token\""));
    }

    /// Token Refresh Response DTO 직렬화 테스트
    #[test]
    fn should_serialize_token_refresh_response_with_camel_case() {
        #[derive(Debug, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct TokenRefreshResponse {
            access_token: String,
            refresh_token: String,
        }

        // Arrange
        let response = TokenRefreshResponse {
            access_token: "new_access_token_xxx".to_string(),
            refresh_token: "new_refresh_token_xxx".to_string(),
        };

        // Act
        let json_value: serde_json::Value = serde_json::to_value(&response).unwrap();

        // Assert
        assert!(json_value["accessToken"].is_string());
        assert!(json_value["refreshToken"].is_string());
    }

    /// Logout Request DTO 직렬화 테스트
    #[test]
    fn should_serialize_logout_request_with_camel_case() {
        #[derive(Debug, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct LogoutRequest {
            refresh_token: String,
        }

        // Arrange
        let request = LogoutRequest {
            refresh_token: "refresh_token_xxx".to_string(),
        };

        // Act
        let json_str = serde_json::to_string(&request).unwrap();

        // Assert
        assert!(json_str.contains("\"refreshToken\""));
        assert!(!json_str.contains("\"refresh_token\""));
    }
}
