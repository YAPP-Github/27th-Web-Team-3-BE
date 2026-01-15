use std::sync::Arc;
use std::time::Duration;

use async_openai::{
    config::OpenAIConfig,
    error::OpenAIError,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
    Client,
};

use crate::error::AppError;

/// OpenAI 호출 타임아웃 (초)
const OPENAI_TIMEOUT_SECS: u64 = 25;

/// OpenAI 에러를 세분화된 AppError로 변환
fn classify_openai_error(error: OpenAIError) -> AppError {
    match &error {
        OpenAIError::ApiError(api_err) => {
            // API 에러 타입 기반 분류
            let err_type = api_err.r#type.as_deref().unwrap_or("");
            let message = &api_err.message;

            // 에러 코드가 JSON Value일 수 있으므로 문자열로 변환
            let err_code = api_err
                .code
                .as_ref()
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if err_type == "invalid_request_error"
                && (err_code == "invalid_api_key" || message.contains("API key"))
            {
                AppError::OpenAiAuthError
            } else if err_type == "rate_limit_error"
                || err_code == "rate_limit_exceeded"
                || message.contains("rate limit")
            {
                AppError::OpenAiRateLimitError
            } else if err_type == "server_error"
                || err_code.contains("server")
                || message.contains("server")
            {
                AppError::OpenAiTemporaryError
            } else {
                AppError::OpenAiError(message.clone())
            }
        }
        OpenAIError::Reqwest(req_err) => {
            // HTTP 요청 에러 분류
            if req_err.is_timeout() || req_err.is_connect() {
                AppError::OpenAiTemporaryError
            } else if req_err.status().map(|s| s.as_u16()) == Some(401) {
                AppError::OpenAiAuthError
            } else if req_err.status().map(|s| s.as_u16()) == Some(429) {
                AppError::OpenAiRateLimitError
            } else if req_err
                .status()
                .map(|s| s.is_server_error())
                .unwrap_or(false)
            {
                AppError::OpenAiTemporaryError
            } else {
                AppError::OpenAiError(req_err.to_string())
            }
        }
        _ => AppError::OpenAiError(error.to_string()),
    }
}

/// AI 클라이언트 인터페이스
///
/// OpenAI API 호출을 추상화하여 테스트에서 Mock 객체로 대체할 수 있습니다.
#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait AiClientTrait: Send + Sync {
    /// 채팅 완성 요청
    async fn complete(
        &self,
        messages: Vec<ChatCompletionRequestMessage>,
    ) -> Result<String, AppError>;

    /// API 연결 상태 확인 (모델 목록 조회)
    async fn check_connectivity(&self) -> Result<(), AppError>;

    /// 헬스체크용 최소 텍스트 생성
    ///
    /// 실제 텍스트 생성이 가능한지 검증합니다.
    /// 최소 토큰으로 호출하여 비용을 절감합니다.
    async fn health_check(&self) -> Result<String, AppError>;
}

/// Arc로 래핑된 AiClient (Clone 지원)
pub type AiClient = Arc<dyn AiClientTrait>;

/// OpenAI API 클라이언트 구현체
#[derive(Clone)]
pub struct OpenAiClient {
    client: Client<OpenAIConfig>,
}

impl OpenAiClient {
    pub fn new(api_key: &str) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key);
        Self {
            client: Client::with_config(config),
        }
    }
}

#[async_trait::async_trait]
impl AiClientTrait for OpenAiClient {
    async fn complete(
        &self,
        messages: Vec<ChatCompletionRequestMessage>,
    ) -> Result<String, AppError> {
        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-4o-mini")
            .messages(messages)
            .build()
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // OpenAI 호출 타임아웃 적용 (25초)
        let response = tokio::time::timeout(
            Duration::from_secs(OPENAI_TIMEOUT_SECS),
            self.client.chat().create(request),
        )
        .await
        .map_err(|_| AppError::OpenAiTemporaryError)? // 타임아웃
        .map_err(classify_openai_error)?; // OpenAI 에러

        Ok(response.choices[0]
            .message
            .content
            .clone()
            .unwrap_or_default())
    }

    async fn check_connectivity(&self) -> Result<(), AppError> {
        self.client
            .models()
            .list()
            .await
            .map_err(classify_openai_error)?;
        Ok(())
    }

    async fn health_check(&self) -> Result<String, AppError> {
        // 최소 토큰으로 실제 텍스트 생성 검증
        let messages = vec![ChatCompletionRequestMessage::System(
            ChatCompletionRequestSystemMessageArgs::default()
                .content("Respond with exactly 'ok'")
                .build()
                .map_err(|e| AppError::Internal(e.to_string()))?,
        )];

        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-4o-mini")
            .messages(messages)
            .max_tokens(5_u16) // 최소 토큰 사용
            .build()
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let response = tokio::time::timeout(
            Duration::from_secs(5), // 헬스체크용 짧은 타임아웃
            self.client.chat().create(request),
        )
        .await
        .map_err(|_| AppError::OpenAiTemporaryError)?
        .map_err(classify_openai_error)?;

        Ok(response.choices[0]
            .message
            .content
            .clone()
            .unwrap_or_default())
    }
}

/// 메시지 빌더 헬퍼 함수 (crate 내부용)
pub(crate) fn build_system_message(content: &str) -> Result<ChatCompletionRequestMessage, AppError> {
    Ok(ChatCompletionRequestMessage::System(
        ChatCompletionRequestSystemMessageArgs::default()
            .content(content)
            .build()
            .map_err(|e| AppError::Internal(e.to_string()))?,
    ))
}

pub(crate) fn build_user_message(content: &str) -> Result<ChatCompletionRequestMessage, AppError> {
    Ok(ChatCompletionRequestMessage::User(
        ChatCompletionRequestUserMessageArgs::default()
            .content(content)
            .build()
            .map_err(|e| AppError::Internal(e.to_string()))?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_openai_client() {
        let client = OpenAiClient::new("test-api-key");
        assert!(std::mem::size_of_val(&client) > 0);
    }

    #[test]
    fn should_build_system_message() {
        let result = build_system_message("test prompt");
        assert!(result.is_ok());
    }

    #[test]
    fn should_build_user_message() {
        let result = build_user_message("test content");
        assert!(result.is_ok());
    }
}
