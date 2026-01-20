use axum::{extract::State, Json};
use tracing::info;
use validator::Validate;

use crate::utils::{AppError, BaseResponse};

use super::dto::{RefineRequest, RefineResponse};
use super::service::AiService;

/// 애플리케이션 상태
#[derive(Clone)]
pub struct AppState {
    pub ai_service: AiService,
}

/// POST /api/ai/retrospective/refine
///
/// 회고 말투 정제 API
#[utoipa::path(
    post,
    path = "/api/ai/retrospective/refine",
    request_body = RefineRequest,
    responses(
        (status = 200, description = "성공", body = BaseResponse<RefineResponse>),
        (status = 400, description = "잘못된 요청 (유효성 검증 실패)", body = ErrorResponse),
        (status = 401, description = "인증 실패 (유효하지 않은 비밀 키)", body = ErrorResponse),
        (status = 500, description = "서버 에러", body = ErrorResponse)
    ),
    tag = "AI"
)]
pub async fn refine_retrospective(
    State(state): State<AppState>,
    Json(request): Json<RefineRequest>,
) -> Result<Json<BaseResponse<RefineResponse>>, AppError> {
    info!("Received refine request");

    // 유효성 검증
    request.validate()?;

    // 서비스 호출
    let response = state.ai_service.refine_content(&request).await?;

    info!("Refine request completed successfully");
    Ok(Json(BaseResponse::success(response)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ai::dto::ToneStyle;

    #[test]
    fn should_validate_refine_request_with_valid_data() {
        // Arrange
        let request = RefineRequest {
            content: "오늘 힘들었음".to_string(),
            tone_style: ToneStyle::Kind,
            secret_key: "test-secret".to_string(),
        };

        // Act
        let result = request.validate();

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_reject_empty_content() {
        // Arrange
        let request = RefineRequest {
            content: "".to_string(),
            tone_style: ToneStyle::Kind,
            secret_key: "test-secret".to_string(),
        };

        // Act
        let result = request.validate();

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn should_reject_empty_secret_key() {
        // Arrange
        let request = RefineRequest {
            content: "오늘 힘들었음".to_string(),
            tone_style: ToneStyle::Kind,
            secret_key: "".to_string(),
        };

        // Act
        let result = request.validate();

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn should_reject_content_exceeding_max_length() {
        // Arrange
        let long_content = "가".repeat(5001);
        let request = RefineRequest {
            content: long_content,
            tone_style: ToneStyle::Kind,
            secret_key: "test-secret".to_string(),
        };

        // Act
        let result = request.validate();

        // Assert
        assert!(result.is_err());
    }
}
