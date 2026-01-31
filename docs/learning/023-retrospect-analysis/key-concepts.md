# 핵심 개념: 회고 AI 분석 (API-022)

## 1. 외부 API(OpenAI) 연동

### 개념

외부 AI 서비스(OpenAI ChatCompletion API)를 호출하여 자연어 분석 결과를 얻는 패턴이다.
이 프로젝트에서는 `async-openai` 크레이트를 사용하여 Rust에서 OpenAI API를 호출한다.

### 구현 위치

- `ai/service.rs:1-32` -- `AiService` 구조체 정의 및 클라이언트 초기화
- `ai/service.rs:93-147` -- `call_openai` 메서드 (실제 API 호출)

### 핵심 코드

```rust
// 클라이언트 생성 (ai/service.rs:27-31)
pub fn new(config: &AppConfig) -> Self {
    let openai_config = OpenAIConfig::new().with_api_key(&config.openai_api_key);
    let client = Client::with_config(openai_config);
    Self { client }
}

// API 호출 (ai/service.rs:111-117)
let request = CreateChatCompletionRequestArgs::default()
    .model("gpt-4o-mini")
    .messages(messages)
    .temperature(0.7)
    .max_tokens(4000u32)
    .build()?;
```

### 학습 포인트

- `async-openai` 크레이트는 OpenAI API의 Rust 비동기 클라이언트이다.
- `OpenAIConfig`로 API 키를 설정하고 `Client::with_config`로 클라이언트를 생성한다.
- `temperature(0.7)`: 창의성과 일관성의 균형을 조정하는 파라미터이다 (0.0=확정적, 1.0=창의적).
- `max_tokens(4000)`: 응답 최대 토큰 수를 제한하여 비용을 제어한다.

---

## 2. 타임아웃 처리

### 개념

외부 API 호출 시 응답 지연에 대비하여 타임아웃을 설정한다.
`tokio::time::timeout`을 사용하여 비동기 작업에 시간 제한을 둔다.

### 구현 위치

- `ai/service.rs:121-126`

### 핵심 코드

```rust
let response = tokio::time::timeout(Duration::from_secs(30), api_call)
    .await
    .map_err(|_| {
        AppError::AiServiceUnavailable("AI 서비스 응답 시간이 초과되었습니다".to_string())
    })?
```

### 학습 포인트

- `tokio::time::timeout`은 Future에 시간 제한을 걸어 `Elapsed` 에러를 반환한다.
- 30초 타임아웃은 OpenAI API의 일반적인 응답 시간(수 초~십수 초)에 여유를 둔 설정이다.
- 타임아웃 발생 시 `AiServiceUnavailable` (503) 에러로 변환하여 클라이언트에 적절한 상태를 전달한다.

---

## 3. 프롬프트 엔지니어링 구조

### 개념

AI 모델에게 원하는 출력을 얻기 위해 체계적으로 프롬프트를 구성하는 방법이다.
시스템 프롬프트와 사용자 프롬프트로 분리하여 역할 설정과 데이터 입력을 구분한다.

### 구현 위치

- `prompt.rs:11-108` -- 시스템 프롬프트 (역할, 규칙, 출력 형식 정의)
- `prompt.rs:111-137` -- 사용자 프롬프트 (실제 데이터 포함)

### 프롬프트 구성 전략

| 구성 요소 | 내용 | 위치 |
|-----------|------|------|
| AI 역할 설정 | "따뜻한 AI 분석가" | `prompt.rs:14` |
| 말투 규칙 | 상냥체(~어요) 필수, 격식체(~습니다) 금지 | `prompt.rs:17-23` |
| 분석 항목 정의 | teamInsight, emotionRank, personalMissions | `prompt.rs:27-47` |
| 출력 형식 | JSON 스키마 예시 | `prompt.rs:52-97` |
| 엄격한 규칙 | 정확히 3개, count 내림차순 등 | `prompt.rs:99-107` |
| 데이터 입력 | 참여자별 Q&A 나열 | `prompt.rs:111-137` |

### 학습 포인트

- **시스템 프롬프트**: AI의 페르소나, 행동 규칙, 출력 포맷을 정의한다.
- **사용자 프롬프트**: 실제 분석 대상 데이터를 전달한다.
- "좋은 예"/"나쁜 예"를 함께 제시하면 AI가 원하는 출력 스타일을 더 정확히 이해한다.
- JSON 스키마를 명시적으로 보여주면 파싱 가능한 구조화된 응답을 얻을 확률이 높아진다.
- `MemberAnswerData` 구조체(`prompt.rs:5-9`)를 통해 프롬프트 생성에 필요한 데이터를 타입 안전하게 전달한다.

---

## 4. JSON 응답 파싱

### 개념

AI 모델의 자유 형식 텍스트 응답에서 구조화된 JSON 데이터를 추출하고 Rust 구조체로 역직렬화한다.

### 구현 위치

- `ai/service.rs:79-90` -- JSON 추출 (`extract_json`)
- `ai/service.rs:49-53` -- `serde_json::from_str`으로 파싱
- `dto.rs:480-489` -- `AnalysisResponse` 구조체 (`Serialize` + `Deserialize` 구현)

### 핵심 코드

```rust
// 코드 블록 제거 (ai/service.rs:79-90)
fn extract_json(response: &str) -> &str {
    let trimmed = response.trim();
    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            return &trimmed[start..=end];
        }
    }
    trimmed
}

// 파싱 (ai/service.rs:49-53)
let analysis: AnalysisResponse = serde_json::from_str(json_str).map_err(|e| {
    warn!("AI 응답 JSON 파싱 실패: {}", e);
    AppError::AiAnalysisFailed(format!("AI 응답을 파싱할 수 없습니다: {}", e))
})?;
```

### 학습 포인트

- AI가 ```json ... ``` 코드 블록으로 감싸 응답할 수 있으므로, 첫 `{`부터 마지막 `}`까지를 슬라이싱하여 추출한다.
- `serde_json::from_str`은 JSON 문자열을 Rust 구조체로 역직렬화한다.
- `#[serde(rename_all = "camelCase")]` 어트리뷰트(`dto.rs:481`)가 JSON의 camelCase 키를 Rust의 snake_case 필드로 자동 매핑한다.
- 파싱 실패 시 원본 응답을 `warn!`으로 로깅하여 디버깅을 돕는다.

---

## 5. 멱등성 (Idempotency) -- 이미 분석 시 409 반환

### 개념

동일한 요청을 여러 번 보내도 결과가 변하지 않도록 보장하는 설계 원칙이다.
이 API에서는 이미 분석 완료된 회고에 대한 재요청을 409 Conflict로 거부한다.

### 구현 위치

- `service.rs:1608-1613`

### 핵심 코드

```rust
if retrospect_model.team_insight.is_some() {
    return Err(AppError::RetroAlreadyAnalyzed(
        "이미 분석이 완료된 회고입니다.".to_string(),
    ));
}
```

### 학습 포인트

- `team_insight` 컬럼의 NULL/NOT NULL 상태를 분석 완료 여부의 판별 기준으로 사용한다.
- 별도의 `is_analyzed` 플래그 없이 실제 데이터 존재 여부로 판단하는 실용적 접근이다.
- 409 Conflict 상태 코드는 "현재 리소스 상태와 요청이 충돌"할 때 사용하는 적절한 HTTP 시맨틱이다.
- 에러 타입 `RetroAlreadyAnalyzed`는 `error.rs:90-91`에 정의되어 있다.

---

## 6. 월간 한도 관리

### 개념

AI API 호출 비용을 제어하기 위해 팀당 월간 분석 횟수를 제한한다.
시간 기준은 KST(한국 표준시)이며, 매월 1일 00:00 KST에 리셋된다.

### 구현 위치

- `service.rs:1631-1655`

### 핵심 코드

```rust
// KST 기준 현재 월 시작 시점 계산 (service.rs:1632-1639)
let kst_offset = chrono::Duration::hours(9);
let now_kst = Utc::now().naive_utc() + kst_offset;
let current_month_start =
    chrono::NaiveDate::from_ymd_opt(now_kst.year(), now_kst.month(), 1)
        .ok_or_else(|| AppError::InternalError("날짜 계산 오류".to_string()))?
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| AppError::InternalError("시간 계산 오류".to_string()))?
        - kst_offset; // UTC로 변환

// 해당 팀(retrospect_room_id)의 월간 분석 수 카운트 (service.rs:1642-1649)
let monthly_analysis_count = retrospect::Entity::find()
    .filter(retrospect::Column::RetrospectRoomId.eq(retrospect_room_id))
    .filter(retrospect::Column::TeamInsight.is_not_null())
    .filter(retrospect::Column::UpdatedAt.gte(current_month_start))
    .count(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?
    as i32;
```

### 학습 포인트

- KST 시간 계산: UTC에 +9시간을 더한 후 월초를 구하고, 다시 UTC로 변환하여 DB 쿼리에 사용한다.
- `retrospect_room_id`를 팀 식별자로 사용하여 팀 단위 한도를 관리한다.
- `team_insight IS NOT NULL && updated_at >= 월초` 조건으로 분석 완료 건만 카운트한다.
- 별도의 카운터 테이블 없이 기존 데이터의 상태를 활용하는 간결한 설계이다.
- 월간 한도는 **10회**이며, 초과 시 `AiMonthlyLimitExceeded` (AI4031, 403) 에러를 반환한다.

---

## 7. 응답 검증 (AI 출력 신뢰성 확보)

### 개념

AI 모델의 출력은 항상 기대한 형식과 일치하지 않을 수 있다.
파싱 성공 후에도 비즈니스 규칙에 맞는지 추가 검증하여 데이터 무결성을 보장한다.

### 구현 위치

- `ai/service.rs:55-72`

### 핵심 코드

```rust
// 감정 랭킹 3개 검증
if analysis.emotion_rank.len() != 3 {
    return Err(AppError::AiAnalysisFailed(format!(
        "감정 랭킹이 3개여야 하지만 {}개입니다",
        analysis.emotion_rank.len()
    )));
}

// 각 사용자 미션 3개 검증
for pm in &analysis.personal_missions {
    if pm.missions.len() != 3 {
        return Err(AppError::AiAnalysisFailed(format!(
            "사용자 {}의 미션이 3개여야 하지만 {}개입니다",
            pm.user_id, pm.missions.len()
        )));
    }
}
```

### 학습 포인트

- AI 출력을 맹목적으로 신뢰하지 않고, 비즈니스 규칙에 맞는지 반드시 검증한다.
- 검증 실패 시 구체적인 메시지(기대값 vs 실제값)를 포함하여 디버깅을 용이하게 한다.
- 프롬프트에 "반드시 정확히 3개"라고 명시하더라도, 코드 레벨 검증은 필수이다.

---

## 8. 트랜잭션을 통한 원자적 DB 저장

### 개념

AI 분석 결과를 여러 테이블에 걸쳐 저장할 때, 트랜잭션으로 원자성을 보장한다.
한 부분이 실패하면 전체 변경이 롤백되어 데이터 불일치를 방지한다.

### 구현 위치

- `service.rs:1795-1836`

### 저장 대상

| 테이블 | 업데이트 내용 | 위치 |
|--------|-------------|------|
| `retrospects` | `team_insight` 저장, `updated_at` 갱신 | `service.rs:1802-1809` |
| `member_retro` (각 참여자) | `personal_insight` 저장, `status` -> `Analyzed` | `service.rs:1812-1832` |

### 학습 포인트

- SeaORM의 `state.db.begin()`으로 트랜잭션을 시작한다.
- `ActiveModel` 패턴으로 특정 컬럼만 업데이트한다.
- `personal_insight`는 미션 3개를 "제목: 설명\n" 형식으로 결합한 텍스트로 저장한다.
- 모든 업데이트 완료 후 `txn.commit()`으로 한 번에 커밋한다.
- 중간에 에러 발생 시 자동으로 롤백되어 데이터 일관성이 유지된다.

---

## 9. 에러 분류 체계

### 개념

API-022에서 발생할 수 있는 에러들을 세분화된 에러 타입으로 분류하여, 클라이언트가 에러 원인을 정확히 파악할 수 있게 한다.

### 구현 위치

- `error.rs:91-109` (에러 타입 정의)
- `error.rs:196-202` (에러 코드 매핑)
- `error.rs:237-243` (HTTP 상태 코드 매핑)

### API-022 전용 에러 타입

| 에러 타입 | 에러 코드 | HTTP | 의미 |
|-----------|----------|------|------|
| `RetroAlreadyAnalyzed` | RETRO4091 | 409 | 이미 분석 완료 (멱등성) |
| `AiMonthlyLimitExceeded` | AI4031 | 403 | 월간 한도 초과 |
| `RetroInsufficientData` | RETRO4221 | 422 | 데이터 부족 |
| `AiAnalysisFailed` | AI5001 | 500 | AI 분석/파싱 실패 |
| `AiConnectionFailed` | AI5002 | 500 | API 키 무효 |
| `AiServiceUnavailable` | AI5031 | 503 | AI 서비스 타임아웃/과부하 |
| `AiGeneralError` | AI5003 | 500 | 기타 AI 오류 |

### 학습 포인트

- 도메인별 에러 접두사 사용: `RETRO`(회고), `AI`(AI 서비스), `TEAM`(팀)
- HTTP 상태 코드를 의미에 맞게 세분화: 409(충돌), 403(권한/한도), 422(데이터 부족), 500(서버 오류), 503(서비스 불가)
- `IntoResponse` 구현으로 에러가 자동으로 적절한 HTTP 응답으로 변환된다 (`error.rs:252-287`).
