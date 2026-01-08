use anyhow::{Context, Result};
use std::env;

use crate::error::AppError;
use crate::models::request::ToneStyle;

/// AI 서비스
pub struct AiService {
    api_url: String,
    api_key: String,
    secret_key: String,
}

impl AiService {
    /// AI 서비스 인스턴스를 생성합니다.
    pub fn new() -> Result<Self> {
        let api_url = env::var("AI_API_URL")
            .context("AI_API_URL 환경 변수가 설정되지 않았습니다.")?;
        let api_key = env::var("AI_API_KEY")
            .context("AI_API_KEY 환경 변수가 설정되지 않았습니다.")?;
        let secret_key = env::var("SECRET_KEY")
            .context("SECRET_KEY 환경 변수가 설정되지 않았습니다.")?;

        Ok(Self {
            api_url,
            api_key,
            secret_key,
        })
    }

    /// secretKey 검증
    fn validate_secret_key(&self, secret_key: &str) -> Result<(), AppError> {
        if secret_key != self.secret_key {
            return Err(AppError::InvalidSecretKey);
        }
        Ok(())
    }

    /// 회고 가이드 메시지를 생성합니다.
    pub async fn generate_retrospective_guide(
        &self,
        content: &str,
        secret_key: &str,
    ) -> Result<String, AppError> {
        // secretKey 검증
        self.validate_secret_key(secret_key)?;

        // TODO: 실제 AI API 호출 로직 구현
        // 현재는 Mock 응답 반환
        let guide_message = self.mock_ai_guide_response(content);

        Ok(guide_message)
    }

    /// 회고 내용을 다듬습니다.
    pub async fn refine_retrospective(
        &self,
        content: &str,
        tone_style: &ToneStyle,
        secret_key: &str,
    ) -> Result<String, AppError> {
        // secretKey 검증
        self.validate_secret_key(secret_key)?;

        // TODO: 실제 AI API 호출 로직 구현
        // 현재는 Mock 응답 반환
        let refined_content = self.mock_ai_refine_response(content, tone_style);

        Ok(refined_content)
    }

    /// Mock AI 가이드 응답 (실제 구현 시 제거)
    fn mock_ai_guide_response(&self, content: &str) -> String {
        if content.len() < 20 {
            "좋은 시작이에요! 구체적으로 어떤 점이 어려웠는지 작성해보면 어떨까요?".to_string()
        } else if content.contains("어려움") || content.contains("힘들") {
            "어려움을 겪은 상황을 잘 설명하셨네요. 그 상황에서 어떤 노력을 했는지 추가로 작성해보시는 건 어떨까요?".to_string()
        } else if content.contains("배움") || content.contains("학습") {
            "배운 내용을 잘 정리하셨어요! 구체적인 예시나 코드를 추가하면 더 좋은 회고가 될 것 같아요.".to_string()
        } else {
            "회고 내용이 좋네요. 느낀 점이나 앞으로의 개선 방향도 함께 작성해보시면 어떨까요?".to_string()
        }
    }

    /// Mock AI 다듬기 응답 (실제 구현 시 제거)
    fn mock_ai_refine_response(&self, content: &str, tone_style: &ToneStyle) -> String {
        match tone_style {
            ToneStyle::Kind => {
                // 상냥체로 변환
                content
                    .replace("존나", "정말")
                    .replace("ㅋㅋ", "")
                    .replace(".", "~")
                    .trim()
                    .to_string() + "요."
            }
            ToneStyle::Polite => {
                // 정중체로 변환
                content
                    .replace("존나", "매우")
                    .replace("ㅋㅋ", "")
                    .trim()
                    .to_string() + "습니다."
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_secret_key_success() {
        env::set_var("SECRET_KEY", "test_key_123");
        env::set_var("AI_API_URL", "http://test.com");
        env::set_var("AI_API_KEY", "test_api_key");

        let service = AiService::new().unwrap();
        let result = service.validate_secret_key("test_key_123");

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_secret_key_failure() {
        env::set_var("SECRET_KEY", "correct_key");
        env::set_var("AI_API_URL", "http://test.com");
        env::set_var("AI_API_KEY", "test_api_key");

        let service = AiService::new().unwrap();
        let result = service.validate_secret_key("wrong_key");

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_generate_retrospective_guide() {
        env::set_var("SECRET_KEY", "test_key");
        env::set_var("AI_API_URL", "http://test.com");
        env::set_var("AI_API_KEY", "test_api_key");

        let service = AiService::new().unwrap();
        let result = service
            .generate_retrospective_guide("짧은 내용", "test_key")
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap().contains("좋은 시작"));
    }

    #[tokio::test]
    async fn test_refine_retrospective_polite() {
        env::set_var("SECRET_KEY", "test_key");
        env::set_var("AI_API_URL", "http://test.com");
        env::set_var("AI_API_KEY", "test_api_key");

        let service = AiService::new().unwrap();
        let result = service
            .refine_retrospective("오늘 일 존나 힘들었음", &ToneStyle::Polite, "test_key")
            .await;

        assert!(result.is_ok());
        let refined = result.unwrap();
        assert!(refined.contains("매우"));
        assert!(refined.ends_with("습니다."));
    }

    #[tokio::test]
    async fn test_refine_retrospective_kind() {
        env::set_var("SECRET_KEY", "test_key");
        env::set_var("AI_API_URL", "http://test.com");
        env::set_var("AI_API_KEY", "test_api_key");

        let service = AiService::new().unwrap();
        let result = service
            .refine_retrospective("오늘 일 존나 힘들었음", &ToneStyle::Kind, "test_key")
            .await;

        assert!(result.is_ok());
        let refined = result.unwrap();
        assert!(refined.contains("정말"));
        assert!(refined.ends_with("요."));
    }
}


