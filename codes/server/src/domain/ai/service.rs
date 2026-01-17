use super::{
    dto::{RetrospectiveGuideRequest, RetrospectiveGuideResult},
    prompt::GUIDE_SYSTEM_PROMPT,
};
use crate::utils::error::AppError;
use async_openai::config::OpenAIConfig;
use async_openai::types::{
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    CreateChatCompletionRequestArgs,
};
use async_openai::Client;
use std::env;

pub struct AiService;

impl AiService {
    pub async fn generate_guide(
        req: RetrospectiveGuideRequest,
    ) -> Result<RetrospectiveGuideResult, AppError> {
        // 1. Secret Key 검증 (서버 인증용)
        let server_secret =
            env::var("APP_SECRET_KEY").unwrap_or_else(|_| "mySecretKey123".to_string());

        if req.secret_key != server_secret {
            return Err(AppError::unauthorized("유효하지 않은 비밀 키입니다."));
        }

        // 2. OpenAI API Key 로드 (OPEN_CHAT_KEY 사용)
        let api_key = env::var("OPEN_CHAT_KEY")
            .map_err(|_| AppError::internal_error("OPEN_CHAT_KEY 환경 변수가 설정되지 않았습니다."))?;

        // 3. OpenAI 클라이언트 설정
        let config = OpenAIConfig::default().with_api_key(api_key);
        let client = Client::with_config(config);

        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-4o")
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(GUIDE_SYSTEM_PROMPT)
                    .build()
                    .map_err(|e| AppError::internal_error(e.to_string()))?
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(req.current_content.clone())
                    .build()
                    .map_err(|e| AppError::internal_error(e.to_string()))?
                    .into(),
            ])
            .build()
            .map_err(|e| AppError::internal_error(e.to_string()))?;

        let response = client
            .chat()
            .create(request)
            .await
            .map_err(|e| AppError::internal_error(format!("OpenAI API 호출 실패: {}", e)))?;

        let guide_message = response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_else(|| "가이드를 생성하지 못했습니다.".to_string());

        Ok(RetrospectiveGuideResult {
            current_content: req.current_content,
            guide_message,
        })
    }
}
