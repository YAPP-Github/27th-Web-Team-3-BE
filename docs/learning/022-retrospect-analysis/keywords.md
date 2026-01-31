# 학습 키워드: 회고 AI 분석 (API-022)

## 1. async-openai (크레이트)

**정의**: OpenAI API의 Rust 비동기 클라이언트 라이브러리이다.

**사용 위치**: `ai/service.rs:3-9`

```rust
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
    Client,
};
```

**관련 구조체**:
- `OpenAIConfig` -- API 키 등 설정 (`ai/service.rs:28`)
- `Client<OpenAIConfig>` -- HTTP 클라이언트 (`ai/service.rs:22`)
- `ChatCompletionRequestMessage` -- 메시지(system/user/assistant) 타입
- `CreateChatCompletionRequestArgs` -- 요청 빌더 (`ai/service.rs:111-117`)

---

## 2. reqwest (간접 사용)

**정의**: Rust의 비동기 HTTP 클라이언트이다. `async-openai` 내부에서 실제 HTTP 요청을 처리한다.

**사용 방식**: 직접 `reqwest`를 호출하지 않고 `async-openai`의 `Client`가 내부적으로 사용한다. `call_openai` 메서드(`ai/service.rs:93-147`)에서 `chat.create(request)` 호출 시 `reqwest` 기반 HTTP POST가 실행된다.

---

## 3. serde_json::from_str

**정의**: JSON 문자열을 Rust 구조체로 역직렬화(Deserialize)하는 함수이다.

**사용 위치**: `ai/service.rs:49`

```rust
let analysis: AnalysisResponse = serde_json::from_str(json_str).map_err(|e| {
    warn!("AI 응답 JSON 파싱 실패: {}", e);
    AppError::AiAnalysisFailed(format!("AI 응답을 파싱할 수 없습니다: {}", e))
})?;
```

**관련 개념**:
- `Deserialize` trait -- 구조체에 `#[derive(Deserialize)]`로 적용 (`dto.rs:480`)
- `#[serde(rename_all = "camelCase")]` -- JSON 키 규칙 자동 변환 (`dto.rs:481`)
- `Result<T, serde_json::Error>` -- 파싱 실패 시 에러 타입

---

## 4. team_insight (DB 컬럼)

**정의**: `retrospects` 테이블의 컬럼으로, AI 분석 결과의 팀 인사이트 텍스트를 저장한다.

**사용 위치**:
- 분석 완료 여부 판별: `service.rs:1609` (`retrospect_model.team_insight.is_some()`)
- 분석 결과 저장: `service.rs:1804` (`retrospect_active.team_insight = Set(Some(team_insight.clone()))`)
- 월간 한도 카운트: `service.rs:1644` (`retrospect::Column::TeamInsight.is_not_null()`)

**역할**:
- `NULL` -> 분석 미완료 상태
- `Some(텍스트)` -> 분석 완료 상태
- 별도 플래그 없이 실제 데이터 유무로 상태를 관리하는 패턴이다.

---

## 5. personal_insight (DB 컬럼)

**정의**: `member_retro` 테이블의 컬럼으로, 각 참여자의 개인 미션 분석 결과를 텍스트로 저장한다.

**사용 위치**: `service.rs:1826`

```rust
mr_active.personal_insight = Set(personal_insight);
```

**저장 형식**: "미션제목: 미션설명\n미션제목: 미션설명\n미션제목: 미션설명" (`service.rs:1818-1822`)

---

## 6. AppError::AiAnalysisFailed

**정의**: AI 분석 과정에서 발생한 오류를 나타내는 에러 타입이다.

**에러 코드**: `AI5001`
**HTTP 상태**: `500 Internal Server Error`

**발생 위치**:
- JSON 파싱 실패: `ai/service.rs:52`
- 감정 랭킹 개수 불일치: `ai/service.rs:57-60`
- 개인 미션 개수 불일치: `ai/service.rs:66-70`

**정의 위치**: `error.rs:99-100`

```rust
/// AI5001: 데이터 종합 분석 중 오류 (500)
AiAnalysisFailed(String),

/// AI5002: AI 연결 실패 (500)
AiConnectionFailed(String),

/// AI5031: AI 서비스 일시적 오류 (503)
AiServiceUnavailable(String),

/// AI5003: AI 일반 오류 (500)
AiGeneralError(String),
```

**참고**: 실제 구현에서는 AI 에러가 4가지로 세분화되어 있다 (`error.rs:99-109`). 스펙 문서에서는 `AI5001` 하나로 통합되어 있으나, 구현에서는 원인별로 에러 코드가 다르다. 자세한 비교는 [README.md](./README.md)의 "스펙 문서 vs 실제 구현 차이점" 섹션을 참고한다.

---

## 7. AppError::RetroAlreadyAnalyzed

**정의**: 이미 분석 완료된 회고에 대해 재분석을 시도할 때 발생하는 에러 타입이다.

**에러 코드**: `RETRO4091`
**HTTP 상태**: `409 Conflict`

**발생 위치**: `service.rs:1610-1612`

```rust
return Err(AppError::RetroAlreadyAnalyzed(
    "이미 분석이 완료된 회고입니다.".to_string(),
));
```

**정의 위치**: `error.rs:90-91`

```rust
/// RETRO4091: 이미 분석 완료된 회고 (409)
RetroAlreadyAnalyzed(String),
```

---

## 8. AppError::AiMonthlyLimitExceeded

**정의**: 팀의 월간 AI 분석 횟수가 제한(10회)을 초과했을 때 발생하는 에러 타입이다.

**에러 코드**: `AI4031`
**HTTP 상태**: `403 Forbidden`

**발생 위치**: `service.rs:1652-1654`

```rust
return Err(AppError::AiMonthlyLimitExceeded(
    "월간 분석 가능 횟수를 초과하였습니다.".to_string(),
));
```

**정의 위치**: `error.rs:93-94`

```rust
/// AI4031: 월간 분석 가능 횟수 초과 (403)
AiMonthlyLimitExceeded(String),
```

---

## 9. AppError::RetroInsufficientData

**정의**: AI 분석을 수행하기 위한 최소 데이터 기준(참여자 1명 이상, 답변 3개 이상)을 충족하지 못할 때 발생하는 에러 타입이다.

**에러 코드**: `RETRO4221`
**HTTP 상태**: `422 Unprocessable Entity`

**발생 위치**:
- 제출 참여자 0명: `service.rs:1670-1672`
- 유효 답변 < 3개: `service.rs:1688-1690`

**정의 위치**: `error.rs:96-97`

```rust
/// RETRO4221: 분석할 회고 답변 데이터 부족 (422)
RetroInsufficientData(String),
```

**스펙 차이**: API 스펙 문서(`022-retrospect-analysis.md`)에서는 이 에러를 `RETRO4042` (404)로 정의하고 있으나, 실제 구현은 `RETRO4221` (422 Unprocessable Entity)를 사용한다. 422가 의미론적으로 더 정확하다.

---

## 10. MemberAnswerData (구조체)

**정의**: AI 프롬프트에 입력할 참여자별 답변 데이터를 담는 구조체이다.

**정의 위치**: `prompt.rs:5-9`

```rust
pub struct MemberAnswerData {
    pub user_id: i64,
    pub user_name: String,
    pub answers: Vec<(String, String)>, // (질문, 답변)
}
```

**사용 위치**:
- 데이터 조립: `service.rs:1749-1775`
- 프롬프트 생성: `prompt.rs:111` (`AnalysisPrompt::user_prompt(members_data)`)
- AI 서비스 호출: `ai/service.rs:38` (`analyze_retrospective(&self, members_data: &[MemberAnswerData])`)

---

## 11. AnalysisResponse (DTO)

**정의**: AI 분석 결과를 담는 응답 DTO이다. OpenAI 응답에서 역직렬화되고, API 응답으로 직렬화된다.

**정의 위치**: `dto.rs:480-489`

```rust
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisResponse {
    /// 팀 전체를 위한 AI 분석 메시지
    pub team_insight: String,
    /// 감정 키워드 순위 리스트 (내림차순 정렬, 정확히 3개)
    pub emotion_rank: Vec<EmotionRankItem>,
    /// 사용자별 개인 맞춤 미션 리스트 (userId 오름차순 정렬)
    pub personal_missions: Vec<PersonalMissionItem>,
}
```

**특징**: `Serialize`와 `Deserialize` 모두 구현되어 있다.
- `Deserialize`: AI 응답 JSON을 이 구조체로 파싱 (`ai/service.rs:49`)
- `Serialize`: API 응답으로 클라이언트에 전달 (`handler.rs:432`)

---

## 12. EmotionRankItem (DTO)

**정의**: 감정 랭킹의 개별 항목을 나타내는 구조체이다.

**정의 위치**: `dto.rs:444-455`

```rust
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EmotionRankItem {
    /// 순위 (1부터 시작, 감정 빈도 기준 내림차순)
    pub rank: i32,
    /// 감정 키워드 (예: "피로", "뿌듯")
    pub label: String,
    /// 해당 감정에 대한 상세 설명 및 원인 분석
    pub description: String,
    /// 해당 감정을 선택/언급한 횟수
    pub count: i32,
}
```

**검증**: 정확히 3개여야 한다 (`ai/service.rs:56-61`).

---

## 13. PersonalMissionItem (DTO)

**정의**: 사용자별 개인 미션 목록을 나타내는 구조체이다.

**정의 위치**: `dto.rs:468-477`

```rust
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PersonalMissionItem {
    /// 사용자 고유 ID
    pub user_id: i64,
    /// 사용자 이름
    pub user_name: String,
    /// 해당 사용자의 개인 미션 리스트 (정확히 3개)
    pub missions: Vec<MissionItem>,
}
```

**검증**: 각 사용자의 `missions`는 정확히 3개여야 한다 (`ai/service.rs:64-71`).
**정렬**: AI 서비스 호출 후 `service.rs:1790`에서 `userId` 오름차순으로 정렬한다.

---

## 14. MissionItem (DTO)

**정의**: 개별 미션의 제목과 설명을 담는 구조체이다.

**정의 위치**: `dto.rs:458-465`

```rust
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MissionItem {
    /// 개인 미션 제목 (예: "감정 표현 적극적으로 하기")
    pub mission_title: String,
    /// 개인 미션 상세 설명 및 인사이트
    pub mission_desc: String,
}
```

---

## 15. AnalysisPrompt (구조체)

**정의**: 회고 분석용 프롬프트를 생성하는 정적 메서드를 모아둔 구조체이다.

**정의 위치**: `prompt.rs:2`

**메서드**:
- `system_prompt()` -> `String` (`prompt.rs:13-108`): 시스템 프롬프트 생성
- `user_prompt(members_data: &[MemberAnswerData])` -> `String` (`prompt.rs:111-137`): 사용자 프롬프트 생성

---

## 16. tokio::time::timeout

**정의**: 비동기 작업에 시간 제한을 거는 `tokio` 유틸리티 함수이다.

**사용 위치**: `ai/service.rs:121`

```rust
let response = tokio::time::timeout(Duration::from_secs(30), api_call)
    .await
    .map_err(|_| {
        AppError::AiServiceUnavailable("AI 서비스 응답 시간이 초과되었습니다".to_string())
    })?
```

**반환 타입**: `Result<T, tokio::time::error::Elapsed>` -- 타임아웃 시 `Elapsed` 에러이다.

---

## 17. extract_json (헬퍼 함수)

**정의**: AI 응답 텍스트에서 JSON 부분만 추출하는 정적 메서드이다.

**정의 위치**: `ai/service.rs:79-90`

**동작**: `str::find('{')` ~ `str::rfind('}')` 범위를 슬라이싱하여 코드 블록(```json ... ```)이나 부가 텍스트를 제거한다.

---

## 18. RetrospectStatus::Analyzed

**정의**: 회고 참여자의 상태를 나타내는 열거형 값으로, AI 분석이 완료된 상태이다.

**사용 위치**:
- 제출 완료 멤버 조회 필터: `service.rs:1663` (`.is_in([RetrospectStatus::Submitted, RetrospectStatus::Analyzed])`)
- 분석 완료 후 상태 변경: `service.rs:1827` (`mr_active.status = Set(RetrospectStatus::Analyzed)`)

---

## 19. retrospect_room_id

**정의**: 팀(레트로룸) 식별자로, 월간 한도를 팀 단위로 관리하기 위해 사용한다.

**사용 위치**: `service.rs:1629` (`let retrospect_room_id = retrospect_model.retrospect_room_id`)

**월간 한도 쿼리에서의 역할**: `service.rs:1643` (`retrospect::Column::RetrospectRoomId.eq(retrospect_room_id)`) -- 같은 팀 내 모든 회고를 대상으로 분석 횟수를 카운트한다.

---

## 20. #[instrument] (tracing 매크로)

**정의**: `tracing` 크레이트의 함수 계측 매크로로, 함수 호출 시 자동으로 span을 생성한다.

**사용 위치**: `ai/service.rs:35`

```rust
#[instrument(skip(self, members_data), fields(member_count = members_data.len()))]
pub async fn analyze_retrospective(
    &self,
    members_data: &[MemberAnswerData],
) -> Result<AnalysisResponse, AppError> {
```

**기능**:
- `skip(self, members_data)` -- 로깅에서 큰 데이터를 제외한다.
- `fields(member_count = ...)` -- 커스텀 필드를 span에 추가한다.
