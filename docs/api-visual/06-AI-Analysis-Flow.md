# 🤖 AI Analysis Flow

> OpenAI를 활용한 회고 분석 상세 플로우

---

## 📍 Overview

```mermaid
flowchart TB
    subgraph input["📥 Input"]
        RESPONSES["팀원 답변들"]
        METHOD["회고 방식"]
    end

    subgraph process["⚙️ Process"]
        VALIDATE["검증"]
        PROMPT["프롬프트 생성"]
        API["OpenAI API 호출"]
        PARSE["결과 파싱"]
    end

    subgraph output["📤 Output"]
        TEAM["팀 인사이트"]
        EMOTION["감정 분석"]
        T_MISSION["팀 미션"]
        P_MISSION["개인 미션"]
    end

    input --> VALIDATE --> PROMPT --> API --> PARSE --> output
```

---

## 1️⃣ 분석 요청 조건

```mermaid
flowchart TB
    START["분석 요청"]

    CHECK1{"회고방 Owner?"}
    CHECK2{"제출된 답변 있음?"}
    CHECK3{"월간 한도 내?"}
    CHECK4{"이미 분석됨?"}

    START --> CHECK1
    CHECK1 -->|No| ERR1["❌ RETRO4031<br/>권한 없음"]
    CHECK1 -->|Yes| CHECK2

    CHECK2 -->|No| ERR2["❌ AI4002<br/>데이터 부족"]
    CHECK2 -->|Yes| CHECK3

    CHECK3 -->|No| ERR3["❌ AI4031<br/>한도 초과"]
    CHECK3 -->|Yes| CHECK4

    CHECK4 -->|Yes| ERR4["❌ RETRO4091<br/>이미 분석됨"]
    CHECK4 -->|No| PROCEED["✅ 분석 진행"]
```

### 분석 조건 요약

| 조건 | 에러 코드 | 설명 |
|------|----------|------|
| Owner 권한 | RETRO4031 | 회고방 소유자만 분석 가능 |
| 제출된 답변 | AI4002 | 최소 1명 이상 제출 필요 |
| 월간 한도 | AI4031 | 월 10회 제한 |
| 중복 분석 | RETRO4091 | 회고당 1회만 분석 |

---

## 2️⃣ 데이터 수집

```mermaid
sequenceDiagram
    participant S as Service
    participant DB as Database

    S->>DB: SELECT retrospect
    DB-->>S: { title, method }

    S->>DB: SELECT member_retro<br/>WHERE status = 'SUBMITTED'
    DB-->>S: [participants]

    S->>DB: SELECT responses<br/>WHERE retrospect_id
    DB-->>S: [{ member, question, answer }]

    Note over S: 데이터 구조화
```

### 수집 데이터 구조

```json
{
  "retrospect": {
    "title": "스프린트 1 회고",
    "method": "KPT"
  },
  "participants": [
    {
      "memberId": 1,
      "nickname": "홍길동",
      "responses": [
        {
          "question": "Keep: 유지할 점은?",
          "content": "팀 커뮤니케이션이 잘 되었습니다..."
        }
      ]
    }
  ]
}
```

---

## 3️⃣ 프롬프트 구성

```mermaid
flowchart TB
    subgraph system["System Prompt"]
        S1["역할 정의"]
        S2["분석 지침"]
        S3["출력 형식"]
    end

    subgraph user["User Prompt"]
        U1["회고 제목"]
        U2["회고 방식"]
        U3["팀원 답변들"]
    end

    system --> COMBINE["프롬프트 조합"]
    user --> COMBINE
    COMBINE --> API["OpenAI API"]
```

### System Prompt 구조

```
당신은 회고 분석 전문가입니다.

분석 지침:
1. 팀 전체의 핵심 인사이트를 도출하세요
2. 팀원들의 감정을 분석하세요
3. 실행 가능한 미션을 제안하세요

출력 형식:
- JSON 형식으로 응답
- 한국어로 작성
- 구체적이고 실행 가능한 내용
```

### User Prompt 예시

```
회고 제목: 스프린트 1 회고
회고 방식: KPT

팀원 답변:
---
[홍길동]
Keep: 팀 커뮤니케이션이 잘 되었습니다...
Problem: 일정 관리가 어려웠습니다...
Try: 데일리 스크럼을 도입하고 싶습니다...
---
[김철수]
Keep: 코드 리뷰 문화가 좋았습니다...
...
```

---

## 4️⃣ OpenAI API 호출

```mermaid
sequenceDiagram
    participant S as AiService
    participant O as OpenAI API

    S->>O: POST /v1/chat/completions
    Note right of S: model: gpt-4o<br/>temperature: 0.7<br/>max_tokens: 4000

    alt ✅ 성공
        O-->>S: JSON Response
    else ⏱️ 타임아웃
        O-->>S: Timeout Error
        S->>S: AI5002 반환
    else ❌ API 에러
        O-->>S: Error Response
        S->>S: AI5001 반환
    end
```

### API 설정

| Parameter | Value | 설명 |
|-----------|-------|------|
| model | gpt-4o | 최신 모델 |
| temperature | 0.7 | 창의성 수준 |
| max_tokens | 4000 | 최대 응답 길이 |
| timeout | 60s | 타임아웃 |

---

## 5️⃣ 응답 파싱

```mermaid
flowchart TB
    RAW["Raw Response"]

    CLEAN["코드블록 제거"]
    PARSE["JSON 파싱"]
    VALIDATE["구조 검증"]

    RAW --> CLEAN --> PARSE --> VALIDATE

    VALIDATE -->|Valid| SUCCESS["✅ AnalysisResponse"]
    VALIDATE -->|Invalid| ERROR["❌ AI5001"]
```

### 응답 구조

```json
{
  "teamInsight": "이번 스프린트에서 팀은 커뮤니케이션 측면에서...",
  "emotionRank": [
    {
      "emotion": "성취감",
      "reason": "목표한 기능을 모두 완성했기 때문"
    },
    {
      "emotion": "피로감",
      "reason": "연속된 야근으로 인한 체력 소모"
    },
    {
      "emotion": "기대감",
      "reason": "다음 스프린트에 대한 새로운 도전"
    }
  ],
  "teamMissions": [
    {
      "mission": "데일리 스크럼 15분 제한",
      "description": "효율적인 미팅을 위해 시간을 엄수합니다"
    }
  ],
  "personalMissions": [
    {
      "memberId": 1,
      "nickname": "홍길동",
      "missions": [
        {
          "mission": "문서화 습관 기르기",
          "description": "작업 내용을 꼼꼼히 기록합니다"
        }
      ]
    }
  ]
}
```

---

## 6️⃣ 결과 저장

```mermaid
sequenceDiagram
    participant S as Service
    participant DB as Database

    Note over S,DB: 분석 결과 저장

    S->>DB: UPDATE retrospect<br/>SET team_insight = '...'

    loop 각 참여자
        S->>DB: UPDATE member_retro<br/>SET personal_insight = '...'<br/>SET status = 'ANALYZED'
    end

    S->>DB: UPDATE member<br/>SET insight_count += 1

    DB-->>S: OK
```

---

## 📊 분석 결과 활용

```mermaid
flowchart LR
    subgraph result["분석 결과"]
        TEAM["팀 인사이트"]
        EMOTION["감정 분석"]
        T_MISSION["팀 미션"]
        P_MISSION["개인 미션"]
    end

    subgraph usage["활용"]
        VIEW["회고 상세<br/>API-013"]
        EXPORT["PDF 내보내기<br/>API-021"]
        STORAGE["보관함<br/>API-019"]
    end

    result --> usage
```

---

## 🚨 에러 처리

| Code | HTTP | 상황 | 대응 |
|------|------|------|------|
| AI4002 | 400 | 데이터 부족 | 답변 제출 유도 |
| AI4031 | 403 | 월간 한도 초과 | 다음 달까지 대기 |
| AI5001 | 500 | 분석 실패 | 재시도 |
| AI5002 | 500 | 연결 실패 | 잠시 후 재시도 |
| AI5031 | 503 | 서비스 불가 | 관리자 문의 |

---

## 📈 분석 제한

```mermaid
flowchart LR
    subgraph limits["분석 제한"]
        MONTHLY["월간 10회"]
        ONCE["회고당 1회"]
        MIN["최소 1명 제출"]
    end
```

| 제한 | 값 | 설명 |
|------|---|------|
| 월간 한도 | 10회 | 사용자별 월 10회 |
| 회고당 | 1회 | 중복 분석 불가 |
| 최소 데이터 | 1명 | 제출된 답변 필요 |

---

## 🔗 Related

- [[00-HOME|🏠 HOME]]
- [[03-Retrospect-Flow|📝 Retrospect Flow]]
- [[apis/API-023 AI 분석|API-022 AI 분석]]
- [[09-Retrospect-APIs|📝 Retrospect APIs]]

---

#ai #openai #analysis #flow
