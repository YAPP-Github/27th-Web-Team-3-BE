# 동작 흐름: 회고 참여자 등록 (API-014)

## 전체 흐름 요약

```
클라이언트 요청
  -> [핸들러] Path 파라미터 검증 + 사용자 ID 추출
    -> [서비스] 회고 조회
      -> [서비스] 팀 멤버십 확인
        -> [서비스] 과거 회고 여부 확인
          -> [서비스] 중복 참석 확인
            -> [서비스] member 정보 조회
              -> [서비스] member_retro 레코드 생성
                -> 응답 반환
```

## 단계별 상세 흐름

### 1단계: 핸들러 - Path 파라미터 검증 및 사용자 인증

**소스**: `handler.rs:135~157`

```rust
pub async fn create_participant(
    user: AuthUser,
    State(state): State<AppState>,
    Path(retrospect_id): Path<i64>,
) -> Result<Json<BaseResponse<CreateParticipantResponse>>, AppError> {
    if retrospect_id < 1 {
        return Err(AppError::BadRequest(
            "retrospectId는 1 이상의 양수여야 합니다.".to_string(),
        ));
    }
    let user_id = user.user_id()?;
    let result = RetrospectService::create_participant(state, user_id, retrospect_id).await?;
    // ...
}
```

- `AuthUser` 미들웨어가 JWT에서 유저 정보를 자동 추출합니다 (`handler.rs:136`).
- `retrospect_id`가 1 미만이면 즉시 `AppError::BadRequest` 반환 (`handler.rs:141~145`).
- `user.user_id()?`로 JWT에서 사용자 ID를 파싱합니다 (`handler.rs:148`).
- Request Body가 없으며, Path 파라미터와 JWT만 사용합니다.

### 2단계: 서비스 - 회고 조회 및 팀 멤버십 확인

**소스**: `service.rs:323~352` (`find_retrospect_for_member` 함수)

```rust
async fn find_retrospect_for_member(
    state: &AppState,
    user_id: i64,
    retrospect_id: i64,
) -> Result<retrospect::Model, AppError> {
    let retrospect_model = retrospect::Entity::find_by_id(retrospect_id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?
        .ok_or_else(|| {
            AppError::RetrospectNotFound("존재하지 않는 회고이거나 접근 권한이 없습니다.".to_string())
        })?;

    let is_member = member_team::Entity::find()
        .filter(member_team::Column::MemberId.eq(user_id))
        .filter(member_team::Column::TeamId.eq(retrospect_model.team_id))
        .one(&state.db)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    if is_member.is_none() {
        return Err(AppError::RetrospectNotFound(
            "존재하지 않는 회고이거나 접근 권한이 없습니다.".to_string(),
        ));
    }

    Ok(retrospect_model)
}
```

- **회고 조회**: `retrospect::Entity::find_by_id(retrospect_id)`로 DB에서 회고를 조회합니다 (`service.rs:328~336`).
  - 회고가 없으면 `AppError::RetrospectNotFound` (HTTP 404, 코드 `RETRO4041`) 반환.
- **멤버십 확인**: `member_team` 테이블에서 `user_id`와 `team_id` 조합을 검색합니다 (`service.rs:338~343`).
  - 멤버가 아니면 `AppError::RetrospectNotFound` 반환 (보안상 404로 통합 처리).
- 성공 시 `retrospect::Model`을 반환합니다 (`service.rs:351`).

### 3단계: 서비스 - 과거/진행중 회고 여부 확인

**소스**: `service.rs:364~370`

```rust
let now_kst = Utc::now().naive_utc() + chrono::Duration::hours(9);
if retrospect_model.start_time <= now_kst {
    return Err(AppError::RetrospectAlreadyStarted(
        "이미 시작되었거나 종료된 회고에는 참석할 수 없습니다.".to_string(),
    ));
}
```

- `Utc::now().naive_utc()`로 현재 UTC 시간을 가져온 뒤 9시간을 더해 KST로 변환합니다 (`service.rs:365`).
- `retrospect_model.start_time`과 비교하여 이미 시작된 회고인지 판단합니다 (`service.rs:366`).
- 시작 시간이 현재 시각 이전이거나 같으면 `AppError::RetrospectAlreadyStarted` (HTTP 400, 코드 `RETRO4002`) 반환.

### 4단계: 서비스 - 중복 참석 확인 (애플리케이션 레벨)

**소스**: `service.rs:372~384`

```rust
let existing_participant = member_retro::Entity::find()
    .filter(member_retro::Column::MemberId.eq(user_id))
    .filter(member_retro::Column::RetrospectId.eq(retrospect_id))
    .one(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

if existing_participant.is_some() {
    return Err(AppError::ParticipantDuplicate(
        "이미 참석자로 등록되어 있습니다.".to_string(),
    ));
}
```

- `member_retro` 테이블에서 `(member_id, retrospect_id)` 조합으로 기존 레코드를 검색합니다 (`service.rs:373~378`).
- 이미 존재하면 `AppError::ParticipantDuplicate` (HTTP 409, 코드 `RETRO4091`) 반환 (`service.rs:380~384`).

### 5단계: 서비스 - member 정보 조회 (닉네임 추출)

**소스**: `service.rs:386~398`

```rust
let member_model = member::Entity::find_by_id(user_id)
    .one(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?
    .ok_or_else(|| AppError::InternalError("회원 정보를 찾을 수 없습니다.".to_string()))?;

let nickname = member_model
    .email
    .split('@')
    .next()
    .unwrap_or(&member_model.email)
    .to_string();
```

- `member` 테이블에서 사용자 정보를 조회합니다 (`service.rs:387~391`).
- 이메일에서 `@` 앞부분을 추출하여 닉네임으로 사용합니다 (`service.rs:393~398`).

### 6단계: 서비스 - member_retro 레코드 생성

**소스**: `service.rs:400~419`

```rust
let member_retro_model = member_retro::ActiveModel {
    member_id: Set(user_id),
    retrospect_id: Set(retrospect_id),
    personal_insight: Set(None),
    ..Default::default()
};

let inserted = member_retro_model.insert(&state.db).await.map_err(|e| {
    let error_msg = e.to_string().to_lowercase();
    if error_msg.contains("duplicate")
        || error_msg.contains("unique")
        || error_msg.contains("constraint")
    {
        AppError::ParticipantDuplicate("이미 참석자로 등록되어 있습니다.".to_string())
    } else {
        AppError::InternalError(e.to_string())
    }
})?;
```

- `member_retro::ActiveModel`을 생성하여 `member_id`, `retrospect_id`, `personal_insight(None)`을 설정합니다 (`service.rs:401~406`).
- `insert` 시 DB 유니크 제약 위반이 발생하면 에러 메시지를 파싱하여 `ParticipantDuplicate`로 매핑합니다 (`service.rs:408~419`).
  - 이중 방어: 4단계 애플리케이션 레벨 체크 + DB 레벨 유니크 제약

### 7단계: 서비스 - 응답 반환

**소스**: `service.rs:421~427`

```rust
Ok(CreateParticipantResponse {
    participant_id: inserted.member_retro_id,
    member_id: user_id,
    nickname,
})
```

- 생성된 `member_retro_id`를 `participant_id`로, 유저 ID와 닉네임을 함께 반환합니다.

### 8단계: 핸들러 - 최종 응답 구성

**소스**: `handler.rs:153~156`

```rust
Ok(Json(BaseResponse::success_with_message(
    result,
    "회고 참석자로 성공적으로 등록되었습니다.",
)))
```

- `BaseResponse` 래퍼로 감싸서 표준 응답 형식으로 반환합니다.
- HTTP 200과 함께 `isSuccess: true`, `code: "COMMON200"` 응답을 전달합니다.

## 에러 발생 시점 요약

| 단계 | 에러 조건 | AppError 타입 | HTTP | 코드 |
|------|----------|---------------|------|------|
| 1단계 | retrospectId < 1 | BadRequest | 400 | COMMON400 |
| 2단계 | 회고 없음 | RetrospectNotFound | 404 | RETRO4041 |
| 2단계 | 팀 멤버 아님 | RetrospectNotFound | 404 | RETRO4041 |
| 3단계 | 과거/진행중 회고 | RetrospectAlreadyStarted | 400 | RETRO4002 |
| 4단계 | 중복 참석 | ParticipantDuplicate | 409 | RETRO4091 |
| 5단계 | 회원 정보 없음 | InternalError | 500 | COMMON500 |
| 6단계 | DB 유니크 위반 | ParticipantDuplicate | 409 | RETRO4091 |
| 6단계 | DB 기타 오류 | InternalError | 500 | COMMON500 |
