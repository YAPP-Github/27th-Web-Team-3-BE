use std::time::Duration;

use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
    Client,
};
use tracing::{info, instrument, warn};

use crate::config::AppConfig;
use crate::domain::retrospect::dto::{AnalysisResponse, GuideItem};
use crate::utils::AppError;

use super::prompt::{AnalysisPrompt, AssistantPrompt, MemberAnswerData};

/// 어시스턴트 가이드 응답 (내부용)
#[derive(Debug, serde::Deserialize)]
pub struct AssistantGuideRaw {
    pub guides: Vec<GuideItem>,
}

/// AI 서비스
#[derive(Clone)]
pub struct AiService {
    client: Client<OpenAIConfig>,
}

impl AiService {
    /// 새 AiService 인스턴스 생성
    pub fn new(config: &AppConfig) -> Self {
        let openai_config = OpenAIConfig::new().with_api_key(&config.openai_api_key);
        let client = Client::with_config(openai_config);

        Self { client }
    }

    /// 회고 종합 분석 (API-022)
    #[instrument(skip(self, members_data), fields(member_count = members_data.len()))]
    pub async fn analyze_retrospective(
        &self,
        members_data: &[MemberAnswerData],
    ) -> Result<AnalysisResponse, AppError> {
        info!("회고 종합 분석 시작 (참여자 {}명)", members_data.len());

        let system_prompt = AnalysisPrompt::system_prompt();
        let user_prompt = AnalysisPrompt::user_prompt(members_data);

        let raw_response = self.call_openai(&system_prompt, &user_prompt).await?;

        // JSON 파싱 (코드 블록 제거 후 파싱 시도)
        let json_str = Self::extract_json(&raw_response);
        let analysis: AnalysisResponse = serde_json::from_str(json_str).map_err(|e| {
            warn!("AI 응답 JSON 파싱 실패: {}", e);
            warn!("AI 원본 응답: {}", raw_response);
            AppError::AiAnalysisFailed(format!("AI 응답을 파싱할 수 없습니다: {}", e))
        })?;

        // 응답 검증: emotionRank 정확히 3개
        if analysis.emotion_rank.len() != 3 {
            return Err(AppError::AiAnalysisFailed(format!(
                "감정 랭킹이 3개여야 하지만 {}개입니다",
                analysis.emotion_rank.len()
            )));
        }

        // 응답 검증: 각 사용자의 missions 정확히 3개
        for pm in &analysis.personal_missions {
            if pm.missions.len() != 3 {
                return Err(AppError::AiAnalysisFailed(format!(
                    "사용자 {}의 미션이 3개여야 하지만 {}개입니다",
                    pm.user_id,
                    pm.missions.len()
                )));
            }
        }

        info!("회고 종합 분석 완료");
        Ok(analysis)
    }

    /// 회고 어시스턴트 가이드 생성 (API-029)
    #[instrument(skip(self))]
    pub async fn generate_assistant_guide(
        &self,
        question_content: &str,
        user_content: Option<&str>,
    ) -> Result<Vec<GuideItem>, AppError> {
        let (system_prompt, user_prompt) = match user_content {
            Some(content) if !content.trim().is_empty() => {
                info!("맞춤 가이드 생성 요청");
                (
                    AssistantPrompt::personalized_system_prompt(),
                    AssistantPrompt::personalized_user_prompt(question_content, content),
                )
            }
            _ => {
                info!("초기 가이드 생성 요청");
                (
                    AssistantPrompt::initial_system_prompt(),
                    AssistantPrompt::initial_user_prompt(question_content),
                )
            }
        };

        let raw_response = self.call_openai(&system_prompt, &user_prompt).await?;

        // JSON 파싱
        let json_str = Self::extract_json(&raw_response);
        let guide_response: AssistantGuideRaw = serde_json::from_str(json_str).map_err(|e| {
            warn!("AI 응답 JSON 파싱 실패: {}", e);
            warn!("AI 원본 응답: {}", raw_response);
            AppError::AiAnalysisFailed(format!("AI 응답을 파싱할 수 없습니다: {}", e))
        })?;

        // 응답 검증: guides 1~3개
        let guide_count = guide_response.guides.len();
        if guide_count == 0 || guide_count > 3 {
            return Err(AppError::AiAnalysisFailed(format!(
                "가이드는 1~3개여야 하지만 {}개입니다",
                guide_count
            )));
        }

        info!("어시스턴트 가이드 생성 완료");
        Ok(guide_response.guides)
    }

    /// AI 응답에서 JSON 부분 추출 (코드 블록 제거)
    fn extract_json(response: &str) -> &str {
        let trimmed = response.trim();

        // ```json ... ``` 패턴 처리
        if let Some(start) = trimmed.find('{') {
            if let Some(end) = trimmed.rfind('}') {
                return &trimmed[start..=end];
            }
        }

        trimmed
    }

    /// OpenAI API 호출 (타임아웃 포함)
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
            .max_tokens(4000u32)
            .build()
            .map_err(|e| AppError::AiGeneralError(e.to_string()))?;

        let chat = self.client.chat();
        let api_call = chat.create(request);
        let response = tokio::time::timeout(Duration::from_secs(30), api_call)
            .await
            .map_err(|_| {
                AppError::AiServiceUnavailable("AI 서비스 응답 시간이 초과되었습니다".to_string())
            })?
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::retrospect::dto::AnalysisResponse;

    // ===== extract_json 테스트 =====

    #[test]
    fn should_extract_json_from_plain_response() {
        // Arrange
        let response = r#"{"teamInsight": "test"}"#;

        // Act
        let result = AiService::extract_json(response);

        // Assert
        assert_eq!(result, r#"{"teamInsight": "test"}"#);
    }

    #[test]
    fn should_extract_json_from_code_block() {
        // Arrange
        let response = "```json\n{\"teamInsight\": \"test\"}\n```";

        // Act
        let result = AiService::extract_json(response);

        // Assert
        assert_eq!(result, "{\"teamInsight\": \"test\"}");
    }

    #[test]
    fn should_extract_json_with_surrounding_text() {
        // Arrange
        let response = "Here is the result:\n{\"teamInsight\": \"test\"}\nDone.";

        // Act
        let result = AiService::extract_json(response);

        // Assert
        assert_eq!(result, "{\"teamInsight\": \"test\"}");
    }

    #[test]
    fn should_parse_valid_analysis_response() {
        // Arrange
        let json = r#"{
            "teamInsight": "팀이 잘했습니다",
            "emotionRank": [
                {"rank": 1, "label": "뿌듯", "description": "성취감", "count": 5},
                {"rank": 2, "label": "피로", "description": "업무량", "count": 3},
                {"rank": 3, "label": "기대", "description": "미래", "count": 2}
            ],
            "personalMissions": [
                {
                    "userId": 1,
                    "userName": "소은",
                    "missions": [
                        {"missionTitle": "미션1", "missionDesc": "설명1"},
                        {"missionTitle": "미션2", "missionDesc": "설명2"},
                        {"missionTitle": "미션3", "missionDesc": "설명3"}
                    ]
                }
            ]
        }"#;

        // Act
        let result: Result<AnalysisResponse, _> = serde_json::from_str(json);

        // Assert
        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert_eq!(analysis.team_insight, "팀이 잘했습니다");
        assert_eq!(analysis.emotion_rank.len(), 3);
        assert_eq!(analysis.personal_missions.len(), 1);
        assert_eq!(analysis.personal_missions[0].missions.len(), 3);
    }
}
