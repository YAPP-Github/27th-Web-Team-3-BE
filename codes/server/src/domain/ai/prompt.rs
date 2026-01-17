use super::dto::ToneStyle;

/// AI 프롬프트 템플릿
pub struct RefinePrompt;

impl RefinePrompt {
    /// 회고 정제 시스템 프롬프트 생성
    pub fn system_prompt(tone_style: ToneStyle) -> String {
        let tone_description = match tone_style {
            ToneStyle::Kind => {
                r#"상냥체로 변환해주세요.
- 친근하고 부드러운 어투를 사용합니다
- "~해요", "~했어요", "~네요" 등의 표현을 사용합니다
- 이모티콘이나 느낌표를 적절히 사용할 수 있습니다
- 예시: "오늘 정말 힘들었어요", "많이 배웠네요!""#
            }
            ToneStyle::Polite => {
                r#"정중체로 변환해주세요.
- 공손하고 격식 있는 어투를 사용합니다
- "~습니다", "~했습니다", "~입니다" 등의 표현을 사용합니다
- 전문적이고 진중한 톤을 유지합니다
- 예시: "오늘 업무가 힘들었습니다", "많은 것을 배웠습니다""#
            }
        };

        format!(
            r#"당신은 회고록 작성을 도와주는 AI 어시스턴트입니다.
사용자가 작성한 회고 내용을 주어진 말투 스타일로 정제해주세요.

## 말투 스타일
{}

## 규칙
1. 원본 내용의 의미와 핵심 메시지를 유지합니다
2. 문장을 자연스럽게 다듬습니다
3. 맞춤법과 띄어쓰기를 교정합니다
4. 내용을 추가하거나 삭제하지 않습니다
5. 정제된 내용만 출력합니다 (추가 설명 없이)"#,
            tone_description
        )
    }

    /// 사용자 프롬프트 생성
    pub fn user_prompt(content: &str) -> String {
        format!("다음 회고 내용을 정제해주세요:\n\n{}", content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_generate_kind_system_prompt() {
        // Arrange
        let tone = ToneStyle::Kind;

        // Act
        let prompt = RefinePrompt::system_prompt(tone);

        // Assert
        assert!(prompt.contains("상냥체"));
        assert!(prompt.contains("~해요"));
    }

    #[test]
    fn should_generate_polite_system_prompt() {
        // Arrange
        let tone = ToneStyle::Polite;

        // Act
        let prompt = RefinePrompt::system_prompt(tone);

        // Assert
        assert!(prompt.contains("정중체"));
        assert!(prompt.contains("~습니다"));
    }

    #[test]
    fn should_generate_user_prompt_with_content() {
        // Arrange
        let content = "오늘 힘들었음";

        // Act
        let prompt = RefinePrompt::user_prompt(content);

        // Assert
        assert!(prompt.contains(content));
        assert!(prompt.contains("정제해주세요"));
    }
}
