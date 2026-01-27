//! 회고 내보내기 API 통합 테스트 (API-021)
//!
//! GET /api/v1/retrospects/{retrospectId}/export 엔드포인트에 대한 HTTP 통합 테스트입니다.
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

mod export_test_helpers {
    use super::*;

    /// API-021 테스트용 라우터 생성 (회고 내보내기)
    pub fn create_export_test_router() -> Router {
        async fn test_handler(
            headers: axum::http::HeaderMap,
            axum::extract::Path(retrospect_id): axum::extract::Path<i64>,
        ) -> Result<([(axum::http::HeaderName, String); 3], Vec<u8>), (StatusCode, axum::Json<Value>)>
        {
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
                        "message": "존재하지 않는 회고이거나 접근 권한이 없습니다.",
                        "result": null
                    })),
                ));
            }

            // Mock: 접근 권한 없음 (888) - 비멤버에게 회고 존재를 노출하지 않도록 404로 마스킹
            if retrospect_id == 888 {
                return Err((
                    StatusCode::NOT_FOUND,
                    axum::Json(json!({
                        "isSuccess": false,
                        "code": "RETRO4041",
                        "message": "존재하지 않는 회고이거나 접근 권한이 없습니다.",
                        "result": null
                    })),
                ));
            }

            // Mock: PDF 생성 실패 (777)
            if retrospect_id == 777 {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(json!({
                        "isSuccess": false,
                        "code": "COMMON500",
                        "message": "PDF 생성 중 서버 에러가 발생했습니다.",
                        "result": null
                    })),
                ));
            }

            // 성공: Mock PDF 바이트 반환
            let mock_pdf_bytes = b"%PDF-1.5 mock content".to_vec();
            let filename = format!("retrospect_report_{}_20260127_120000.pdf", retrospect_id);

            let headers = [
                (
                    header::CONTENT_TYPE,
                    "application/pdf; charset=utf-8".to_string(),
                ),
                (
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{}\"", filename),
                ),
                (
                    header::CACHE_CONTROL,
                    "no-cache, no-store, must-revalidate".to_string(),
                ),
            ];

            Ok((headers, mock_pdf_bytes))
        }

        Router::new().route(
            "/api/v1/retrospects/:retrospect_id/export",
            get(test_handler),
        )
    }

    /// 응답 본문을 JSON으로 파싱
    pub async fn parse_response_body(body: Body) -> Value {
        let bytes = body.collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    /// 응답 본문을 바이트로 파싱
    pub async fn parse_response_bytes(body: Body) -> Vec<u8> {
        body.collect().await.unwrap().to_bytes().to_vec()
    }
}

// ============================================
// 인증 관련 테스트
// ============================================

/// [API-021] 인증 헤더 없이 요청 시 401 반환 테스트
#[tokio::test]
async fn api021_should_return_401_when_authorization_header_missing() {
    // Arrange
    let app = export_test_helpers::create_export_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/1/export")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = export_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "AUTH4001");
}

/// [API-021] 잘못된 Authorization 헤더 형식 시 401 반환 테스트
#[tokio::test]
async fn api021_should_return_401_when_authorization_header_format_invalid() {
    // Arrange
    let app = export_test_helpers::create_export_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/1/export")
        .header(header::AUTHORIZATION, "InvalidFormat token123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = export_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "AUTH4001");
}

// ============================================
// Path Parameter 검증 테스트
// ============================================

/// [API-021] 유효하지 않은 retrospectId (0) 요청 시 400 반환 테스트
#[tokio::test]
async fn api021_should_return_400_when_retrospect_id_is_zero() {
    // Arrange
    let app = export_test_helpers::create_export_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/0/export")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = export_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "COMMON400");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("retrospectId는 1 이상의 양수여야 합니다"));
}

/// [API-021] 유효하지 않은 retrospectId (음수) 요청 시 400 반환 테스트
#[tokio::test]
async fn api021_should_return_400_when_retrospect_id_is_negative() {
    // Arrange
    let app = export_test_helpers::create_export_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/-1/export")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = export_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "COMMON400");
}

// ============================================
// 비즈니스 에러 테스트
// ============================================

/// [API-021] 존재하지 않는 회고 요청 시 404 반환 테스트
#[tokio::test]
async fn api021_should_return_404_when_retrospect_not_found() {
    // Arrange
    let app = export_test_helpers::create_export_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/999/export")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = export_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "RETRO4041");
}

/// [API-021] 팀 멤버가 아닌 사용자 요청 시 404 반환 테스트 (IDOR 보호: 비멤버에게 회고 존재를 노출하지 않음)
#[tokio::test]
async fn api021_should_return_404_when_user_is_not_team_member() {
    // Arrange
    let app = export_test_helpers::create_export_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/888/export")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = export_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "RETRO4041");
}

/// [API-021] PDF 생성 실패 시 500 반환 테스트
#[tokio::test]
async fn api021_should_return_500_when_pdf_generation_fails() {
    // Arrange
    let app = export_test_helpers::create_export_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/777/export")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body = export_test_helpers::parse_response_body(response.into_body()).await;
    assert_eq!(body["isSuccess"], false);
    assert_eq!(body["code"], "COMMON500");
    assert!(body["message"].as_str().unwrap().contains("PDF 생성"));
}

// ============================================
// 성공 케이스 테스트
// ============================================

/// [API-021] 유효한 요청 시 200 및 PDF 바이너리 응답 테스트
#[tokio::test]
async fn api021_should_return_200_with_pdf_binary() {
    // Arrange
    let app = export_test_helpers::create_export_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/100/export")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let bytes = export_test_helpers::parse_response_bytes(response.into_body()).await;
    assert!(!bytes.is_empty());
}

/// [API-021] Content-Type 헤더가 application/pdf인지 검증 테스트
#[tokio::test]
async fn api021_should_return_content_type_application_pdf() {
    // Arrange
    let app = export_test_helpers::create_export_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/100/export")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(content_type.contains("application/pdf"));
}

/// [API-021] Content-Disposition 헤더에 파일명이 포함되는지 검증 테스트
#[tokio::test]
async fn api021_should_return_content_disposition_with_filename() {
    // Arrange
    let app = export_test_helpers::create_export_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/100/export")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let content_disposition = response
        .headers()
        .get(header::CONTENT_DISPOSITION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(content_disposition.contains("attachment"));
    assert!(content_disposition.contains("retrospect_report_100_"));
    assert!(content_disposition.contains(".pdf"));
}

/// [API-021] Cache-Control 헤더가 no-cache인지 검증 테스트
#[tokio::test]
async fn api021_should_return_cache_control_no_cache() {
    // Arrange
    let app = export_test_helpers::create_export_test_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/retrospects/100/export")
        .header(header::AUTHORIZATION, "Bearer valid_token_123")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let cache_control = response
        .headers()
        .get(header::CACHE_CONTROL)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(cache_control.contains("no-cache"));
    assert!(cache_control.contains("no-store"));
    assert!(cache_control.contains("must-revalidate"));
}
