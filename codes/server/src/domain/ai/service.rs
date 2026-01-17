use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
    Client,
};
use tracing::{info, instrument};

use crate::config::AppConfig;
use crate::utils::AppError;

use super::dto::{RefineRequest, RefineResponse};
use super::prompt::RefinePrompt;

/// AI 서비스
#[derive(Clone)]
pub struct AiService {
    client: Client<OpenAIConfig>,
    secret_key: String,
}

impl AiService {
    /// 새 AiService 인스턴스 생성
    pub fn new(config: &AppConfig) -> Self {
        let openai_config = OpenAIConfig::new().with_api_key(&config.openai_api_key);
        let client = Client::with_config(openai_config);

        Self {
            client,
            secret_key: config.secret_key.clone(),
        }
    }

    /// 비밀 키 검증
    pub fn validate_secret_key(&self, provided_key: &str) -> Result<(), AppError> {
        if provided_key != self.secret_key {
            return Err(AppError::InvalidSecretKey);
        }
        Ok(())
    }

    /// 회고 내용 정제
    #[instrument(skip(self, request), fields(tone_style = ?request.tone_style))]
    pub async fn refine_content(&self, request: &RefineRequest) -> Result<RefineResponse, AppError> {
        // 비밀 키 검증
        self.validate_secret_key(&request.secret_key)?;

        info!("Refining content with tone style: {:?}", request.tone_style);

        // 프롬프트 생성
        let system_prompt = RefinePrompt::system_prompt(request.tone_style);
        let user_prompt = RefinePrompt::user_prompt(&request.content);

        // OpenAI API 호출
        let refined_content = self
            .call_openai(&system_prompt, &user_prompt)
            .await?;

        Ok(RefineResponse::new(
            request.content.clone(),
            refined_content,
            request.tone_style,
        ))
    }

    /// OpenAI API 호출
    async fn call_openai(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, AppError> {
        let messages: Vec<ChatCompletionRequestMessage> = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system_prompt)
                .build()
                .map_err(|e| AppError::AiGeneralError(e.to_string()))?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(user_prompt)
                .build()
                .map_err(|e| AppError::AiGeneralError(e.to_string()))?
                .into(),
        ];

        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-4o-mini")
            .messages(messages)
            .temperature(0.7)
            .max_tokens(2000u32)
            .build()
            .map_err(|e| AppError::AiGeneralError(e.to_string()))?;

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(|e| {
                let error_msg = e.to_string();
                if error_msg.contains("401") || error_msg.contains("Unauthorized") {
                    AppError::AiConnectionFailed("API 키가 유효하지 않습니다".to_string())
                } else if error_msg.contains("429") || error_msg.contains("rate limit") {
                    AppError::AiServiceUnavailable("요청 한도 초과".to_string())
                } else if error_msg.contains("503") || error_msg.contains("unavailable") {
                    AppError::AiServiceUnavailable(error_msg)
                } else {
                    AppError::AiGeneralError(error_msg)
                }
            })?;

        let content = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .ok_or_else(|| AppError::AiGeneralError("AI 응답이 비어있습니다".to_string()))?;

        info!("AI response received successfully");
        Ok(content)
    }
}

/// 테스트용 Mock Service
#[cfg(test)]
pub mod mock {
    use super::*;
    use super::super::dto::ToneStyle;

    pub struct MockAiService {
        secret_key: String,
    }

    impl MockAiService {
        pub fn new(secret_key: &str) -> Self {
            Self {
                secret_key: secret_key.to_string(),
            }
        }

        pub fn validate_secret_key(&self, provided_key: &str) -> Result<(), AppError> {
            if provided_key != self.secret_key {
                return Err(AppError::InvalidSecretKey);
            }
            Ok(())
        }

        pub async fn refine_content(&self, request: &RefineRequest) -> Result<RefineResponse, AppError> {
            self.validate_secret_key(&request.secret_key)?;

            // Mock: 간단한 변환 로직
            let refined = match request.tone_style {
                ToneStyle::Kind => format!("{}요~", request.content.trim_end_matches(|c: char| c == '.' || c == '요')),
                ToneStyle::Polite => format!("{}습니다.", request.content.trim_end_matches(|c: char| c == '.' || c == '요' || c == '음')),
            };

            Ok(RefineResponse::new(
                request.content.clone(),
                refined,
                request.tone_style,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::MockAiService;
    use super::*;
    use super::super::dto::ToneStyle;

    #[test]
    fn should_validate_correct_secret_key() {
        // Arrange
        let service = MockAiService::new("test-secret");

        // Act
        let result = service.validate_secret_key("test-secret");

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_reject_invalid_secret_key() {
        // Arrange
        let service = MockAiService::new("test-secret");

        // Act
        let result = service.validate_secret_key("wrong-secret");

        // Assert
        assert!(matches!(result, Err(AppError::InvalidSecretKey)));
    }

    #[tokio::test]
    async fn should_refine_content_with_kind_tone() {
        // Arrange
        let service = MockAiService::new("test-secret");
        let request = RefineRequest {
            content: "오늘 힘들었음".to_string(),
            tone_style: ToneStyle::Kind,
            secret_key: "test-secret".to_string(),
        };

        // Act
        let result = service.refine_content(&request).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.original_content, "오늘 힘들었음");
        assert_eq!(response.tone_style, ToneStyle::Kind);
    }

    #[tokio::test]
    async fn should_reject_refine_with_invalid_secret_key() {
        // Arrange
        let service = MockAiService::new("test-secret");
        let request = RefineRequest {
            content: "오늘 힘들었음".to_string(),
            tone_style: ToneStyle::Kind,
            secret_key: "wrong-secret".to_string(),
        };

        // Act
        let result = service.refine_content(&request).await;

        // Assert
        assert!(matches!(result, Err(AppError::InvalidSecretKey)));
    }
}
