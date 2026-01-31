# 동작 흐름: 회고 답변 임시 저장 (API-016)

## 전체 흐름 요약

```
클라이언트 요청
  -> [Axum] AuthUser extractor (JWT 인증)
  -> [Handler] save_draft
    -> retrospect_id >= 1 검증
    -> user.user_id()로 사용자 ID 추출
  -> [Service] save_draft (비즈니스 로직)
    -> validate_drafts() (개수, 중복, 범위, 길이 검증)
    -> DB: 회고 존재 확인 (retrospect 테이블)
    -> DB: 작성 권한 확인 (member_retro 테이블)
    -> DB: 사용자의 response_id 목록 조회 (member_response 테이블)
    -> DB: response 목록 조회 (response 테이블, response_id 오름차순)
    -> 데이터 무결성 검증 (질문 수 5개 일치 여부)
    -> 트랜잭션 시작
      -> 각 답변(DraftItem)별로 루프
      -> Response.content 업데이트 (덮어쓰기)
      -> Response.updated_at 업데이트 (동일 타임스탬프)
    -> 트랜잭션 커밋
    -> KST 변환 후 DraftSaveResponse 생성
  -> [Handler] BaseResponse 래핑 후 JSON 반환
```

## 단계별 상세

### 1단계: JWT 인증 및 입력 검증

**파일**: `handler.rs:229-252`, `service.rs:1494-1539`

- **JWT 인증**: `AuthUser` extractor를 통해 요청 헤더의 Bearer 토큰을 검증합니다. `user.user_id()` 헬퍼 메서드로 JWT의 `sub` 클레임을 `i64`로 파싱합니다.
- **Path Parameter**: `retrospect_id`가 1 이상의 양수인지 핸들러에서 검증합니다.
- **비즈니스 검증**: 서비스 진입 후 `validate_drafts()` 함수에서 `DraftSaveRequest`의 `drafts` 배열에 대해 다음 검증을 수행합니다:
    1. 최소 1개, 최대 5개
    2. 질문 번호 중복 여부 (`HashSet` 사용)
    3. 질문 번호(`questionNumber`) 1~5 범위
    4. 답변 내용(`content`) 1,000자 제한 (`chars().count()` 사용, 바이트가 아닌 유니코드 문자 기준)

### 2단계: 서비스 진입 및 권한 확인

**파일**: `src/domain/retrospect/service.rs:460-493`

```rust
// 회고 존재 확인
let _retrospect_model = retrospect::Entity::find_by_id(retrospect_id)
    .one(&state.db).await?
    .ok_or_else(|| AppError::RetrospectNotFound(...))?;

// 작성 권한 확인 (참석자 여부)
let _member_retro_model = member_retro::Entity::find()
    .filter(member_retro::Column::MemberId.eq(user_id))
    .filter(member_retro::Column::RetrospectId.eq(retrospect_id))
    .one(&state.db).await?
    .ok_or_else(|| AppError::TeamAccessDenied(...))?;
```

- 회고가 없으면 `RETRO4041` (404 Not Found)
- 참석자가 아니면 `TEAM4031` (403 Forbidden)

> **참고**: API-016은 `find_retrospect_for_member()` 헬퍼를 사용하지 않고 독립적으로 회고 존재와 참석자 여부를 분리 검증합니다. API-021의 `find_retrospect_for_member()`는 보안상 404로 통합하지만, API-016은 "존재하지 않음"과 "권한 없음"을 구분하여 각각 404와 403을 반환합니다.

### 3단계: 업데이트 대상 응답(Response) 식별

**파일**: `src/domain/retrospect/service.rs:495-528`

회고 생성 시(`API-011`) 이미 5개의 빈 응답(`Response`)이 생성되어 있습니다.
사용자가 수정하려는 응답이 본인의 것인지 확인하기 위해 `member_response` 매핑 테이블을 거쳐 조회합니다.

1.  `member_response` 테이블에서 `user_id`에 해당하는 `response_id` 목록 조회
2.  응답 ID 목록이 비어 있으면 `TEAM4031` (403) 반환
3.  `response` 테이블에서 `retrospect_id`와 `response_id`가 모두 일치하는 레코드 조회 (response_id 오름차순 정렬)
4.  조회된 응답 개수가 5개인지 검증 (무결성 체크)

### 4단계: 트랜잭션 내 답변 업데이트

```rust
let now = Utc::now().naive_utc();
let txn = state.db.begin().await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

for draft in &req.drafts {
    let idx = (draft.question_number - 1) as usize;
    let response_model = &responses[idx];

    let mut active: response::ActiveModel = response_model.clone().into();
    // content가 None이면 빈 문자열로 저장 (기존 내용 삭제)
    active.content = Set(draft.content.clone().unwrap_or_default());
    active.updated_at = Set(now);
    active.update(&txn).await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
}

txn.commit().await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```

> **주의**: `updated_at`는 루프 바깥에서 한 번만 `Utc::now().naive_utc()`를 호출하여 모든 답변이 동일한 타임스탬프를 가집니다. `NaiveDateTime` 타입이므로 타임존 정보 없이 UTC 시간을 저장합니다.

- `questionNumber` (1~5)를 인덱스 (0~4)로 변환하여 해당 `response` 모델을 찾습니다.
- `ActiveModel`로 변환하여 `content`와 `updated_at` 필드만 수정(`Set`)합니다.
- 루프를 돌며 각 답변을 `update` 하고, 모든 작업이 성공하면 `commit` 합니다.

## 에러 흐름

| 단계 | 조건 | 에러 타입 | 에러 코드 | HTTP 상태 |
|------|------|-----------|-----------|-----------|
| 1 (핸들러) | retrospectId < 1 | `AppError::BadRequest` | COMMON400 | 400 |
| 1 (서비스) | 빈 배열, 개수 초과, 질문 번호 범위/중복 | `AppError::BadRequest` | COMMON400 | 400 |
| 1 (서비스) | 답변 1,000자 초과 | `AppError::RetroAnswerTooLong` | RETRO4003 | 400 |
| 2 (서비스) | 회고가 DB에 없음 | `AppError::RetrospectNotFound` | RETRO4041 | 404 |
| 3 (서비스) | 사용자가 참석자가 아님 | `AppError::TeamAccessDenied` | TEAM4031 | 403 |
| 4 (서비스) | member_response가 비어있음 | `AppError::TeamAccessDenied` | TEAM4031 | 403 |
| 5 (서비스) | DB 무결성 오류 (질문 수 불일치) | `AppError::InternalError` | COMMON500 | 500 |
| 6 (서비스) | DB 업데이트/트랜잭션 실패 | `AppError::InternalError` | COMMON500 | 500 |