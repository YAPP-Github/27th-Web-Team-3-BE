use crate::models::request::ToneStyle;

/// Generate a system prompt for refining retrospective content
pub fn get_refine_system_prompt(tone_style: &ToneStyle) -> String {
    let tone_description = match tone_style {
        ToneStyle::Kind => "친근하고 상냥한 말투로 작성해주세요. '~어요', '~네요' 등의 표현을 사용하세요.",
        ToneStyle::Polite => "정중하고 예의 바른 말투로 작성해주세요. 존댓말을 사용하고 격식을 갖춰주세요.",
    };

    format!(
        r#"당신은 회고 내용을 정제하는 AI 어시스턴트입니다.
사용자가 작성한 회고 내용을 다음 기준에 따라 정제해주세요:

1. 비속어나 은어를 적절한 표현으로 변환
2. 문법과 맞춤법 교정
3. {}
4. 원래 의미는 최대한 유지
5. 자연스럽고 읽기 쉬운 문장으로 개선

정제된 내용만 출력하고, 다른 설명은 추가하지 마세요."#,
        tone_description
    )
}

/// Generate a user prompt for refining retrospective content
pub fn get_refine_user_prompt(content: &str) -> String {
    format!("다음 회고 내용을 정제해주세요:\n\n{}", content)
}

/// Generate a system prompt for providing writing guide
pub fn get_guide_system_prompt() -> String {
    r#"당신은 회고 작성을 돕는 친절한 AI 어시스턴트입니다.
사용자가 현재 작성 중인 회고 내용을 보고, 다음과 같은 가이드를 제공해주세요:

1. 현재 작성된 내용에 대한 긍정적인 피드백
2. 더 구체적으로 작성할 수 있는 부분 제안
3. 회고에 포함하면 좋을 추가 내용 제안 (예: 배운 점, 개선할 점, 느낀 점 등)
4. 친근하고 격려하는 톤으로 작성

가이드 메시지는 2-3문장 정도로 간결하게 작성하고, 사용자가 부담스럽지 않게 제안해주세요."#
        .to_string()
}

/// Generate a user prompt for providing writing guide
pub fn get_guide_user_prompt(current_content: &str) -> String {
    if current_content.trim().is_empty() {
        "사용자가 회고 작성을 시작하려고 합니다. 어떻게 시작하면 좋을지 가이드를 제공해주세요.".to_string()
    } else {
        format!("사용자가 다음과 같이 회고를 작성 중입니다:\n\n{}\n\n이 내용을 바탕으로 다음 단계 작성을 위한 가이드를 제공해주세요.", current_content)
    }
}

