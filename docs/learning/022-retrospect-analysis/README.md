# API-022: 회고 AI 분석 (회고 종합 분석)

## 개요

`POST /api/v1/retrospects/{retrospectId}/analysis`

특정 회고 세션에 쌓인 모든 팀원의 답변을 종합 분석하여 **AI 인사이트**, **감정 통계**, **맞춤형 미션**을 생성하는 API이다.
OpenAI GPT-4o-mini 모델을 호출하여 팀원 답변을 자연어로 분석하고, 구조화된 JSON 응답을 파싱하여 DB에 저장한다.

## 관련 소스 파일

| 파일 | 역할 |
|------|------|
| `codes/server/src/domain/retrospect/handler.rs` | HTTP 핸들러 (`analyze_retrospective_handler`, 410-436행) |
| `codes/server/src/domain/retrospect/service.rs` | 비즈니스 로직 (`analyze_retrospective`, 1581-1841행) |
| `codes/server/src/domain/ai/service.rs` | AI 서비스 (OpenAI 호출 + 응답 파싱, 36-76행) |
| `codes/server/src/domain/ai/prompt.rs` | 프롬프트 템플릿 (`AnalysisPrompt`, 전체) |
| `codes/server/src/domain/retrospect/dto.rs` | 응답 DTO (`EmotionRankItem` 444-455행, `MissionItem` 458-465행, `PersonalMissionItem` 468-477행, `AnalysisResponse` 480-489행) |
| `codes/server/src/utils/error.rs` | 에러 타입 정의 (`RetroAlreadyAnalyzed` 91행, `AiMonthlyLimitExceeded` 94행 등, 91-109행) |

## API 스펙 문서

- `docs/api-specs/022-retrospect-analysis.md`

## 주요 기능 요약

1. **회고 존재 확인 및 이미 분석 여부 검사** (멱등성 보장 - 중복 분석 시 409 반환)
2. **팀 멤버십 확인** (접근 제어)
3. **월간 한도 체크** (팀당 월 10회, KST 기준 매월 1일 리셋)
4. **최소 데이터 기준 확인** (참여자 1명 이상, 답변 3개 이상)
5. **멤버별 답변 데이터 수집** 및 AI 프롬프트 구성
6. **OpenAI API 호출** (GPT-4o-mini, 30초 타임아웃)
7. **JSON 응답 파싱** 및 검증 (감정 3개, 미션 각 3개)
8. **트랜잭션 기반 DB 저장** (`team_insight`, `personal_insight`, 상태 변경)

## 응답 구조

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "회고 분석이 성공적으로 완료되었습니다.",
  "result": {
    "teamInsight": "팀 전체 인사이트 메시지",
    "emotionRank": [
      { "rank": 1, "label": "피로", "description": "원인 설명", "count": 6 },
      { "rank": 2, "label": "뿌듯", "description": "원인 설명", "count": 4 },
      { "rank": 3, "label": "불안", "description": "원인 설명", "count": 2 }
    ],
    "personalMissions": [
      {
        "userId": 1,
        "userName": "소은",
        "missions": [
          { "missionTitle": "미션 제목", "missionDesc": "미션 설명" }
        ]
      }
    ]
  }
}
```

## 에러 코드 (실제 구현 기준)

| 에러 코드 | HTTP 상태 | AppError 타입 | 설명 |
|-----------|----------|--------------|------|
| RETRO4041 | 404 Not Found | `RetrospectNotFound` | 존재하지 않는 회고 |
| RETRO4091 | 409 Conflict | `RetroAlreadyAnalyzed` | 이미 분석 완료된 회고 |
| TEAM4031 | 403 Forbidden | `TeamAccessDenied` | 팀 접근 권한 없음 |
| AI4031 | 403 Forbidden | `AiMonthlyLimitExceeded` | 월간 분석 한도 초과 |
| RETRO4221 | 422 Unprocessable Entity | `RetroInsufficientData` | 분석 데이터 부족 |
| AI5001 | 500 Internal Server Error | `AiAnalysisFailed` | AI 분석/파싱 실패 |
| AI5002 | 500 Internal Server Error | `AiConnectionFailed` | AI API 키 무효 |
| AI5031 | 503 Service Unavailable | `AiServiceUnavailable` | AI 타임아웃/과부하 |
| AI5003 | 500 Internal Server Error | `AiGeneralError` | 기타 AI 오류 |

## 스펙 문서 vs 실제 구현 차이점

API 스펙 문서(`docs/api-specs/022-retrospect-analysis.md`)와 실제 구현 사이에 다음과 같은 차이가 존재한다.

### 1. 데이터 부족 에러 코드

| 항목 | 스펙 문서 | 실제 구현 |
|------|----------|----------|
| 에러 코드 | `RETRO4042` | `RETRO4221` |
| HTTP 상태 | 404 Not Found | 422 Unprocessable Entity |
| 설명 | 스펙은 "데이터 부족"을 404로 분류 | 구현은 의미론적으로 더 정확한 422 사용 |

**근거**: `error.rs:198` (`RetroInsufficientData` -> `RETRO4221`), `error.rs:239` (`UNPROCESSABLE_ENTITY`)

스펙에서는 최소 데이터 기준 미달 시 `RETRO4042` (404)를 반환한다고 명시하지만, 실제 구현에서는 `AppError::RetroInsufficientData`가 `RETRO4221` (422 Unprocessable Entity)를 반환한다. 422가 "요청은 올바르지만 처리할 수 없는 엔터티"라는 의미이므로, 데이터 부족 상황에 더 적합한 HTTP 시맨틱이다.

### 2. AI 에러 코드 세분화

| 항목 | 스펙 문서 | 실제 구현 |
|------|----------|----------|
| AI 분석 실패 | `AI5001` (500) | `AI5001` (500) -- 동일 |
| API 키 무효 | 미정의 (AI5001에 통합) | `AI5002` (500) |
| 서비스 타임아웃/과부하 | 미정의 (AI5001에 통합) | `AI5031` (503) |
| 기타 AI 오류 | 미정의 (AI5001에 통합) | `AI5003` (500) |

**근거**: `error.rs:199-202`, `error.rs:240-243`

스펙 문서는 모든 AI 관련 에러를 `AI5001` (500)로 통합하고 있지만, 실제 구현에서는 4가지 에러 타입으로 세분화되어 있다. 특히 `AiServiceUnavailable`은 503 상태 코드를 반환하여 클라이언트가 재시도 가능한 에러를 구분할 수 있도록 한다.

### 3. 이미 분석 완료 에러 (스펙 누락)

| 항목 | 스펙 문서 | 실제 구현 |
|------|----------|----------|
| 에러 코드 | 에러 코드 요약 테이블에 미포함 | `RETRO4091` |
| HTTP 상태 | -- | 409 Conflict |
| 설명 | 스펙 에러 테이블에 이미 분석된 회고 재분석 시나리오 누락 | 구현은 `team_insight IS NOT NULL` 검사로 중복 분석 방지 |

**근거**: `error.rs:196` (`RetroAlreadyAnalyzed` -> `RETRO4091`), `service.rs:1609-1612`

스펙 문서의 에러 코드 요약 테이블(`AUTH4001`, `AI4031`, `RETRO4041`, `RETRO4042`, `AI5001`)에 `RETRO4091` (이미 분석 완료)이 포함되어 있지 않다. 실제 구현에서는 `retrospects.team_insight`가 `NOT NULL`이면 409 Conflict를 반환하여 멱등성을 보장한다.

### 4. 팀 접근 권한 에러 (스펙 누락)

| 항목 | 스펙 문서 | 실제 구현 |
|------|----------|----------|
| 에러 코드 | 에러 코드 요약 테이블에 미포함 | `TEAM4031` |
| HTTP 상태 | -- | 403 Forbidden |
| 설명 | 스펙 에러 테이블에 팀 접근 권한 검사 누락 | 구현은 `member_team` 테이블로 멤버십 확인 |

**근거**: `error.rs:187` (`TeamAccessDenied` -> `TEAM4031`), `service.rs:1615-1627`

스펙 문서의 에러 코드 요약 테이블에 `TEAM4031` (팀 접근 권한 없음)이 포함되어 있지 않다. 실제 구현에서는 요청자가 해당 회고의 팀 멤버가 아니면 403 Forbidden을 반환한다.

### 5. 핸들러의 사용자 ID 추출 방식

| 항목 | 스펙 문서 | 실제 구현 |
|------|----------|----------|
| 방식 | 명시되지 않음 | `user.0.sub.parse::<i64>()` |
| 비고 | -- | 다른 핸들러는 `user.user_id()`를 사용하지만, API-022 핸들러는 직접 파싱 |

**근거**: `handler.rs:423-427`

API-022의 `analyze_retrospective_handler`는 `user.0.sub.parse()`로 사용자 ID를 직접 추출한다. 같은 파일의 다른 핸들러들(`create_retrospect`, `list_team_retrospects` 등)이 `user.user_id()?` 헬퍼 메서드를 사용하는 것과 일관성이 없다. 기능적으로는 동일하지만 코드 스타일 차이가 있다.

## 학습 문서

- [flow.md](./flow.md) - 동작 흐름 상세
- [key-concepts.md](./key-concepts.md) - 핵심 개념 정리
- [keywords.md](./keywords.md) - 학습 키워드
