use std::sync::Arc;

use crate::error::AppError;
use crate::global::validator::SecretKeyValidator;

use super::client::{
    build_system_message, build_user_message, AiClient, AiClientTrait, OpenAiClient,
};
use super::dto::{GuideResponse, RefineResponse, ToneStyle};
use super::prompt::{
    get_refine_few_shot_examples, get_refine_system_prompt, GUIDE_FEW_SHOT_EXAMPLES,
    GUIDE_SYSTEM_PROMPT,
};
use super::retry::with_retry;

/// AI 서비스
///
/// AiClient를 통해 실제 OpenAI 클라이언트 또는 Mock 클라이언트를 주입할 수 있습니다.
#[derive(Clone)]
pub struct AiService {
    client: AiClient,
    secret_key_validator: SecretKeyValidator,
}

impl AiService {
    /// 실제 OpenAI 클라이언트로 서비스 생성
    pub fn new(api_key: &str, secret_key_validator: SecretKeyValidator) -> Self {
        Self {
            client: Arc::new(OpenAiClient::new(api_key)),
            secret_key_validator,
        }
    }

    /// 커스텀 클라이언트로 서비스 생성 (테스트용)
    pub fn with_client<C: AiClientTrait + 'static>(
        client: C,
        secret_key_validator: SecretKeyValidator,
    ) -> Self {
        Self {
            client: Arc::new(client),
            secret_key_validator,
        }
    }

    #[tracing::instrument(
        skip(self, secret_key),
        fields(content_length = content.len())
    )]
    pub async fn provide_guide(
        &self,
        content: &str,
        secret_key: &str,
    ) -> Result<GuideResponse, AppError> {
        // Secret Key 검증
        self.secret_key_validator.validate(secret_key)?;
        tracing::debug!("Secret key validated");

        // 메시지 구성
        let messages = vec![
            build_system_message(GUIDE_SYSTEM_PROMPT)?,
            build_user_message(GUIDE_FEW_SHOT_EXAMPLES)?,
            build_user_message(&format!("User: \"{}\"", content))?,
        ];

        tracing::debug!("Calling OpenAI API for guide generation");

        // OpenAI API 호출 (재시도 로직 적용)
        let guide_message = with_retry(|| {
            let client = self.client.clone();
            let messages = messages.clone();
            async move { client.complete(messages).await }
        })
        .await?;

        tracing::debug!(
            response_length = guide_message.len(),
            "OpenAI response received"
        );

        Ok(GuideResponse {
            current_content: content.to_string(),
            guide_message,
        })
    }

    #[tracing::instrument(
        skip(self, secret_key),
        fields(content_length = content.len(), tone_style = %tone_style)
    )]
    pub async fn refine_retrospective(
        &self,
        content: &str,
        tone_style: ToneStyle,
        secret_key: &str,
    ) -> Result<RefineResponse, AppError> {
        // Secret Key 검증
        self.secret_key_validator.validate(secret_key)?;
        tracing::debug!("Secret key validated");

        // 프롬프트 가져오기
        let system_prompt = get_refine_system_prompt(tone_style);
        let few_shot_examples = get_refine_few_shot_examples(tone_style);

        // 메시지 구성
        let messages = vec![
            build_system_message(&system_prompt)?,
            build_user_message(few_shot_examples)?,
            build_user_message(&format!("User: \"{}\"", content))?,
        ];

        tracing::debug!("Calling OpenAI API for content refinement");

        // OpenAI API 호출 (재시도 로직 적용)
        let mut refined_content = with_retry(|| {
            let client = self.client.clone();
            let messages = messages.clone();
            async move { client.complete(messages).await }
        })
        .await?;

        // 후처리: "Assistant: " 접두사 및 따옴표 제거
        refined_content = refined_content
            .trim_start_matches("Assistant:")
            .trim()
            .trim_matches('"')
            .to_string();

        tracing::debug!(
            response_length = refined_content.len(),
            "OpenAI response processed"
        );

        Ok(RefineResponse {
            original_content: content.to_string(),
            refined_content,
            tone_style: tone_style.to_string(),
        })
    }

    /// OpenAI API 연결 상태 확인
    ///
    /// 모델 목록을 조회하는 경량 API 호출로 연결 상태를 확인합니다.
    pub async fn check_connectivity(&self) -> Result<(), AppError> {
        self.client.check_connectivity().await
    }

    /// 헬스체크용 최소 텍스트 생성
    ///
    /// 실제 텍스트 생성이 가능한지 검증합니다.
    pub async fn health_check(&self) -> Result<String, AppError> {
        self.client.health_check().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ai::client::MockAiClientTrait;

    #[test]
    fn ai_service_should_create_with_api_key() {
        let validator = SecretKeyValidator::new("test-key".to_string());
        let service = AiService::new("sk-test-key", validator);

        // Service가 생성되었는지 확인 (client는 private이므로 직접 접근 불가)
        assert!(std::mem::size_of_val(&service) > 0);
    }

    #[tokio::test]
    async fn provide_guide_should_fail_with_invalid_secret_key() {
        let validator = SecretKeyValidator::new("correct-key".to_string());
        let service = AiService::new("sk-test-key", validator);

        let result = service.provide_guide("test content", "wrong-key").await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::InvalidSecretKey));
    }

    #[tokio::test]
    async fn refine_retrospective_should_fail_with_invalid_secret_key() {
        let validator = SecretKeyValidator::new("correct-key".to_string());
        let service = AiService::new("sk-test-key", validator);

        let result = service
            .refine_retrospective("test content", ToneStyle::Kind, "wrong-key")
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::InvalidSecretKey));
    }

    #[tokio::test]
    async fn refine_retrospective_should_fail_with_invalid_secret_key_polite() {
        let validator = SecretKeyValidator::new("correct-key".to_string());
        let service = AiService::new("sk-test-key", validator);

        let result = service
            .refine_retrospective("test content", ToneStyle::Polite, "wrong-key")
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::InvalidSecretKey));
    }

    // ===== Mock 기반 성공 케이스 테스트 =====

    fn create_mock_client_with_response(response: &'static str) -> MockAiClientTrait {
        let mut mock = MockAiClientTrait::new();
        mock.expect_complete()
            .returning(move |_| Ok(response.to_string()));
        mock.expect_check_connectivity().returning(|| Ok(()));
        mock.expect_health_check().returning(|| Ok("ok".to_string()));
        mock
    }

    #[tokio::test]
    async fn should_provide_guide_successfully() {
        // Arrange
        let mock = create_mock_client_with_response(
            "좋은 시작이에요! 어떤 기술을 배우셨나요? 더 구체적으로 작성해보세요.",
        );
        let validator = SecretKeyValidator::new("valid-key".to_string());
        let service = AiService::with_client(mock, validator);

        // Act
        let result = service
            .provide_guide("오늘 프로젝트를 진행하면서...", "valid-key")
            .await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.current_content, "오늘 프로젝트를 진행하면서...");
        assert!(!response.guide_message.is_empty());
        assert!(response.guide_message.contains("좋은 시작이에요!"));
    }

    #[tokio::test]
    async fn should_refine_with_kind_style() {
        // Arrange
        let mock = create_mock_client_with_response("오늘 일이 많이 힘들었어요.");
        let validator = SecretKeyValidator::new("valid-key".to_string());
        let service = AiService::with_client(mock, validator);

        // Act
        let result = service
            .refine_retrospective("오늘 일 힘들었음", ToneStyle::Kind, "valid-key")
            .await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.original_content, "오늘 일 힘들었음");
        assert_eq!(response.refined_content, "오늘 일이 많이 힘들었어요.");
        assert_eq!(response.tone_style, "KIND");
    }

    #[tokio::test]
    async fn should_refine_with_polite_style() {
        // Arrange
        let mock = create_mock_client_with_response("오늘 일이 많이 힘들었습니다.");
        let validator = SecretKeyValidator::new("valid-key".to_string());
        let service = AiService::with_client(mock, validator);

        // Act
        let result = service
            .refine_retrospective("오늘 일 힘들었음", ToneStyle::Polite, "valid-key")
            .await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.original_content, "오늘 일 힘들었음");
        assert_eq!(response.refined_content, "오늘 일이 많이 힘들었습니다.");
        assert_eq!(response.tone_style, "POLITE");
    }

    #[tokio::test]
    async fn should_handle_empty_response() {
        // Arrange
        let mock = create_mock_client_with_response("");
        let validator = SecretKeyValidator::new("valid-key".to_string());
        let service = AiService::with_client(mock, validator);

        // Act
        let result = service.provide_guide("test content", "valid-key").await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.guide_message.is_empty());
    }

    #[tokio::test]
    async fn should_handle_long_response() {
        // Arrange
        let long_response = "가".repeat(5000);
        let mock = {
            let mut m = MockAiClientTrait::new();
            let response_clone = long_response.clone();
            m.expect_complete()
                .returning(move |_| Ok(response_clone.clone()));
            m.expect_check_connectivity().returning(|| Ok(()));
            m.expect_health_check().returning(|| Ok("ok".to_string()));
            m
        };
        let validator = SecretKeyValidator::new("valid-key".to_string());
        let service = AiService::with_client(mock, validator);

        // Act
        let result = service.provide_guide("test content", "valid-key").await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.guide_message.len(), 5000 * 3); // UTF-8 한글 = 3 bytes
    }

    #[tokio::test]
    async fn should_strip_assistant_prefix_from_refine_response() {
        // Arrange
        let mock = create_mock_client_with_response("Assistant: 정제된 내용입니다.");
        let validator = SecretKeyValidator::new("valid-key".to_string());
        let service = AiService::with_client(mock, validator);

        // Act
        let result = service
            .refine_retrospective("원본", ToneStyle::Kind, "valid-key")
            .await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.refined_content, "정제된 내용입니다.");
    }

    #[tokio::test]
    async fn should_strip_quotes_from_refine_response() {
        // Arrange
        let mock = create_mock_client_with_response("\"정제된 내용입니다.\"");
        let validator = SecretKeyValidator::new("valid-key".to_string());
        let service = AiService::with_client(mock, validator);

        // Act
        let result = service
            .refine_retrospective("원본", ToneStyle::Kind, "valid-key")
            .await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.refined_content, "정제된 내용입니다.");
    }

    #[tokio::test]
    async fn should_fail_when_openai_returns_error() {
        // Arrange
        let mut mock = MockAiClientTrait::new();
        mock.expect_complete()
            .returning(|_| Err(AppError::OpenAiError("API Error".to_string())));
        mock.expect_check_connectivity().returning(|| Ok(()));
        mock.expect_health_check().returning(|| Ok("ok".to_string()));

        let validator = SecretKeyValidator::new("valid-key".to_string());
        let service = AiService::with_client(mock, validator);

        // Act
        let result = service.provide_guide("test content", "valid-key").await;

        // Assert
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::OpenAiError(_)));
    }

    #[tokio::test]
    async fn should_check_connectivity_successfully() {
        // Arrange
        let mut mock = MockAiClientTrait::new();
        mock.expect_complete().returning(|_| Ok("".to_string()));
        mock.expect_check_connectivity().returning(|| Ok(()));
        mock.expect_health_check().returning(|| Ok("ok".to_string()));

        let validator = SecretKeyValidator::new("valid-key".to_string());
        let service = AiService::with_client(mock, validator);

        // Act
        let result = service.check_connectivity().await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_fail_connectivity_check_when_api_unavailable() {
        // Arrange
        let mut mock = MockAiClientTrait::new();
        mock.expect_complete().returning(|_| Ok("".to_string()));
        mock.expect_check_connectivity()
            .returning(|| Err(AppError::OpenAiError("Connection failed".to_string())));
        mock.expect_health_check().returning(|| Ok("ok".to_string()));

        let validator = SecretKeyValidator::new("valid-key".to_string());
        let service = AiService::with_client(mock, validator);

        // Act
        let result = service.check_connectivity().await;

        // Assert
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::OpenAiError(_)));
    }

    #[tokio::test]
    async fn should_perform_health_check_successfully() {
        // Arrange
        let mut mock = MockAiClientTrait::new();
        mock.expect_complete().returning(|_| Ok("".to_string()));
        mock.expect_check_connectivity().returning(|| Ok(()));
        mock.expect_health_check().returning(|| Ok("ok".to_string()));

        let validator = SecretKeyValidator::new("valid-key".to_string());
        let service = AiService::with_client(mock, validator);

        // Act
        let result = service.health_check().await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "ok");
    }

    #[tokio::test]
    async fn should_fail_health_check_when_api_error() {
        // Arrange
        let mut mock = MockAiClientTrait::new();
        mock.expect_complete().returning(|_| Ok("".to_string()));
        mock.expect_check_connectivity().returning(|| Ok(()));
        mock.expect_health_check()
            .returning(|| Err(AppError::OpenAiError("Health check failed".to_string())));

        let validator = SecretKeyValidator::new("valid-key".to_string());
        let service = AiService::with_client(mock, validator);

        // Act
        let result = service.health_check().await;

        // Assert
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::OpenAiError(_)));
    }
}
