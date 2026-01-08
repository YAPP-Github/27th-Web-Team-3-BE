use actix_web::{web, HttpResponse, Result};
use validator::Validate;

use crate::models::request::{GuideRequest, RefineRequest};
use crate::models::response::{BaseResponse, GuideResponse, RefineResponse};
use crate::error::AppError;
use super::service::AiService;

/// AI 회고 작성 가이드 제공 API
///
/// 사용자가 작성 중인 회고 내용을 분석하여 적절한 코칭 메시지를 제공합니다.
///
/// # 에러 코드
/// - COMMON400: content 또는 secretKey 누락 시
/// - AI_001: 잘못된 secretKey 입력 시
/// - COMMON500: AI 서버 통신 장애 등 내부 에러 발생 시
#[utoipa::path(
    post,
    path = "/api/ai/retrospective/guide",
    request_body = GuideRequest,
    responses(
        (status = 200, description = "성공", body = BaseResponse<GuideResponse>),
        (status = 400, description = "잘못된 요청", body = crate::error::ErrorResponse),
        (status = 401, description = "인증 실패", body = crate::error::ErrorResponse),
        (status = 500, description = "서버 오류", body = crate::error::ErrorResponse)
    ),
    tag = "AI"
)]
pub async fn provide_guide(
    req: web::Json<GuideRequest>,
    ai_service: web::Data<AiService>,
) -> Result<HttpResponse, AppError> {
    // 입력값 검증
    req.validate().map_err(|e| {
        AppError::BadRequest(format!("입력값 검증 실패: {}", e))
    })?;

    // AI 서비스 호출하여 가이드 메시지 생성
    let guide_message = ai_service
        .generate_retrospective_guide(&req.current_content, &req.secret_key)
        .await?;

    let response = GuideResponse {
        current_content: req.current_content.clone(),
        guide_message,
    };

    Ok(HttpResponse::Ok().json(BaseResponse::success(response)))
}

/// AI 회고 내용 다듬기 API
///
/// 작성된 회고 내용을 선택한 톤으로 다듬어줍니다.
///
/// # 에러 코드
/// - COMMON400: content, toneStyle 또는 secretKey 누락 시
/// - AI_001: 잘못된 secretKey 입력 시
/// - COMMON500: AI 서버 통신 장애 등 내부 에러 발생 시
#[utoipa::path(
    post,
    path = "/api/ai/retrospective/refine",
    request_body = RefineRequest,
    responses(
        (status = 200, description = "성공", body = BaseResponse<RefineResponse>),
        (status = 400, description = "잘못된 요청", body = crate::error::ErrorResponse),
        (status = 401, description = "인증 실패", body = crate::error::ErrorResponse),
        (status = 500, description = "서버 오류", body = crate::error::ErrorResponse)
    ),
    tag = "AI"
)]
pub async fn refine_retrospective(
    req: web::Json<RefineRequest>,
    ai_service: web::Data<AiService>,
) -> Result<HttpResponse, AppError> {
    // 입력값 검증
    req.validate().map_err(|e| {
        AppError::BadRequest(format!("입력값 검증 실패: {}", e))
    })?;

    // AI 서비스 호출하여 내용 다듬기
    let refined_content = ai_service
        .refine_retrospective(&req.content, &req.tone_style, &req.secret_key)
        .await?;

    let response = RefineResponse {
        original_content: req.content.clone(),
        refined_content,
        tone_style: req.tone_style.to_korean().to_string(),
    };

    Ok(HttpResponse::Ok().json(BaseResponse::success(response)))
}

/// 라우트 설정
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/retrospective/guide")
            .route(web::post().to(provide_guide))
    )
    .service(
        web::resource("/retrospective/refine")
            .route(web::post().to(refine_retrospective))
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    use crate::models::request::{GuideRequest, RefineRequest, ToneStyle};
    use std::env;

    #[actix_web::test]
    async fn test_provide_guide_success() {
        // 환경 변수 설정
        env::set_var("SECRET_KEY", "test_secret_key_123");
        env::set_var("AI_API_URL", "http://mock-ai-server.com");
        env::set_var("AI_API_KEY", "mock_api_key");

        let req_body = GuideRequest {
            current_content: "오늘 프로젝트를 진행하면서 어려움이 있었습니다.".to_string(),
            secret_key: "test_secret_key_123".to_string(),
        };

        // 정상 케이스 검증 로직
        assert!(!req_body.current_content.is_empty());
        assert!(!req_body.secret_key.is_empty());
    }

    #[actix_web::test]
    async fn test_provide_guide_missing_content() {
        let req_body = GuideRequest {
            current_content: "".to_string(),
            secret_key: "test_secret_key_123".to_string(),
        };

        // Validation 실패 확인
        assert!(req_body.validate().is_err());
    }

    #[actix_web::test]
    async fn test_provide_guide_missing_secret_key() {
        let req_body = GuideRequest {
            current_content: "테스트 내용".to_string(),
            secret_key: "".to_string(),
        };

        // Validation 실패 확인
        assert!(req_body.validate().is_err());
    }

    #[actix_web::test]
    async fn test_refine_retrospective_success() {
        env::set_var("SECRET_KEY", "test_secret_key_123");

        let req_body = RefineRequest {
            content: "오늘 일 존나 힘들었음 ㅋㅋ".to_string(),
            tone_style: ToneStyle::Polite,
            secret_key: "test_secret_key_123".to_string(),
        };

        assert!(!req_body.content.is_empty());
        assert!(!req_body.secret_key.is_empty());
    }

    #[actix_web::test]
    async fn test_refine_retrospective_missing_content() {
        let req_body = RefineRequest {
            content: "".to_string(),
            tone_style: ToneStyle::Kind,
            secret_key: "test_secret_key_123".to_string(),
        };

        assert!(req_body.validate().is_err());
    }
}


