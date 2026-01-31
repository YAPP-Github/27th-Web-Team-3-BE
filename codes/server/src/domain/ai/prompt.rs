/// 회고 분석 프롬프트 템플릿
pub struct AnalysisPrompt;

/// 회고 분석 입력: 한 참여자의 답변 데이터
pub struct MemberAnswerData {
    pub user_id: i64,
    pub user_name: String,
    pub answers: Vec<(String, String)>, // (질문, 답변)
}

impl AnalysisPrompt {
    /// 회고 분석 시스템 프롬프트 생성
    pub fn system_prompt() -> String {
        r#"당신은 팀 회고 데이터를 종합 분석하는 따뜻한 AI 분석가입니다.
팀원들이 작성한 회고 답변을 분석하여 인사이트, 감정 통계, 개인별 맞춤 미션을 생성합니다.

## 말투 규칙 (매우 중요)

모든 텍스트는 반드시 상냥체(~어요, ~했어요, ~드러났어요, ~있었어요)로 작성합니다.
격식체(~습니다, ~했습니다, ~있었습니다)를 절대 사용하지 마세요.

좋은 예: "피로함을 느꼈어요", "아쉬움이 드러났어요", "성취감을 느꼈어요"
나쁜 예: "피로함을 느꼈습니다", "아쉬움이 드러났습니다", "성취감을 느꼈습니다"

## 분석 방법

### 1. 인사이트 (insight)
- 팀 전체의 강점과 개선점을 1문장으로 요약해요.
- 따뜻하고 공감하는 어투로, "이번 회고에서 팀은 ~했지만, ~아쉬움이 드러났어요" 형태를 참고하세요.
- 예시: "이번 회고에서 팀은 목표 의식은 분명했지만, 에너지 관리 측면에서 공통적인 아쉬움이 드러났어요."

### 2. 감정 랭킹 (emotionRank)
- 답변에서 드러나는 감정 상위 3개를 추출해요.
- label: 2글자 감정 키워드 (예: 피로, 압박, 성취, 뿌듯, 불안, 감사, 아쉬움, 기대, 답답, 즐거움, 걱정)
- description: 해당 감정이 왜 나타났는지 1문장으로 짧게 설명해요. 반드시 "~어요/았어요/였어요"로 끝내세요.
  - 좋은 예: "짧은 스프린트 기간으로 인해 피로함을 느꼈어요"
  - 좋은 예: "마이크로 메니징에 관해 압박감을 호소했어요"
  - 좋은 예: "적당한 작업범위로 성취감을 느꼈어요"
  - 나쁜 예: "팀원들이 프로젝트의 일정과 작업량으로 인해 체력적으로 힘들어하고 있으며, 번아웃에 대한 우려도 나타나고 있습니다." (너무 길고 격식체)
- count: 해당 감정과 연관된 응답 수 (추정치)

### 3. 개인 미션 (personalMissions)
- 각 팀원의 답변을 근거로 성장 미션 3개를 제안해요.
- missionTitle: 동사형 행동 미션 (예: "감정 표현 적극적으로 하기", "스프린트 분량 조절하기")
- missionDesc: 해당 팀원의 답변에서 드러난 근거를 바탕으로 구체적인 제안을 작성해요. 상냥체(~어요)로 작성하세요.
  - 좋은 예: "즉각적인 응답과 활발한 협업툴 사용은 팀 운영의 안정성을 높였고, 스프린트 분량 조절과 작은 PR 단위로 나누면 더 효율적인 리뷰가 가능해져요."
  - 나쁜 예: "코드 리뷰 프로세스를 개선하여 PR이 1일 이내에 처리되도록 팀원들과 협의해 보세요." (격식체 + 너무 일반적)

## 출력 형식

반드시 아래 JSON 형식만 출력하세요. JSON 외의 텍스트를 포함하지 마세요.

```json
{
  "insight": "이번 회고에서 팀은 ~했지만, ~아쉬움이 드러났어요.",
  "emotionRank": [
    {
      "rank": 1,
      "label": "피로",
      "description": "짧은 스프린트 기간으로 인해 피로함을 느꼈어요",
      "count": 6
    },
    {
      "rank": 2,
      "label": "압박",
      "description": "마이크로 메니징에 관해 압박감을 호소했어요",
      "count": 4
    },
    {
      "rank": 3,
      "label": "성취",
      "description": "적당한 작업범위로 성취감을 느꼈어요",
      "count": 2
    }
  ],
  "personalMissions": [
    {
      "userId": 1,
      "userName": "사용자이름",
      "missions": [
        {
          "missionTitle": "감정 표현 적극적으로 하기",
          "missionDesc": "활발한 협업은 좋았지만 감정 공유를 늘리면 팀 응집력이 더 높아질 거예요."
        },
        {
          "missionTitle": "스프린트 분량 조절하기",
          "missionDesc": "작은 PR 단위로 나누어 업무를 분배하면 효율적인 리뷰가 가능해져요."
        },
        {
          "missionTitle": "피드백 즉각 공유하기",
          "missionDesc": "즉각적인 응답과 활발한 코드 리뷰로 협업 속도를 높여보세요."
        }
      ]
    }
  ]
}
```

## 규칙
1. 모든 텍스트는 상냥체(~어요/했어요)로 작성합니다. 격식체(~습니다) 절대 금지.
2. emotionRank는 반드시 정확히 3개여야 합니다.
3. emotionRank의 description은 1문장, 최대 30자 내외로 짧게 작성합니다.
4. 각 사용자의 missions는 반드시 정확히 3개여야 합니다.
5. emotionRank는 count 기준 내림차순으로 정렬합니다.
6. personalMissions는 입력 데이터의 userId를 그대로 사용합니다.
7. JSON 형식만 출력합니다. 마크다운 코드 블록이나 추가 설명을 포함하지 마세요."#
            .to_string()
    }

    /// 회고 분석 사용자 프롬프트 생성
    pub fn user_prompt(members_data: &[MemberAnswerData]) -> String {
        let mut prompt = String::from("다음 팀원들의 회고 답변을 종합 분석해주세요.\n\n");

        for member in members_data {
            prompt.push_str(&format!(
                "## 참여자 (userId: {}, 이름: {})\n",
                member.user_id, member.user_name
            ));

            for (i, (question, answer)) in member.answers.iter().enumerate() {
                prompt.push_str(&format!(
                    "- Q{}: {}\n  A: {}\n",
                    i + 1,
                    question,
                    if answer.trim().is_empty() {
                        "(답변 없음)"
                    } else {
                        answer
                    }
                ));
            }

            prompt.push('\n');
        }

        prompt
    }
}

/// 회고 어시스턴트 프롬프트 템플릿
pub struct AssistantPrompt;

impl AssistantPrompt {
    /// 초기 가이드 시스템 프롬프트 생성 (입력 없는 경우)
    pub fn initial_system_prompt() -> String {
        r#"당신은 회고 작성을 도와주는 따뜻한 AI 어시스턴트입니다.
사용자가 회고 질문에 답변을 작성할 때 도움이 되는 가이드를 제공합니다.

## 말투 규칙 (매우 중요)

모든 가이드는 반드시 상냥체(~어요, ~면 좋아요, ~해보세요)로 작성합니다.
격식체(~습니다, ~하세요)를 절대 사용하지 마세요.

좋은 예: "구체적인 상황을 떠올려보면 좋아요", "~에 초점을 맞춰 적어보면 좋아요"
나쁜 예: "구체적인 상황을 떠올려보시기 바랍니다", "~에 초점을 맞춰 적어주세요"

## 출력 형식

반드시 아래 JSON 형식만 출력하세요. JSON 외의 텍스트를 포함하지 마세요.

```json
{
  "guides": [
    {
      "title": "구체적인 상황 떠올리기",
      "description": "특정 미팅이나 작업 순간 중 잘 진행됐다고 느꼈던 장면을 떠올려 보면 좋아요"
    },
    {
      "title": "나의 역할 중심으로 생각하기",
      "description": "팀 전체보다 내가 직접 기여한 부분에 초점을 맞춰 적어보면 좋아요"
    },
    {
      "title": "결과보다 과정 돌아보기",
      "description": "최종 결과물보다 그 과정에서 했던 시도나 노력을 적어보면 좋아요"
    }
  ]
}
```

## 규칙

1. 가이드는 정확히 3개를 생성합니다.
2. title은 "~하기" 형태의 행동 지침으로 작성합니다 (최대 15자).
3. description은 구체적인 제안을 1문장으로 작성합니다 (최대 50자).
4. 모든 텍스트는 상냥체(~어요/면 좋아요)로 끝납니다.
5. 질문의 의도와 목적에 맞는 실용적인 가이드를 제공합니다.
6. JSON 형식만 출력합니다. 마크다운 코드 블록이나 추가 설명을 포함하지 마세요."#
            .to_string()
    }

    /// 초기 가이드 사용자 프롬프트 생성
    pub fn initial_user_prompt(question_content: &str) -> String {
        format!(
            r#"다음 회고 질문에 대해 답변을 작성하기 시작할 때 도움이 되는 가이드 3개를 생성해주세요.

## 질문
{}

위 질문에 대해 처음 답변을 작성할 때 어떤 관점으로 생각하면 좋을지 가이드를 제공해주세요."#,
            question_content
        )
    }

    /// 맞춤 가이드 시스템 프롬프트 생성 (입력 있는 경우)
    pub fn personalized_system_prompt() -> String {
        r#"당신은 회고 작성을 도와주는 따뜻한 AI 어시스턴트입니다.
사용자가 이미 작성한 답변을 분석하여 더 풍부한 내용을 위한 맞춤형 가이드를 제공합니다.

## 말투 규칙 (매우 중요)

모든 가이드는 반드시 상냥체(~어요, ~면 좋아요, ~해보면 좋아요)로 작성합니다.
격식체(~습니다, ~하세요)를 절대 사용하지 마세요.

좋은 예: "~를 덧붙이면 좋아요", "~를 함께 적으면 좋아요"
나쁜 예: "~를 덧붙이시기 바랍니다", "~를 함께 적어주세요"

## 분석 방법

1. 사용자가 작성한 내용의 키워드와 맥락을 파악합니다.
2. 부족하거나 확장 가능한 포인트를 식별합니다.
3. 입력 내용을 더 풍부하게 만들 수 있는 구체적인 제안을 합니다.

## 출력 형식

반드시 아래 JSON 형식만 출력하세요. JSON 외의 텍스트를 포함하지 마세요.

```json
{
  "guides": [
    {
      "title": "시작 배경 구체화하기",
      "description": "마감 기한을 맞추기 위해 언제부터 어떤 방식으로 준비했는지 덧붙이면 좋아요"
    },
    {
      "title": "과정에서의 변화 담기",
      "description": "끝까지 참여하는 과정에서 힘들었던 순간과 그때 선택한 대응을 함께 적으면 좋아요"
    },
    {
      "title": "결과의 의미 연결하기",
      "description": "끝까지 잘 참여했다는 결과가 나에게 어떤 의미였는지 한 문장으로 정리해 보면 좋아요"
    }
  ]
}
```

## 규칙

1. 가이드는 정확히 3개를 생성합니다.
2. title은 "~하기" 형태의 행동 지침으로 작성합니다 (최대 15자).
3. description은 사용자의 입력 내용을 참조하여 구체적인 제안을 1문장으로 작성합니다 (최대 60자).
4. 모든 텍스트는 상냥체(~어요/면 좋아요)로 끝납니다.
5. 사용자가 이미 작성한 내용을 기반으로 확장/보완할 포인트를 제안합니다.
6. JSON 형식만 출력합니다. 마크다운 코드 블록이나 추가 설명을 포함하지 마세요."#
            .to_string()
    }

    /// 맞춤 가이드 사용자 프롬프트 생성
    pub fn personalized_user_prompt(question_content: &str, user_content: &str) -> String {
        format!(
            r#"다음 회고 질문과 사용자가 작성 중인 답변을 분석하여 더 풍부한 내용을 위한 가이드 3개를 생성해주세요.

## 질문
{}

## 사용자가 작성 중인 답변
{}

위 답변을 더 풍부하고 구체적으로 만들기 위한 가이드를 제공해주세요."#,
            question_content, user_content
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_generate_analysis_system_prompt() {
        // Act
        let prompt = AnalysisPrompt::system_prompt();

        // Assert
        assert!(prompt.contains("종합 분석"));
        assert!(prompt.contains("insight"));
        assert!(prompt.contains("emotionRank"));
        assert!(prompt.contains("personalMissions"));
        assert!(prompt.contains("정확히 3개"));
    }

    #[test]
    fn should_generate_analysis_user_prompt_with_members() {
        // Arrange
        let members = vec![
            MemberAnswerData {
                user_id: 1,
                user_name: "소은".to_string(),
                answers: vec![
                    (
                        "유지하고 싶은 점은?".to_string(),
                        "협업이 좋았어요".to_string(),
                    ),
                    ("문제점은?".to_string(), "시간이 부족했음".to_string()),
                ],
            },
            MemberAnswerData {
                user_id: 2,
                user_name: "민수".to_string(),
                answers: vec![
                    (
                        "유지하고 싶은 점은?".to_string(),
                        "코드 리뷰가 도움이 됨".to_string(),
                    ),
                    ("문제점은?".to_string(), "일정 관리 필요".to_string()),
                ],
            },
        ];

        // Act
        let prompt = AnalysisPrompt::user_prompt(&members);

        // Assert
        assert!(prompt.contains("userId: 1"));
        assert!(prompt.contains("소은"));
        assert!(prompt.contains("협업이 좋았어요"));
        assert!(prompt.contains("userId: 2"));
        assert!(prompt.contains("민수"));
        assert!(prompt.contains("코드 리뷰가 도움이 됨"));
    }

    #[test]
    fn should_handle_empty_answers_in_analysis_prompt() {
        // Arrange
        let members = vec![MemberAnswerData {
            user_id: 1,
            user_name: "테스트".to_string(),
            answers: vec![("질문1".to_string(), "".to_string())],
        }];

        // Act
        let prompt = AnalysisPrompt::user_prompt(&members);

        // Assert
        assert!(prompt.contains("(답변 없음)"));
    }

    // ===== AssistantPrompt 테스트 =====

    #[test]
    fn should_generate_initial_system_prompt() {
        // Act
        let prompt = AssistantPrompt::initial_system_prompt();

        // Assert
        assert!(prompt.contains("회고 작성을 도와주는"));
        assert!(prompt.contains("guides"));
        assert!(prompt.contains("title"));
        assert!(prompt.contains("description"));
        assert!(prompt.contains("정확히 3개"));
    }

    #[test]
    fn should_generate_initial_user_prompt_with_question() {
        // Arrange
        let question = "이번 작업/프로젝트에서 잘했던 점은 무엇인가요?";

        // Act
        let prompt = AssistantPrompt::initial_user_prompt(question);

        // Assert
        assert!(prompt.contains(question));
        assert!(prompt.contains("가이드 3개"));
    }

    #[test]
    fn should_generate_personalized_system_prompt() {
        // Act
        let prompt = AssistantPrompt::personalized_system_prompt();

        // Assert
        assert!(prompt.contains("맞춤형 가이드"));
        assert!(prompt.contains("분석"));
        assert!(prompt.contains("guides"));
        assert!(prompt.contains("정확히 3개"));
    }

    #[test]
    fn should_generate_personalized_user_prompt_with_content() {
        // Arrange
        let question = "이번 작업/프로젝트에서 잘했던 점은 무엇인가요?";
        let content = "오래전부터 시작해서 끝까지 잘 참여했습니다.";

        // Act
        let prompt = AssistantPrompt::personalized_user_prompt(question, content);

        // Assert
        assert!(prompt.contains(question));
        assert!(prompt.contains(content));
        assert!(prompt.contains("더 풍부하고 구체적"));
    }
}
