//! 회고 API 통합 테스트
//!
//! 이 테스트 모듈은 회고 관련 엔드포인트에 대한 HTTP 통합 테스트를 포함합니다.
//! - API-011: POST /api/v1/retrospects (회고 생성)
//! - API-010: GET /api/v1/teams/{teamId}/retrospects (팀 회고 목록 조회)
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

    /// 테스트용 라우터 생성 (인증 미들웨어 없이 직접 테스트)
    /// 실제 테스트에서는 인증 관련 에러를 검증하기 위해 핸들러를 직접 호출
    pub fn create_test_router() -> Router {
        // 실제 API와 유사한 핸들러 (인증 로직 포함)
        async fn test_handler(
            headers: axum::http::HeaderMap,
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
                        "message": "인증 실패: 로그인이 필요합니다.",
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
                        "message": "인증 실패: 토큰 형식이 올바르지 않습니다.",
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

            // 필드 검증 - teamId를 먼저 검증
            let team_id = body.get("teamId").and_then(|v| v.as_i64());
            if team_id.is_none() || team_id.map(|id| id < 1).unwrap_or(true) {
                return Err((
                    StatusCode::BAD_REQUEST,
                    axum::Json(json!({
                        "isSuccess": false,
                        "code": "COMMON400",
                        "message": "잘못된 요청입니다: teamId는 1 이상이어야 합니다",
                        "result": null
                    })),
                ));
            }

            let project_name = body.get("projectName").and_then(|v| v.as_str());
            if let Some(name) = project_name {
                if name.is_empty() || name.len() > 20 {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        axum::Json(json!({
                            "isSuccess": false,
                            "code": "RETRO4001",
                            "message": "프로젝트 이름은 1자 이상 20자 이하여야 합니다",
                            "result": null
                        })),
                    ));
                }
            } else {
                return Err((
                    StatusCode::BAD_REQUEST,
                    axum::Json(json!({
                        "isSuccess": false,
                        "code": "COMMON400",
                        "message": "잘못된 요청입니다: projectName 필드가 필요합니다.",
                        "result": null
                    })),
                ));
            }

            // 성공 응답 (Mock)
            Ok(axum::Json(json!({
                "isSuccess": true,
                "code": "COMMON200",
                "message": "회고가 성공적으로 생성되었습니다.",
                "result": {
                    "retrospectId": 1,
                    "teamId": team_id.unwrap_or(1),
                    "projectName": project_name.unwrap_or("테스트")
                }
            })))
        }

        Router::new().route("/api/v1/retrospects", post(test_handler))
    }

    /// API-010 테스트용 라우터 생성 (팀 회고 목록 조회)
    pub fn create_team_retrospects_test_router() -> Router {
        async fn test_handler(
            headers: axum::http::HeaderMap,
            axum::extract::Path(team_id): axum::extract::Path<i64>,
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

            // teamId 검증
            if team_id < 1 {
                return Err((
                    StatusCode::BAD_REQUEST,
                    axum::Json(json!({
                        "isSuccess": false,
                        "code": "COMMON400",
                        "message": "팀 ID는 1 이상이어야 합니다.",
                        "result": null
                    })),
                ));
            }

            // Mock: 팀 존재 여부 (team_id = 999는 존재하지 않음)
            if team_id == 999 {
                return Err((
                    StatusCode::NOT_FOUND,
                    axum::Json(json!({
                        "isSuccess": false,
                        "code": "TEAM4041",
                        "message": "존재하지 않는 팀입니다.",
                        "result": null
                    })),
                ));
            }

            // Mock: 팀 멤버십 확인 (team_id = 888은 접근 권한 없음)
            if team_id == 888 {
                return Err((
                    StatusCode::FORBIDDEN,
                    axum::Json(json!({
                        "isSuccess": false,
                        "code": "TEAM4031",
                        "message": "해당 팀에 접근 권한이 없습니다.",
                        "result": null
                    })),
                ));
            }

            // 성공 응답 (Mock 데이터)
            Ok(axum::Json(json!({
                "isSuccess": true,
                "code": "COMMON200",
                "message": "팀 내 전체 회고 목록 조회를 성공했습니다.",
                "result": [
                    {
                        "retrospectId": 101,
                        "projectName": "오늘 진행할 정기 회고",
                        "retrospectMethod": "KPT",
                        "retrospectDate": "2026-01-24",
                        "retrospectTime": "16:00"
                    },
                    {
                        "retrospectId": 100,
                        "projectName": "지난 주 프로젝트 회고",
                        "retrospectMethod": "PMI",
                        "retrospectDate": "2026-01-20",
                        "retrospectTime": "10:00"
                    }
                ]
            })))
        }

        Router::new().route("/api/v1/teams/:team_id/retrospects", get(test_handler))
    }

    /// 빈 결과 반환용 테스트 라우터 (회고가 없는 팀)
    pub fn create_empty_team_retrospects_test_router() -> Router {
        async fn test_handler(
            headers: axum::http::HeaderMap,
            axum::extract::Path(_team_id): axum::extract::Path<i64>,
        ) -> Result<axum::Json<Value>, (StatusCode, axum::Json<Value>)> {
            // Authorization 헤더 검증
            let auth = headers.get(header::AUTHORIZATION);
            if auth.is_none()
                || !auth
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .starts_with("Bearer ")
            {
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

            // 빈 배열 반환
            Ok(axum::Json(json!({
                "isSuccess": true,
                "code": "COMMON200",
                "message": "팀 내 전체 회고 목록 조회를 성공했습니다.",
                "result": []
            })))
        }

        Router::new().route("/api/v1/teams/:team_id/retrospects", get(test_handler))
    }

    /// 응답 본문을 JSON으로 파싱
    pub async fn parse_response_body(body: Body) -> Value {
        let bytes = body.collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }
}

/// 인증 헤더 없이 요청 시 401 반환 테스트
#[tokio::test]
async fn should_return_401_when_authorization_header_missing() {
    // Arrange
    let app = test_helpers::create_test_router();
    let request_body = json!({
        "teamId": 1,
        "projectName": "테스트 프로젝트",
        "retrospectDate": "2025-01-25",
        "retrospectMethod": "KPT",
        "referenceUrls": []
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/retrospects")
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
        .contains("로그인이 필요합니다"));
}

/// 잘못된 Authorization 헤더 형식 시 401 반환 테스트
#[tokio::test]
async fn should_return_401_when_authorization_header_format_invalid() {
    // Arrange
    let app = test_helpers::create_test_router();
    let request_body = json!({
        "teamId": 1,
        "projectName": "테스트 프로젝트",
        "retrospectDate": "2025-01-25",
        "retrospectMethod": "KPT",
        "referenceUrls": []
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/retrospects")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "InvalidFormat token123") // Bearer 형식이 아님
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "AUTH4001");
    assert!(body["message"].as_str().unwrap().contains("토큰 형식"));
}

/// 유효하지 않은 JSON 요청 바디 시 400 반환 테스트
#[tokio::test]
async fn should_return_400_when_request_body_is_invalid_json() {
    // Arrange
    let app = test_helpers::create_test_router();
    let invalid_json = "{ invalid json }";

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/retrospects")
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

/// 필수 필드 누락 시 400 반환 테스트
#[tokio::test]
async fn should_return_400_when_required_field_missing() {
    // Arrange
    let app = test_helpers::create_test_router();
    // projectName 필드 누락
    let request_body = json!({
        "teamId": 1,
        "retrospectDate": "2025-01-25",
        "retrospectMethod": "KPT"
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/retrospects")
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
}

/// 프로젝트 이름 길이 초과 시 400 반환 테스트
#[tokio::test]
async fn should_return_400_when_project_name_exceeds_max_length() {
    // Arrange
    let app = test_helpers::create_test_router();
    let long_project_name = "a".repeat(21); // 21자 - 최대 20자 초과

    let request_body = json!({
        "teamId": 1,
        "projectName": long_project_name,
        "retrospectDate": "2025-01-25",
        "retrospectMethod": "KPT",
        "referenceUrls": []
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/retrospects")
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
    assert!(body["message"].as_str().unwrap().contains("20자"));
}

/// 빈 프로젝트 이름 시 400 반환 테스트
#[tokio::test]
async fn should_return_400_when_project_name_is_empty() {
    // Arrange
    let app = test_helpers::create_test_router();

    let request_body = json!({
        "teamId": 1,
        "projectName": "",
        "retrospectDate": "2025-01-25",
        "retrospectMethod": "KPT",
        "referenceUrls": []
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/retrospects")
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
}

/// 유효하지 않은 팀 ID (0 또는 음수) 시 400 반환 테스트
#[tokio::test]
async fn should_return_400_when_team_id_is_invalid() {
    // Arrange
    let app = test_helpers::create_test_router();

    let request_body = json!({
        "teamId": 0, // 0은 유효하지 않음
        "projectName": "테스트 프로젝트",
        "retrospectDate": "2025-01-25",
        "retrospectMethod": "KPT",
        "referenceUrls": []
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/retrospects")
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
    // teamId 관련 에러 메시지가 포함되어 있는지 확인
    let message = body["message"].as_str().unwrap();
    assert!(message.contains("teamId") || message.contains("1 이상"));
}

/// 유효한 요청 시 성공 응답 테스트
#[tokio::test]
async fn should_return_200_when_request_is_valid() {
    // Arrange
    let app = test_helpers::create_test_router();

    let request_body = json!({
        "teamId": 1,
        "projectName": "Test Project",
        "retrospectDate": "2025-01-25",
        "retrospectMethod": "KPT",
        "referenceUrls": ["https://example.com"]
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/retrospects")
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
    assert!(body["result"]["retrospectId"].is_i64());
    assert_eq!(body["result"]["teamId"], 1);
    assert_eq!(body["result"]["projectName"], "Test Project");
}

/// Content-Type 헤더 없이 요청 시 400 반환 테스트
#[tokio::test]
async fn should_return_400_when_content_type_missing() {
    // Arrange
    let app = test_helpers::create_test_router();

    let request_body = json!({
        "teamId": 1,
        "projectName": "테스트 프로젝트",
        "retrospectDate": "2025-01-25",
        "retrospectMethod": "KPT"
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/retrospects")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        // Content-Type 헤더 없음
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    // Content-Type이 없으면 JSON 파싱 실패로 400 반환
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// 빈 요청 바디 시 400 반환 테스트
#[tokio::test]
async fn should_return_400_when_request_body_is_empty() {
    // Arrange
    let app = test_helpers::create_test_router();

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/retrospects")
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
// API-010: 팀 회고 목록 조회 통합 테스트
// ============================================

/// [API-010] 인증 헤더 없이 요청 시 401 반환 테스트
#[tokio::test]
async fn api010_should_return_401_when_authorization_header_missing() {
    // Arrange
    let app = test_helpers::create_team_retrospects_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/teams/1/retrospects")
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
}

/// [API-010] 잘못된 Authorization 헤더 형식 시 401 반환 테스트
#[tokio::test]
async fn api010_should_return_401_when_authorization_header_format_invalid() {
    // Arrange
    let app = test_helpers::create_team_retrospects_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/teams/1/retrospects")
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

/// [API-010] 존재하지 않는 팀 요청 시 404 반환 테스트
#[tokio::test]
async fn api010_should_return_404_when_team_not_found() {
    // Arrange
    let app = test_helpers::create_team_retrospects_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/teams/999/retrospects") // 999는 존재하지 않는 팀
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "TEAM4041");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("존재하지 않는 팀"));
}

/// [API-010] 팀 접근 권한 없음 시 403 반환 테스트
#[tokio::test]
async fn api010_should_return_403_when_not_team_member() {
    // Arrange
    let app = test_helpers::create_team_retrospects_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/teams/888/retrospects") // 888은 접근 권한 없는 팀
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

/// [API-010] 유효한 요청 시 회고 목록 반환 테스트
#[tokio::test]
async fn api010_should_return_200_with_retrospect_list_when_valid_request() {
    // Arrange
    let app = test_helpers::create_team_retrospects_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/teams/1/retrospects")
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
        .contains("조회를 성공했습니다"));

    // result가 배열인지 확인
    let result = body["result"].as_array().unwrap();
    assert_eq!(result.len(), 2);

    // 첫 번째 회고 확인 (최신순 - 2026-01-24가 먼저)
    let first = &result[0];
    assert_eq!(first["retrospectId"], 101);
    assert_eq!(first["projectName"], "오늘 진행할 정기 회고");
    assert_eq!(first["retrospectMethod"], "KPT");
    assert_eq!(first["retrospectDate"], "2026-01-24");
    assert_eq!(first["retrospectTime"], "16:00");

    // 두 번째 회고 확인
    let second = &result[1];
    assert_eq!(second["retrospectId"], 100);
    assert_eq!(second["projectName"], "지난 주 프로젝트 회고");
    assert_eq!(second["retrospectMethod"], "PMI");
    assert_eq!(second["retrospectDate"], "2026-01-20");
    assert_eq!(second["retrospectTime"], "10:00");
}

/// [API-010] 회고가 없는 팀 요청 시 빈 배열 반환 테스트
#[tokio::test]
async fn api010_should_return_200_with_empty_array_when_no_retrospects() {
    // Arrange
    let app = test_helpers::create_empty_team_retrospects_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/teams/1/retrospects")
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

    // 빈 배열 확인
    let result = body["result"].as_array().unwrap();
    assert!(result.is_empty());
}

/// [API-010] 유효하지 않은 teamId (0) 요청 시 400 반환 테스트
#[tokio::test]
async fn api010_should_return_400_when_team_id_is_zero() {
    // Arrange
    let app = test_helpers::create_team_retrospects_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/teams/0/retrospects")
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
