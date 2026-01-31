# 동작 흐름: 회고 AI 분석 (API-022)

## 전체 흐름 다이어그램

```
클라이언트 → 핸들러 → 서비스 → [검증 단계들] → AI 서비스 → OpenAI API → 응답 파싱 → DB 트랜잭션 저장 → 응답 반환
```

---

## 1단계: HTTP 핸들러 진입

**소스**: `handler.rs:410-436`

```rust
pub async fn analyze_retrospective_handler(
    user: AuthUser,
    State(state): State<AppState>,
    Path(retrospect_id): Path<i64>,
) -> Result<Json<BaseResponse<AnalysisResponse>>, AppError> {
```

- `AuthUser` 미들웨어로 JWT 토큰에서 사용자 정보 추출
- `retrospect_id` Path Parameter 검증 (1 이상 양수)
- `user.0.sub`에서 `user_id`를 `i64`로 파싱 (`handler.rs:423-427`)
- 이후 `RetrospectService::analyze_retrospective()` 호출

---

## 2단계: 회고 존재 확인

**소스**: `service.rs:1599-1606`

```rust
let retrospect_model = retrospect::Entity::find_by_id(retrospect_id)
    .one(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?
    .ok_or_else(|| {
        AppError::RetrospectNotFound("존재하지 않는 회고 세션입니다.".to_string())
    })?;
```

- `retrospect_id`로 DB에서 회고 레코드를 조회한다.
- 존재하지 않으면 `RETRO4041` (404) 에러를 반환한다.

---

## 3단계: 이미 분석 완료 여부 확인 (멱등성)

**소스**: `service.rs:1608-1613`

```rust
if retrospect_model.team_insight.is_some() {
    return Err(AppError::RetroAlreadyAnalyzed(
        "이미 분석이 완료된 회고입니다.".to_string(),
    ));
}
```

- `retrospects.team_insight` 컬럼이 `Some` (NOT NULL)이면 이미 분석된 것이다.
- 재분석 시도 시 `RETRO4091` (409 Conflict) 에러를 반환한다.
- 이 방식으로 **멱등성**을 보장한다 -- 동일 회고에 대한 중복 분석을 방지한다.

---

## 4단계: 팀 멤버십 확인

**소스**: `service.rs:1615-1627`

```rust
let is_team_member = member_team::Entity::find()
    .filter(member_team::Column::MemberId.eq(user_id))
    .filter(member_team::Column::TeamId.eq(retrospect_model.team_id))
    .one(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

if is_team_member.is_none() {
    return Err(AppError::TeamAccessDenied(
        "해당 회고에 접근 권한이 없습니다.".to_string(),
    ));
}
```

- `member_team` 테이블에서 사용자가 해당 팀의 멤버인지 확인한다.
- 팀 멤버가 아니면 `TEAM4031` (403) 에러를 반환한다.

---

## 5단계: 월간 한도 체크

**소스**: `service.rs:1631-1655`

```rust
let kst_offset = chrono::Duration::hours(9);
let now_kst = Utc::now().naive_utc() + kst_offset;
let current_month_start =
    chrono::NaiveDate::from_ymd_opt(now_kst.year(), now_kst.month(), 1)
        .ok_or_else(|| AppError::InternalError("날짜 계산 오류".to_string()))?
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| AppError::InternalError("시간 계산 오류".to_string()))?
        - kst_offset; // UTC로 변환

let monthly_analysis_count = retrospect::Entity::find()
    .filter(retrospect::Column::RetrospectRoomId.eq(retrospect_room_id))
    .filter(retrospect::Column::TeamInsight.is_not_null())
    .filter(retrospect::Column::UpdatedAt.gte(current_month_start))
    .count(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?
    as i32;

if monthly_analysis_count >= 10 {
    return Err(AppError::AiMonthlyLimitExceeded(
        "월간 분석 가능 횟수를 초과하였습니다.".to_string(),
    ));
}
```

핵심 로직:
- **KST(UTC+9) 기준**으로 현재 월의 시작 시점을 계산한다.
- `retrospect_room_id` (팀 단위)로 같은 팀 내 분석 완료 회고 수를 카운트한다.
- `team_insight IS NOT NULL` 조건과 `updated_at >= 월초` 조건으로 필터링한다.
- **팀당 월 10회** 제한을 초과하면 `AI4031` (403) 에러를 반환한다.

---

## 6단계: 최소 데이터 기준 확인

**소스**: `service.rs:1657-1691`

### 6-1. 제출 완료 참여자 수 확인 (`service.rs:1659-1673`)

```rust
let submitted_members = member_retro::Entity::find()
    .filter(member_retro::Column::RetrospectId.eq(retrospect_id))
    .filter(
        member_retro::Column::Status
            .is_in([RetrospectStatus::Submitted, RetrospectStatus::Analyzed]),
    )
    .all(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

if submitted_members.is_empty() {
    return Err(AppError::RetroInsufficientData(
        "분석할 회고 답변 데이터가 부족합니다.".to_string(),
    ));
}
```

- `member_retro` 테이블에서 status가 `SUBMITTED` 또는 `ANALYZED`인 참여자를 조회한다.
- 제출 완료된 참여자가 0명이면 `RETRO4221` (422) 에러를 반환한다.
- **스펙 차이**: API 스펙에서는 이 에러를 `RETRO4042` (404)로 정의하고 있으나, 구현은 `RETRO4221` (422)를 사용한다.

### 6-2. 답변 수 확인 (`service.rs:1675-1691`)

```rust
let all_responses = response::Entity::find()
    .filter(response::Column::RetrospectId.eq(retrospect_id))
    .all(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

let answer_count = all_responses
    .iter()
    .filter(|r| !r.content.trim().is_empty())
    .count();

if answer_count < 3 {
    return Err(AppError::RetroInsufficientData(
        "분석할 회고 답변 데이터가 부족합니다.".to_string(),
    ));
}
```

- 해당 회고의 모든 응답에서 빈 문자열/공백만 있는 답변을 제외하고 카운트한다.
- 유효 답변이 3개 미만이면 `RETRO4221` (422) 에러를 반환한다.

---

## 7단계: 답변 데이터 수집 (AI 프롬프트 입력 준비)

**소스**: `service.rs:1693-1775`

```
member_retro (참여자 목록)
  └→ member (닉네임 매핑)
  └→ member_response (멤버별 response_id 매핑)
       └→ response (질문 + 답변 내용)
```

세부 과정:
1. `member_ids` 추출 후 `member` 테이블에서 닉네임 조회 (`service.rs:1694-1717`)
2. `member_response` 테이블로 멤버별 `response_id` 매핑 구성 (`service.rs:1722-1747`)
3. `response_map`으로 각 응답의 질문/답변 내용을 매핑 (`service.rs:1737-1738`)
4. 최종적으로 `Vec<MemberAnswerData>` 형태로 조합 (`service.rs:1749-1775`)

`MemberAnswerData` 구조 (`prompt.rs:5-9`):
```rust
pub struct MemberAnswerData {
    pub user_id: i64,
    pub user_name: String,
    pub answers: Vec<(String, String)>, // (질문, 답변)
}
```

---

## 8단계: AI 프롬프트 구성

**소스**: `prompt.rs:11-137`

### 시스템 프롬프트 (`prompt.rs:13-108`)
- AI의 역할을 "따뜻한 AI 분석가"로 설정한다.
- 말투 규칙(상냥체 ~어요 필수, 격식체 ~습니다 금지)을 명시한다.
- `teamInsight`, `emotionRank`(정확히 3개), `personalMissions`(사용자당 3개) 출력 형식을 JSON으로 지정한다.

### 사용자 프롬프트 (`prompt.rs:111-137`)
- 참여자별로 `userId`, `이름`, Q1~Qn(질문-답변 쌍)을 나열한다.
- 빈 답변은 `(답변 없음)`으로 대체한다 (`prompt.rs:125-129`).

---

## 9단계: OpenAI API 호출

**소스**: `ai/service.rs:36-76` (analyze_retrospective), `ai/service.rs:93-147` (call_openai)

```rust
let system_prompt = AnalysisPrompt::system_prompt();
let user_prompt = AnalysisPrompt::user_prompt(members_data);
let raw_response = self.call_openai(&system_prompt, &user_prompt).await?;
```

`call_openai` 메서드 상세 (`ai/service.rs:93-147`):
- **모델**: `gpt-4o-mini` (`ai/service.rs:112`)
- **온도**: `0.7` (`ai/service.rs:114`)
- **최대 토큰**: `4000` (`ai/service.rs:115`)
- **타임아웃**: `30초` (tokio::time::timeout, `ai/service.rs:121`)
- 에러 분기 처리 (`ai/service.rs:126-137`):
  - 401/Unauthorized -> `AiConnectionFailed` (AI5002, 500)
  - 429/rate limit -> `AiServiceUnavailable` (AI5031, 503)
  - 503/unavailable -> `AiServiceUnavailable` (AI5031, 503)
  - 기타 -> `AiGeneralError` (AI5003, 500)

---

## 10단계: 응답 파싱 및 검증

**소스**: `ai/service.rs:47-72`

### JSON 추출 (`ai/service.rs:79-90`)
```rust
fn extract_json(response: &str) -> &str {
    let trimmed = response.trim();
    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            return &trimmed[start..=end];
        }
    }
    trimmed
}
```
- AI가 마크다운 코드 블록(```json ... ```)으로 감싸서 응답할 수 있으므로, 첫 번째 `{`부터 마지막 `}`까지를 추출한다.

### JSON 파싱 (`ai/service.rs:49-53`)
```rust
let analysis: AnalysisResponse = serde_json::from_str(json_str).map_err(|e| {
    warn!("AI 응답 JSON 파싱 실패: {}", e);
    AppError::AiAnalysisFailed(format!("AI 응답을 파싱할 수 없습니다: {}", e))
})?;
```

### 구조 검증 (`ai/service.rs:55-72`)
- `emotionRank`가 정확히 **3개**인지 확인
- 각 사용자의 `missions`가 정확히 **3개**인지 확인
- 조건 미충족 시 `AiAnalysisFailed` (AI5001) 에러를 반환한다.

---

## 11단계: DB 트랜잭션 저장

**소스**: `service.rs:1795-1836`

### 트랜잭션 시작 (`service.rs:1796-1800`)
```rust
let txn = state
    .db
    .begin()
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```

### 11-1. retrospects.team_insight 업데이트 (`service.rs:1802-1809`)
```rust
let mut retrospect_active: retrospect::ActiveModel = retrospect_model.clone().into();
retrospect_active.team_insight = Set(Some(team_insight.clone()));
retrospect_active.updated_at = Set(Utc::now().naive_utc());
retrospect_active
    .update(&txn)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```

### 11-2. member_retro 업데이트 (각 참여자마다) (`service.rs:1812-1832`)
```rust
for mr in &submitted_members {
    // personal_missions에서 해당 member_id의 미션 찾기
    let personal_insight = personal_missions
        .iter()
        .find(|pm| pm.user_id == mr.member_id)
        .map(|pm| {
            pm.missions
                .iter()
                .map(|m| format!("{}: {}", m.mission_title, m.mission_desc))
                .collect::<Vec<_>>()
                .join("\n")
        });

    let mut mr_active: member_retro::ActiveModel = mr.clone().into();
    mr_active.personal_insight = Set(personal_insight);
    mr_active.status = Set(RetrospectStatus::Analyzed);
    mr_active
        .update(&txn)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
}
```

- 각 참여자의 `personal_insight`에 미션 정보를 "제목: 설명" 형식으로 저장한다.
- `status`를 `Analyzed`로 변경한다.

### 트랜잭션 커밋 (`service.rs:1834-1836`)
```rust
txn.commit()
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```

---

## 12단계: 응답 반환

**소스**: `service.rs:1838-1840`, `handler.rs:432-435`

```rust
// service.rs
Ok(analysis)

// handler.rs
Ok(Json(BaseResponse::success_with_message(
    result,
    "회고 분석이 성공적으로 완료되었습니다.",
)))
```

- `personalMissions`는 `userId` 오름차순으로 정렬되어 반환된다 (`service.rs:1790`).
- `BaseResponse::success_with_message`로 감싸서 표준 응답 형식으로 반환한다.

---

## 에러 흐름 요약

| 단계 | 조건 | 에러 타입 | 에러 코드 | HTTP |
|------|------|----------|----------|------|
| 2 | 회고 미존재 | `RetrospectNotFound` | RETRO4041 | 404 |
| 3 | 이미 분석 완료 | `RetroAlreadyAnalyzed` | RETRO4091 | 409 |
| 4 | 팀 멤버 아님 | `TeamAccessDenied` | TEAM4031 | 403 |
| 5 | 월간 한도 초과 | `AiMonthlyLimitExceeded` | AI4031 | 403 |
| 6 | 참여자 0명 또는 답변 < 3개 | `RetroInsufficientData` | RETRO4221 | 422 |
| 9 | API 타임아웃 | `AiServiceUnavailable` | AI5031 | 503 |
| 9 | API 키 무효 | `AiConnectionFailed` | AI5002 | 500 |
| 9 | Rate limit 초과 또는 503 | `AiServiceUnavailable` | AI5031 | 503 |
| 9 | 기타 API 오류 | `AiGeneralError` | AI5003 | 500 |
| 10 | JSON 파싱 실패 | `AiAnalysisFailed` | AI5001 | 500 |
| 10 | 감정/미션 개수 불일치 | `AiAnalysisFailed` | AI5001 | 500 |

> **스펙 문서와의 차이**: API 스펙(`docs/api-specs/022-retrospect-analysis.md`)에서는 데이터 부족 에러를 `RETRO4042` (404)로, AI 에러를 `AI5001` (500) 하나로 정의하고 있다. 또한 `RETRO4091` (이미 분석 완료, 409)과 `TEAM4031` (팀 접근 권한, 403) 에러가 스펙 에러 코드 요약 테이블에서 누락되어 있다. 실제 구현에서는 데이터 부족을 `RETRO4221` (422)로, AI 에러를 4가지 코드(`AI5001`/`AI5002`/`AI5031`/`AI5003`)로 세분화하여 처리한다.
