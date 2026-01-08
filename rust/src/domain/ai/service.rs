use crate::error::AppError;
use crate::domain::ai::prompt;

/// 회고 작성 가이드 메시지 생성
pub async fn generate_retrospective_guide(content: &str) -> Result<String, AppError> {
    // AI 프롬프트 생성
    let prompt = prompt::create_retrospective_guide_prompt(content);

    // TODO: 실제 AI API 호출 (현재는 Mock 응답)
    // 실제 구현 시 OpenAI API 등을 호출
    let guide_message = mock_ai_response(&prompt).await?;

    Ok(guide_message)
}

/// Mock AI 응답 (개발/테스트용)
async fn mock_ai_response(_prompt: &str) -> Result<String, AppError> {
    // 실제 환경에서는 여기서 AI API를 호출
    Ok("좋은 시작이에요! 구체적으로 어떤 어려움이 있었는지, 그리고 무엇을 배웠는지 더 자세히 작성해보면 좋을 것 같아요. 또한 다음에 비슷한 상황이 왔을 때 어떻게 대처할지 계획을 추가하면 더 완성도 높은 회고가 될 거예요.".to_string())
}

/// 회고 다듬기
pub async fn refine_retrospective(content: &str, tone_style: &str) -> Result<String, AppError> {
    // AI 프롬프트 생성
    let prompt = prompt::create_refine_prompt(content, tone_style);

    // TODO: 실제 AI API 호출
    let refined = mock_refine_response(content, tone_style).await?;

    Ok(refined)
}

/// Mock 다듬기 응답 (개발/테스트용)
async fn mock_refine_response(content: &str, tone_style: &str) -> Result<String, AppError> {
    // 실제 환경에서는 여기서 AI API를 호출
    let refined = if tone_style == "상냥체" {
        content.replace("존나", "정말").replace("ㅋㅋ", "")
    } else {
        content.replace("존나", "매우").replace("ㅋㅋ", "")
    };

    Ok(refined.trim().to_string())
}

