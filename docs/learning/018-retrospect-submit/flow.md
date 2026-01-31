# 동작 흐름: 회고 최종 제출 (API-017)

## 전체 흐름도

```
클라이언트 요청
    |
    v
[1] 핸들러: retrospectId 경로 파라미터 검증
    |
    v
[2] 핸들러: JWT에서 user_id 추출
    |
    v
[3] 서비스: 답변 비즈니스 검증 (트랜잭션 전)
    |   - 정확히 5개 답변 확인
    |   - questionNumber 1~5 모두 존재 확인
    |   - 공백만으로 구성된 답변 체크
    |   - 1,000자 초과 체크
    |
    v
[4] 서비스: 회고 존재 여부 확인
    |
    v
[5] 서비스: 트랜잭션 시작
    |
    v
[6] 서비스: 참석자(member_retro) 확인 + 행 잠금
    |
    v
[7] 서비스: 이미 제출 완료 여부 확인
    |
    v
[8] 서비스: 해당 멤버의 response 목록 조회
    |
    v
[9] 서비스: 5개 답변 내용 업데이트
    |
    v
[10] 서비스: member_retro 상태를 SUBMITTED로 변경
    |
    v
[11] 서비스: 트랜잭션 커밋
    |
    v
[12] 응답 반환 (KST 변환)
```

## 단계별 상세 설명

### 단계 1~2: 핸들러 레이어

**소스**: `handler.rs:324~347`

```rust
pub async fn submit_retrospect(
    user: AuthUser,
    State(state): State<AppState>,
    Path(retrospect_id): Path<i64>,
    Json(req): Json<SubmitRetrospectRequest>,
) -> Result<Json<BaseResponse<SubmitRetrospectResponse>>, AppError> {
    if retrospect_id < 1 {
        return Err(AppError::BadRequest(
            "retrospectId는 1 이상의 양수여야 합니다.".to_string(),
        ));
    }
    let user_id = user.user_id()?;
    let result = RetrospectService::submit_retrospect(state, user_id, retrospect_id, req).await?;
    // ...
}
```

- `retrospectId` 경로 파라미터가 1 미만이면 즉시 400 에러 반환
- `AuthUser` 추출자에서 JWT 토큰을 파싱하여 `user_id`를 얻음
- 비즈니스 로직은 서비스 레이어에 완전히 위임

### 단계 3: 답변 비즈니스 검증 (트랜잭션 전)

**소스**: `service.rs:1543~1578`

```rust
fn validate_answers(answers: &[SubmitAnswerItem]) -> Result<(), AppError> {
    // 1. 정확히 5개 답변 확인
    if answers.len() != 5 {
        return Err(AppError::RetroAnswersMissing(...));
    }

    // 2. questionNumber 1~5 모두 존재하는지 확인
    let question_numbers: HashSet<i32> = answers.iter().map(|a| a.question_number).collect();
    let expected: HashSet<i32> = (1..=5).collect();
    if question_numbers != expected {
        return Err(AppError::RetroAnswersMissing(...));
    }

    // 3. 각 답변 내용 검증
    for answer in answers {
        if answer.content.trim().is_empty() {
            return Err(AppError::RetroAnswerWhitespaceOnly(...));
        }
        if answer.content.chars().count() > 1000 {
            return Err(AppError::RetroAnswerTooLong(...));
        }
    }
    Ok(())
}
```

검증 순서:
1. 배열 길이 검사 (`len() != 5`) -> `RETRO4002`
2. `HashSet` 비교로 1~5번 질문 모두 존재하는지 확인 -> `RETRO4002`
3. `trim().is_empty()`로 공백 전용 답변 차단 -> `RETRO4007`
4. `chars().count() > 1000`으로 글자 수 제한 -> `RETRO4003`

이 검증을 트랜잭션 전에 수행하는 이유: 불필요한 DB 연결/잠금을 방지하기 위해서입니다.

### 단계 4: 회고 존재 여부 확인

**소스**: `service.rs:584~589`

```rust
let _retrospect_model = retrospect::Entity::find_by_id(retrospect_id)
    .one(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?
    .ok_or_else(|| AppError::RetrospectNotFound("존재하지 않는 회고입니다.".to_string()))?;
```

- `find_by_id`로 회고 존재 여부만 먼저 확인
- 트랜잭션 외부에서 수행하여 불필요한 트랜잭션 시작을 방지

### 단계 5~6: 트랜잭션 시작 + 참석자 확인 (행 잠금)

**소스**: `service.rs:591~610`

```rust
let txn = state.db.begin().await...;

let member_retro_model = member_retro::Entity::find()
    .filter(member_retro::Column::MemberId.eq(user_id))
    .filter(member_retro::Column::RetrospectId.eq(retrospect_id))
    .lock_exclusive()   // SELECT ... FOR UPDATE
    .one(&txn)
    .await...;
```

- `lock_exclusive()`는 SQL의 `SELECT ... FOR UPDATE`에 해당
- 동일 사용자가 동시에 제출 요청을 보내는 경우 경쟁 조건(race condition) 방지
- 해당 `member_retro` 행이 없으면 참여자가 아니므로 404 반환

### 단계 7: 이미 제출 완료 여부 확인

**소스**: `service.rs:612~619`

```rust
if member_retro_model.status == RetrospectStatus::Submitted
    || member_retro_model.status == RetrospectStatus::Analyzed
{
    return Err(AppError::RetroAlreadySubmitted(
        "이미 제출이 완료된 회고입니다.".to_string(),
    ));
}
```

- 행 잠금 후에 상태를 확인하므로, 동시 요청이 와도 한쪽만 통과
- `SUBMITTED` 또는 `ANALYZED` 상태이면 중복 제출 차단 (`RETRO4033`)

### 단계 8: 해당 멤버의 response 목록 조회

**소스**: `service.rs:621~644`

```rust
let member_response_ids: Vec<i64> = member_response::Entity::find()
    .filter(member_response::Column::MemberId.eq(user_id))
    .all(&txn).await...
    .iter().map(|mr| mr.response_id).collect();

let responses = response::Entity::find()
    .filter(response::Column::RetrospectId.eq(retrospect_id))
    .filter(response::Column::ResponseId.is_in(member_response_ids))
    .order_by_asc(response::Column::ResponseId)
    .all(&txn).await...;

if responses.len() != 5 {
    return Err(AppError::InternalError(...));
}
```

- `member_response` 중간 테이블을 통해 해당 멤버의 응답 ID를 조회
- 응답을 `response_id` 오름차순으로 정렬하여 질문 순서와 매칭
- 응답 수가 5개가 아니면 데이터 정합성 오류로 500 에러

### 단계 9: 답변 내용 업데이트

**소스**: `service.rs:646~659`

```rust
let now = Utc::now().naive_utc();
for answer in &req.answers {
    let idx = (answer.question_number - 1) as usize;
    let response_model = &responses[idx];

    let mut active: response::ActiveModel = response_model.clone().into();
    active.content = Set(answer.content.clone());
    active.updated_at = Set(now);
    active.update(&txn).await...;
}
```

- `questionNumber`(1~5)를 인덱스(0~4)로 변환하여 정렬된 response와 매칭
- `ActiveModel`로 변환 후 `content`와 `updated_at`만 업데이트
- 모든 업데이트가 동일 트랜잭션 내에서 수행됨

### 단계 10: 상태 변경

**소스**: `service.rs:661~668`

```rust
let mut member_retro_active: member_retro::ActiveModel = member_retro_model.clone().into();
member_retro_active.status = Set(RetrospectStatus::Submitted);
member_retro_active.submitted_at = Set(Some(now));
member_retro_active.update(&txn).await...;
```

- `member_retro` 테이블의 `status`를 `SUBMITTED`로 변경
- `submitted_at`에 현재 UTC 시간 기록

### 단계 11~12: 커밋 및 응답 반환

**소스**: `service.rs:670~685`

```rust
txn.commit().await...;

let kst_display = (now + chrono::Duration::hours(9))
    .format("%Y-%m-%d")
    .to_string();

Ok(SubmitRetrospectResponse {
    retrospect_id,
    submitted_at: kst_display,
    status: RetrospectStatus::Submitted,
})
```

- 트랜잭션 커밋으로 모든 변경 사항 확정
- DB에는 UTC로 저장하되 응답에서는 KST(+9시간)로 변환하여 반환
- `YYYY-MM-DD` 형식의 날짜 문자열 생성

## 에러 발생 시점 요약

| 단계 | 에러 | 코드 |
|------|------|------|
| 1 | retrospectId < 1 | COMMON400 |
| 3 | 답변 5개 미만 / 번호 불일치 | RETRO4002 |
| 3 | 공백만 입력 | RETRO4007 |
| 3 | 1,000자 초과 | RETRO4003 |
| 4 | 회고 미존재 | RETRO4041 |
| 6 | 참석자 아님 | RETRO4041 |
| 7 | 이미 제출 완료 | RETRO4033 |
| 8 | response 수 불일치 | COMMON500 |
