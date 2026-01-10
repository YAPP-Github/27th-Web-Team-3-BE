//! 회고 말투 정제 프롬프트
//!
//! 회고 내용을 선택한 말투(상냥체/정중체)로 정제할 때 사용하는 프롬프트입니다.

use crate::domain::ai::dto::ToneStyle;

use super::examples;

/// 정제 System Prompt 생성
pub fn system_prompt(tone_style: ToneStyle) -> String {
    let style_name = tone_style.style_name();
    let style_guide = style_guide(tone_style);

    format!(
        r#"당신은 전문적인 텍스트 편집자입니다.
사용자의 회고 내용을 {}(으)로 정제하는 역할을 수행합니다.

지침:
1. 원본의 의미와 내용을 절대 변경하지 마세요
2. 비속어, 은어, 과도한 축약어를 적절한 표현으로 대체하세요
3. 문장 구조를 자연스럽게 개선하세요
4. 맞춤법과 띄어쓰기를 정확하게 교정하세요
5. {} 스타일에 맞는 어미와 표현을 사용하세요
6. 영어식 문장 구조나 번역체 표현을 사용하지 말고, 한국어 원어민이 실제로 사용하는 자연스러운 문장 흐름으로 작성하세요
7. 문단 도입부에 불필요한 안부형 표현(예: ~어떻게 지내시는지, ~궁금해요)은 사용하지 마세요

{}

정제된 텍스트만 출력하고, 다른 설명은 포함하지 마세요."#,
        style_name, style_name, style_guide
    )
}

/// 정제 Few-Shot Examples 가져오기
pub fn few_shot_examples(tone_style: ToneStyle) -> &'static str {
    match tone_style {
        ToneStyle::Kind => examples::KIND,
        ToneStyle::Polite => examples::POLITE,
    }
}

/// 스타일별 세부 지침
fn style_guide(tone_style: ToneStyle) -> &'static str {
    match tone_style {
        ToneStyle::Kind => KIND_STYLE_GUIDE,
        ToneStyle::Polite => POLITE_STYLE_GUIDE,
    }
}

const KIND_STYLE_GUIDE: &str = r#"상냥체 스타일:
- 따뜻하고 친근한 어조를 유지하세요
- 어미: ~해요, ~이에요, ~네요, ~군요
- 부드럽고 긍정적인 표현 사용
- 공감을 표현하되 과도한 감정 표현이나 안부형 도입은 피하세요
- 영어식 문장 구조나 번역체 표현을 사용하지 마세요
- 예: "정말 좋았어요", "배울 수 있었네요", "노력했어요""#;

const POLITE_STYLE_GUIDE: &str = r#"정중체 스타일:
- 격식을 갖춘 정중한 어조를 유지하세요
- 어미: ~습니다, ~했습니다, ~입니다
- 존중하는 표현 사용
- 감정 표현은 절제하고 사실 중심으로 정리하세요
- 영어식 문장 구조나 번역체 표현을 사용하지 마세요
- 예: "진행했습니다", "배웠습니다", "느꼈습니다""#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_prompt_should_return_kind_prompt() {
        let prompt = system_prompt(ToneStyle::Kind);
        assert!(prompt.contains("상냥체"));
        assert!(prompt.contains("친근"));
    }

    #[test]
    fn system_prompt_should_return_polite_prompt() {
        let prompt = system_prompt(ToneStyle::Polite);
        assert!(prompt.contains("정중체"));
        assert!(prompt.contains("격식"));
    }

    #[test]
    fn system_prompt_should_contain_all_7_guidelines() {
        let prompt = system_prompt(ToneStyle::Kind);

        assert!(prompt.contains("원본의 의미와 내용을 절대 변경하지"));
        assert!(prompt.contains("비속어, 은어, 과도한 축약어"));
        assert!(prompt.contains("문장 구조를 자연스럽게"));
        assert!(prompt.contains("맞춤법과 띄어쓰기"));
        assert!(prompt.contains("스타일에 맞는 어미와 표현"));
        assert!(prompt.contains("영어식 문장 구조나 번역체"));
        assert!(prompt.contains("안부형 표현"));
    }

    #[test]
    fn system_prompt_should_contain_style_guide() {
        let kind_prompt = system_prompt(ToneStyle::Kind);
        let polite_prompt = system_prompt(ToneStyle::Polite);

        assert!(kind_prompt.contains("따뜻하고 친근한 어조"));
        assert!(kind_prompt.contains("~해요, ~이에요, ~네요"));

        assert!(polite_prompt.contains("격식을 갖춘 정중한 어조"));
        assert!(polite_prompt.contains("~습니다, ~했습니다"));
    }

    #[test]
    fn few_shot_examples_should_return_kind_examples() {
        let examples = few_shot_examples(ToneStyle::Kind);
        assert!(examples.contains("었어요"));
    }

    #[test]
    fn few_shot_examples_should_return_polite_examples() {
        let examples = few_shot_examples(ToneStyle::Polite);
        assert!(examples.contains("습니다"));
    }
}
