use crate::config::AppConfig;
use crate::error::AppError;
use crate::models::request::ToneStyle;
use crate::domain::ai::prompt;
use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

/// Calls OpenAI API to refine retrospective content
pub async fn refine_content(
    content: &str,
    tone_style: &ToneStyle,
    config: &AppConfig,
) -> Result<String, AppError> {
    let client = reqwest::Client::new();

    let system_prompt = prompt::get_refine_system_prompt(tone_style);
    let user_prompt = prompt::get_refine_user_prompt(content);

    let request_body = OpenAIRequest {
        model: "gpt-4o-mini".to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: system_prompt,
            },
            Message {
                role: "user".to_string(),
                content: user_prompt,
            },
        ],
        temperature: 0.7,
    };

    let response = client
        .post(format!("{}/v1/chat/completions", config.openai_api_base))
        .header("Authorization", format!("Bearer {}", config.openai_api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| AppError::ExternalApiError(format!("OpenAI API 호출 실패: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(AppError::ExternalApiError(format!(
            "OpenAI API 에러 ({}): {}",
            status, error_text
        )));
    }

    let openai_response: OpenAIResponse = response
        .json()
        .await
        .map_err(|e| AppError::ExternalApiError(format!("응답 파싱 실패: {}", e)))?;

    openai_response
        .choices
        .first()
        .map(|choice| choice.message.content.clone())
        .ok_or_else(|| AppError::ExternalApiError("응답에 내용이 없습니다.".to_string()))
}

/// Calls OpenAI API to provide writing guide
pub async fn provide_writing_guide(
    current_content: &str,
    config: &AppConfig,
) -> Result<String, AppError> {
    let client = reqwest::Client::new();

    let system_prompt = prompt::get_guide_system_prompt();
    let user_prompt = prompt::get_guide_user_prompt(current_content);

    let request_body = OpenAIRequest {
        model: "gpt-4o-mini".to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: system_prompt,
            },
            Message {
                role: "user".to_string(),
                content: user_prompt,
            },
        ],
        temperature: 0.8,
    };

    let response = client
        .post(format!("{}/v1/chat/completions", config.openai_api_base))
        .header("Authorization", format!("Bearer {}", config.openai_api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| AppError::ExternalApiError(format!("OpenAI API 호출 실패: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(AppError::ExternalApiError(format!(
            "OpenAI API 에러 ({}): {}",
            status, error_text
        )));
    }

    let openai_response: OpenAIResponse = response
        .json()
        .await
        .map_err(|e| AppError::ExternalApiError(format!("응답 파싱 실패: {}", e)))?;

    openai_response
        .choices
        .first()
        .map(|choice| choice.message.content.clone())
        .ok_or_else(|| AppError::ExternalApiError("응답에 내용이 없습니다.".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ai::prompt;

    #[test]
    fn refine_prompt_includes_tone_style() {
        let prompt = prompt::get_refine_system_prompt(&ToneStyle::Kind);
        assert!(prompt.contains("상냥한"));
    }
}
