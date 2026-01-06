use async_openai::{
    types::{
        ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessage,
        ChatCompletionRequestUserMessage,
        CreateChatCompletionRequestArgs
    },
    Client,
};

use crate::models::request::ToneStyle;
use crate::domain::ai::prompt::PromptTemplate;
use crate::error::AppError;

pub async fn generate_guide(current_content: &str) -> Result<String, AppError> {
    let client = Client::new();

    let system_message = ChatCompletionRequestSystemMessage {
        content: PromptTemplate::GUIDE_SYSTEM_PROMPT.to_string(),
        role: async_openai::types::Role::System,
        name: None,
    };

    let user_message = ChatCompletionRequestUserMessage {
        content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
            format!("현재 작성 중인 내용:\n{}", current_content)
        ),
        role: async_openai::types::Role::User,
        name: None,
    };

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4o-mini")
        .messages(vec![
            ChatCompletionRequestMessage::System(system_message),
            ChatCompletionRequestMessage::User(user_message),
        ])
        .temperature(0.7)
        .build()
        .map_err(|e| AppError::InternalError(format!("Failed to build request: {}", e)))?;

    let response = client
        .chat()
        .create(request)
        .await
        .map_err(|e| AppError::ExternalApiError(format!("OpenAI API error: {}", e)))?;

    let guide_message = response
        .choices
        .first()
        .and_then(|choice| choice.message.content.clone())
        .ok_or_else(|| AppError::InternalError("No response from OpenAI".to_string()))?;

    Ok(guide_message)
}

pub async fn refine_content(content: &str, tone: &ToneStyle) -> Result<(String, String), AppError> {
    let client = Client::new();

    let system_prompt = PromptTemplate::get_refine_system_prompt(tone);

    let system_message = ChatCompletionRequestSystemMessage {
        content: system_prompt,
        role: async_openai::types::Role::System,
        name: None,
    };

    let user_message = ChatCompletionRequestUserMessage {
        content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
            content.to_string()
        ),
        role: async_openai::types::Role::User,
        name: None,
    };

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4o-mini")
        .messages(vec![
            ChatCompletionRequestMessage::System(system_message),
            ChatCompletionRequestMessage::User(user_message),
        ])
        .temperature(0.3)
        .build()
        .map_err(|e| AppError::InternalError(format!("Failed to build request: {}", e)))?;

    let response = client
        .chat()
        .create(request)
        .await
        .map_err(|e| AppError::ExternalApiError(format!("OpenAI API error: {}", e)))?;

    let refined_content = response
        .choices
        .first()
        .and_then(|choice| choice.message.content.clone())
        .ok_or_else(|| AppError::InternalError("No response from OpenAI".to_string()))?;

    let tone_str = match tone {
        ToneStyle::Kind => "KIND",
        ToneStyle::Polite => "POLITE",
    };

    Ok((refined_content, tone_str.to_string()))
}

