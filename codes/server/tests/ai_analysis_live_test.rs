/// 실제 OpenAI API를 호출하여 회고 분석 input/output을 확인하는 통합 테스트
///
/// 실행 방법:
/// ```
/// cargo test --test ai_analysis_live_test -- --nocapture --ignored
/// ```
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
    Client,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

// ===== 분석 응답 DTO (파싱용) =====

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AnalysisResponse {
    insight: String,
    emotion_rank: Vec<EmotionRankItem>,
    personal_missions: Vec<PersonalMissionItem>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EmotionRankItem {
    rank: i32,
    label: String,
    description: String,
    count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersonalMissionItem {
    user_id: i64,
    user_name: String,
    missions: Vec<MissionItem>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MissionItem {
    mission_title: String,
    mission_desc: String,
}

// ===== 프롬프트 (prompt.rs와 동일) =====

fn system_prompt() -> String {
    r#"당신은 팀 회고 데이터를 종합 분석하는 따뜻한 AI 분석가입니다.
팀원들이 작성한 회고 답변을 분석하여 팀 인사이트, 감정 통계, 개인별 맞춤 미션을 생성합니다.

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

struct MemberAnswerData {
    user_id: i64,
    user_name: String,
    answers: Vec<(String, String)>,
}

fn user_prompt(members_data: &[MemberAnswerData]) -> String {
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

/// KPT 회고 방식의 샘플 데이터 생성
fn create_sample_members() -> Vec<MemberAnswerData> {
    vec![
        MemberAnswerData {
            user_id: 1,
            user_name: "소은".to_string(),
            answers: vec![
                (
                    "계속 유지하고 싶은 좋은 점은 무엇인가요?".to_string(),
                    "매일 아침 스탠드업 미팅이 팀 전체의 진행 상황을 파악하는 데 큰 도움이 됐어요. 서로 뭘 하고 있는지 알 수 있어서 중복 작업이 줄었습니다.".to_string(),
                ),
                (
                    "개선이 필요한 문제점은 무엇인가요?".to_string(),
                    "코드 리뷰가 너무 늦게 이루어져서 PR이 3일 이상 방치되는 경우가 많았습니다. 빠른 피드백이 필요해요.".to_string(),
                ),
                (
                    "다음에 시도해보고 싶은 것은 무엇인가요?".to_string(),
                    "페어 프로그래밍을 주 1회 시도해보고 싶어요. 복잡한 기능 개발 시 함께 하면 버그도 줄고 지식 공유도 될 것 같습니다.".to_string(),
                ),
                (
                    "전체 프로젝트를 돌아보며 느낀 점을 자유롭게 작성해주세요.".to_string(),
                    "첫 스프린트라 다소 혼란스러웠지만, 팀원들이 서로 도와주려는 분위기가 좋았어요. 다만 일정이 빡빡해서 체력적으로 힘들었습니다.".to_string(),
                ),
                (
                    "추가로 공유하고 싶은 의견이 있나요?".to_string(),
                    "회고를 더 자주 했으면 좋겠어요. 2주마다 한 번씩 하면 개선 사항을 빨리 반영할 수 있을 것 같습니다.".to_string(),
                ),
            ],
        },
        MemberAnswerData {
            user_id: 2,
            user_name: "민수".to_string(),
            answers: vec![
                (
                    "계속 유지하고 싶은 좋은 점은 무엇인가요?".to_string(),
                    "GitHub Projects를 활용한 태스크 관리가 효과적이었습니다. 누가 어떤 작업을 하고 있는지 한눈에 볼 수 있었어요.".to_string(),
                ),
                (
                    "개선이 필요한 문제점은 무엇인가요?".to_string(),
                    "API 스펙이 자주 바뀌어서 프론트엔드와의 협업이 어려웠습니다. 스펙 확정 후 작업 시작하는 프로세스가 필요합니다.".to_string(),
                ),
                (
                    "다음에 시도해보고 싶은 것은 무엇인가요?".to_string(),
                    "CI/CD 파이프라인을 좀 더 강화해서 자동 배포까지 연결하고 싶습니다. 수동 배포 과정에서 실수가 몇 번 있었거든요.".to_string(),
                ),
                (
                    "전체 프로젝트를 돌아보며 느낀 점을 자유롭게 작성해주세요.".to_string(),
                    "기술적으로 많이 성장한 느낌이에요. 특히 Rust로 백엔드를 처음 만들어보면서 소유권 시스템에 대한 이해가 깊어졌습니다. 하지만 야근이 잦아서 번아웃 직전이에요.".to_string(),
                ),
                (
                    "추가로 공유하고 싶은 의견이 있나요?".to_string(),
                    "팀 빌딩 활동이 더 있었으면 좋겠어요. 온라인으로만 소통하다 보니 팀원 간의 친밀감이 부족한 것 같습니다.".to_string(),
                ),
            ],
        },
        MemberAnswerData {
            user_id: 3,
            user_name: "지원".to_string(),
            answers: vec![
                (
                    "계속 유지하고 싶은 좋은 점은 무엇인가요?".to_string(),
                    "디자인 시스템을 초기에 잘 잡아놔서 UI 구현 속도가 빨랐어요. 컴포넌트 재사용성이 높아서 좋았습니다.".to_string(),
                ),
                (
                    "개선이 필요한 문제점은 무엇인가요?".to_string(),
                    "테스트 코드 작성에 시간을 충분히 할애하지 못했어요. 급하게 기능만 만들다 보니 나중에 버그가 많이 발견됐습니다.".to_string(),
                ),
                (
                    "다음에 시도해보고 싶은 것은 무엇인가요?".to_string(),
                    "TDD를 본격적으로 도입해보고 싶어요. 테스트를 먼저 작성하면 설계도 더 좋아질 것 같고, 리팩토링 시에도 안전할 것 같습니다.".to_string(),
                ),
                (
                    "전체 프로젝트를 돌아보며 느낀 점을 자유롭게 작성해주세요.".to_string(),
                    "처음에는 막막했는데 점점 속도가 붙으면서 뿌듯함을 느꼈어요. 다만 마감 직전에 몰아서 작업하는 습관을 고쳐야 할 것 같아요.".to_string(),
                ),
                (
                    "추가로 공유하고 싶은 의견이 있나요?".to_string(),
                    "문서화를 좀 더 체계적으로 했으면 좋겠어요. 나중에 합류하는 팀원이나 유지보수할 때 큰 도움이 될 거예요.".to_string(),
                ),
            ],
        },
    ]
}

fn extract_json(response: &str) -> &str {
    let trimmed = response.trim();
    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            return &trimmed[start..=end];
        }
    }
    trimmed
}

// ===== 프롬프트 Input만 보는 테스트 (API 호출 없음) =====
#[test]
fn show_analysis_prompt_input() {
    let members = create_sample_members();

    println!("\n{}", "=".repeat(80));
    println!("===== [INPUT 1] System Prompt =====");
    println!("{}", "=".repeat(80));
    println!("{}", system_prompt());

    println!("\n{}", "=".repeat(80));
    println!("===== [INPUT 2] User Prompt =====");
    println!("{}", "=".repeat(80));
    println!("{}", user_prompt(&members));
}

// ===== 실제 OpenAI API 호출 테스트 =====
#[tokio::test]
#[ignore] // cargo test --test ai_analysis_live_test -- --nocapture --ignored
async fn live_analysis_api_call() {
    dotenvy::dotenv().ok();

    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "test-key".to_string());

    if api_key == "test-key" {
        println!("\n⚠️  OPENAI_API_KEY가 설정되지 않았습니다. .env 파일에 실제 키를 설정해주세요.");
        return;
    }

    let openai_config = OpenAIConfig::new().with_api_key(&api_key);
    let client = Client::with_config(openai_config);
    let members = create_sample_members();

    // ===== INPUT 출력 =====
    let sys_prompt = system_prompt();
    let usr_prompt = user_prompt(&members);

    println!("\n{}", "=".repeat(80));
    println!("===== [INPUT 1] System Prompt =====");
    println!("{}", "=".repeat(80));
    println!("{}", sys_prompt);

    println!("\n{}", "=".repeat(80));
    println!("===== [INPUT 2] User Prompt =====");
    println!("{}", "=".repeat(80));
    println!("{}", usr_prompt);

    // ===== API 호출 =====
    println!("\n{}", "=".repeat(80));
    println!("===== OpenAI API 호출 중... =====");
    println!("{}", "=".repeat(80));

    let messages: Vec<ChatCompletionRequestMessage> = vec![
        ChatCompletionRequestSystemMessageArgs::default()
            .content(sys_prompt.as_str())
            .build()
            .unwrap()
            .into(),
        ChatCompletionRequestUserMessageArgs::default()
            .content(usr_prompt.as_str())
            .build()
            .unwrap()
            .into(),
    ];

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4o-mini")
        .messages(messages)
        .temperature(0.7)
        .max_tokens(4000u32)
        .build()
        .unwrap();

    let chat = client.chat();
    let result = tokio::time::timeout(Duration::from_secs(30), chat.create(request)).await;

    match result {
        Err(_) => {
            println!("❌ 타임아웃: 30초 초과");
            return;
        }
        Ok(Err(e)) => {
            println!("❌ API 호출 실패: {}", e);
            return;
        }
        Ok(Ok(response)) => {
            let raw_content = response
                .choices
                .first()
                .and_then(|c| c.message.content.clone())
                .unwrap_or_default();

            // ===== RAW OUTPUT =====
            println!("\n{}", "=".repeat(80));
            println!("===== [OUTPUT - RAW] AI 원본 응답 =====");
            println!("{}", "=".repeat(80));
            println!("{}", raw_content);

            // ===== PARSED OUTPUT =====
            let json_str = extract_json(&raw_content);
            match serde_json::from_str::<AnalysisResponse>(json_str) {
                Ok(analysis) => {
                    println!("\n{}", "=".repeat(80));
                    println!("===== [OUTPUT - PARSED] 파싱된 분석 결과 =====");
                    println!("{}", "=".repeat(80));
                    println!("{}", serde_json::to_string_pretty(&analysis).unwrap());

                    // ===== VALIDATION =====
                    println!("\n{}", "=".repeat(80));
                    println!("===== [VALIDATION] 응답 검증 =====");
                    println!("{}", "=".repeat(80));
                    println!("✅ insight: \"{}\"", analysis.insight);
                    println!(
                        "✅ emotionRank 개수: {} (기대: 3)",
                        analysis.emotion_rank.len()
                    );
                    for e in &analysis.emotion_rank {
                        println!(
                            "   rank={}, label=\"{}\", count={}, desc=\"{}\"",
                            e.rank, e.label, e.count, e.description
                        );
                    }
                    println!(
                        "✅ personalMissions 개수: {} (기대: 3)",
                        analysis.personal_missions.len()
                    );
                    for pm in &analysis.personal_missions {
                        println!(
                            "   userId={}, userName=\"{}\", missions={}개",
                            pm.user_id,
                            pm.user_name,
                            pm.missions.len()
                        );
                        for m in &pm.missions {
                            println!("     • {}: {}", m.mission_title, m.mission_desc);
                        }
                    }

                    // 구조 검증
                    assert_eq!(
                        analysis.emotion_rank.len(),
                        3,
                        "emotionRank must be exactly 3"
                    );
                    assert_eq!(
                        analysis.personal_missions.len(),
                        3,
                        "personalMissions must match member count"
                    );
                    for pm in &analysis.personal_missions {
                        assert_eq!(
                            pm.missions.len(),
                            3,
                            "each user must have exactly 3 missions"
                        );
                    }
                    println!("\n✅ 모든 검증 통과!");
                }
                Err(e) => {
                    println!("\n❌ JSON 파싱 실패: {}", e);
                    println!("JSON 추출 결과: {}", json_str);
                }
            }
        }
    }
}
