# 동작 흐름: 회고 상세 조회 (API-012)

## 전체 흐름 요약

```
Client
  |
  | GET /api/v1/retrospects/{retrospectId}
  | Authorization: Bearer {token}
  v
Handler (get_retrospect_detail)
  |-- Path Parameter 검증 (retrospectId >= 1)
  |-- JWT에서 user_id 추출
  |-- Service 호출
  v
Service (get_retrospect_detail)
  |-- [1] 회고 존재 여부 확인 (retrospect 테이블)
  |-- [2] 접근 권한 확인 (member_retro_room 테이블)
  |-- [3] 참여 멤버 조회 (member_retro + member 테이블)
  |-- [4] 전체 응답(response) 조회
  |-- [5] 질문 리스트 추출 (응답에서 중복 제거)
  |-- [6] 전체 좋아요 수 집계 (response_like 테이블)
  |-- [7] 전체 댓글 수 집계 (response_comment 테이블)
  |-- [8] 응답 DTO 구성
  v
Client <-- JSON 응답
```

## 단계별 상세 설명

### 1단계: Handler - 요청 수신 및 검증

**파일**: `codes/server/src/domain/retrospect/handler.rs:276-298`

```rust
pub async fn get_retrospect_detail(
    user: AuthUser,
    State(state): State<AppState>,
    Path(retrospect_id): Path<i64>,
) -> Result<Json<BaseResponse<RetrospectDetailResponse>>, AppError> {
    // retrospectId 검증 (1 이상의 양수)
    if retrospect_id < 1 {
        return Err(AppError::BadRequest(
            "retrospectId는 1 이상의 양수여야 합니다.".to_string(),
        ));
    }

    // 사용자 ID 추출
    let user_id = user.user_id()?;

    // 서비스 호출
    let result = RetrospectService::get_retrospect_detail(state, user_id, retrospect_id).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "회고 상세 정보 조회를 성공했습니다.",
    )))
}
```

핸들러의 책임은 3가지로 제한된다:
1. Path Parameter(`retrospectId`)의 유효성을 검증한다 (1 이상의 양수).
2. JWT 토큰에서 `user_id`를 추출한다 (`AuthUser` extractor 사용).
3. Service를 호출하고 결과를 `BaseResponse`로 감싸서 반환한다.

### 2단계: Service - 회고 존재 여부 확인

**파일**: `codes/server/src/domain/retrospect/service.rs:819-824`

```rust
let retrospect_model = retrospect::Entity::find_by_id(retrospect_id)
    .one(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?
    .ok_or_else(|| AppError::RetrospectNotFound("존재하지 않는 회고입니다.".to_string()))?;
```

`retrospect` 테이블에서 `retrospect_id`로 단건 조회한다.
- 레코드가 없으면 `AppError::RetrospectNotFound` (404) 를 반환한다.
- DB 오류 시 `AppError::InternalError` (500) 를 반환한다.

### 3단계: Service - 접근 권한 확인

**파일**: `codes/server/src/domain/retrospect/service.rs:827-839`

```rust
let retrospect_room_id = retrospect_model.retrospect_room_id;
let is_team_member = member_retro_room::Entity::find()
    .filter(member_retro_room::Column::MemberId.eq(user_id))
    .filter(member_retro_room::Column::RetrospectRoomId.eq(retrospect_room_id))
    .one(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

if is_team_member.is_none() {
    return Err(AppError::TeamAccessDenied(
        "해당 회고에 접근 권한이 없습니다.".to_string(),
    ));
}
```

회고가 속한 `retrospect_room_id`(팀 ID)를 통해 `member_retro_room` 테이블에서 해당 사용자가 팀 멤버인지 확인한다.
- 멤버가 아니면 `AppError::TeamAccessDenied` (403) 를 반환한다.

**참고**: 이 API에서는 `find_retrospect_for_member` 헬퍼를 사용하지 않고, 별도로 `member_retro_room` 테이블을 직접 조회하는 방식을 사용한다. `find_retrospect_for_member` 헬퍼(라인 323-352)는 `member_team` 테이블을 통해 확인하며, 보안상 404로 통합 응답하는 패턴이다. 반면 이 API는 403(접근 권한 없음)과 404(회고 없음)를 분리한다.

### 4단계: Service - 참여 멤버 조회

**파일**: `codes/server/src/domain/retrospect/service.rs:842-890`

```rust
// member_retro에서 해당 회고의 참여자 목록을 등록일 기준 오름차순으로 조회
let member_retros = member_retro::Entity::find()
    .filter(member_retro::Column::RetrospectId.eq(retrospect_id))
    .order_by_asc(member_retro::Column::MemberRetroId)
    .all(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

// 멤버 ID 추출
let member_ids: Vec<i64> = member_retros.iter().map(|mr| mr.member_id).collect();

// member 테이블에서 닉네임 조회 (IN 쿼리)
let members = if member_ids.is_empty() {
    vec![]
} else {
    member::Entity::find()
        .filter(member::Column::MemberId.is_in(member_ids))
        .all(&state.db)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?
};

// HashMap으로 member_id -> nickname 매핑
let member_map: HashMap<i64, String> = members
    .iter()
    .map(|m| {
        let nickname = m.nickname.clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Unknown".to_string());
        (m.member_id, nickname)
    })
    .collect();

// member_retro 순서를 유지하면서 RetrospectMemberItem으로 변환
let member_items: Vec<RetrospectMemberItem> = member_retros
    .iter()
    .filter_map(|mr| {
        let name = member_map.get(&mr.member_id);
        name.map(|n| RetrospectMemberItem {
            member_id: mr.member_id,
            user_name: n.clone(),
        })
    })
    .collect();
```

이 단계는 두 테이블을 별도로 조회하고 메모리에서 조인하는 패턴이다:
1. `member_retro` 테이블에서 해당 회고의 참여자 레코드를 `member_retro_id` 기준 오름차순으로 가져온다.
2. 참여자의 `member_id` 목록을 추출하여 `member` 테이블에서 IN 쿼리로 한번에 조회한다.
3. `HashMap<member_id, nickname>`을 구성하여 빠른 룩업을 가능하게 한다.
4. 원래 `member_retro` 순서를 유지하면서 DTO로 변환한다 (등록일 기준 오름차순 정렬 보장).
5. `member` 테이블에 없는 멤버는 `filter_map`으로 건너뛰고 `warn!` 로그를 남긴다.

### 5단계: Service - 질문 리스트 추출

**파일**: `codes/server/src/domain/retrospect/service.rs:892-913`

```rust
let responses = response::Entity::find()
    .filter(response::Column::RetrospectId.eq(retrospect_id))
    .order_by_asc(response::Column::ResponseId)
    .all(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

let response_ids: Vec<i64> = responses.iter().map(|r| r.response_id).collect();

let mut seen_questions = HashSet::new();
let questions: Vec<RetrospectQuestionItem> = responses
    .iter()
    .filter(|r| seen_questions.insert(r.question.clone()))
    .take(5)
    .enumerate()
    .map(|(i, r)| RetrospectQuestionItem {
        index: (i + 1) as i32,
        content: r.question.clone(),
    })
    .collect();
```

질문 리스트는 별도 질문 테이블이 아니라, `response` 테이블의 `question` 필드에서 추출한다:
1. 해당 회고의 모든 응답을 `response_id` 기준 오름차순으로 조회한다.
2. `HashSet`을 이용하여 중복 질문을 제거한다 (`insert`가 `true`를 반환하면 새로운 질문).
3. 최대 5개까지만 취한다 (`.take(5)`).
4. `enumerate()`로 순번(1-based index)을 부여한다.

### 6단계: Service - 전체 좋아요 수 집계

**파일**: `codes/server/src/domain/retrospect/service.rs:916-924`

```rust
let total_like_count = if response_ids.is_empty() {
    0
} else {
    response_like::Entity::find()
        .filter(response_like::Column::ResponseId.is_in(response_ids.clone()))
        .count(&state.db)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))? as i64
};
```

해당 회고에 달린 모든 응답의 좋아요를 `response_like` 테이블에서 `count()` 집계한다.
- `response_ids`가 비어있으면 DB 쿼리 없이 0을 반환한다 (불필요한 쿼리 방지).
- `PaginatorTrait::count()`는 `u64`를 반환하므로 `as i64`로 캐스팅한다.

### 7단계: Service - 전체 댓글 수 집계

**파일**: `codes/server/src/domain/retrospect/service.rs:927-935`

```rust
let total_comment_count = if response_ids.is_empty() {
    0
} else {
    response_comment::Entity::find()
        .filter(response_comment::Column::ResponseId.is_in(response_ids))
        .count(&state.db)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))? as i64
};
```

좋아요 수 집계와 동일한 패턴으로 `response_comment` 테이블에서 댓글 수를 집계한다.

### 8단계: Service - 응답 DTO 구성 (chrono 날짜 포맷팅 포함)

**파일**: `codes/server/src/domain/retrospect/service.rs:937-950`

```rust
// start_time은 생성 시 KST로 저장되므로 변환 불필요
let start_time = retrospect_model.start_time.format("%Y-%m-%d").to_string();

Ok(RetrospectDetailResponse {
    team_id: retrospect_room_id,
    title: retrospect_model.title,
    start_time,
    retro_category: retrospect_model.retrospect_method,
    members: member_items,
    total_like_count,
    total_comment_count,
    questions,
})
```

최종적으로 수집한 데이터를 `RetrospectDetailResponse` DTO로 조합하여 반환한다.
- `start_time`은 `chrono::NaiveDateTime`을 `%Y-%m-%d` 포맷 문자열로 변환한다 (아래 참고).
- `team_id`에는 `retrospect_room_id`를 사용한다.

#### chrono 날짜/시간 처리 상세

엔티티의 `start_time` 필드 타입은 SeaORM의 `DateTime`이며, 이는 `chrono::NaiveDateTime`의 타입 별칭이다.

```rust
// 엔티티 정의 (entity/retrospect.rs:83)
pub start_time: DateTime,  // = chrono::NaiveDateTime
```

`NaiveDateTime`은 타임존 정보가 없는 날짜+시간 값이다. 이 프로젝트에서는 **회고 생성 시 입력받은 KST 날짜/시간을 그대로 `NaiveDateTime`으로 저장**하므로, 조회 시 별도 타임존 변환 없이 `.format()` 메서드로 문자열 변환만 수행한다.

```rust
// 회고 생성 시 (service.rs:120) - KST 날짜/시간을 NaiveDateTime으로 조합하여 저장
let start_time = NaiveDateTime::new(retrospect_date, retrospect_time);

// 상세 조회 시 (service.rs:938) - 저장된 값을 포맷만 하면 됨
let start_time = retrospect_model.start_time.format("%Y-%m-%d").to_string();
```

`chrono::NaiveDateTime::format()`은 `strftime` 스타일의 포맷 문자열을 받는다:
- `%Y` - 4자리 연도 (예: 2026)
- `%m` - 2자리 월 (예: 01)
- `%d` - 2자리 일 (예: 24)
- `%H` - 24시간제 시 (예: 14)
- `%M` - 분 (예: 30)

이 API에서는 날짜 부분만 필요하므로 `"%Y-%m-%d"` 포맷을 사용한다. 같은 `start_time` 필드를 다른 API에서는 시간 포맷으로도 사용한다:
- API-010 (팀 회고 목록): `format("%Y-%m-%d")` + `format("%H:%M")` 로 날짜와 시간을 분리 추출 (dto.rs:204-205)

## 관련 DB 테이블 및 접근 순서

```
[1] retrospect               -- 회고 존재 여부 (find_by_id)
[2] member_retro_room         -- 접근 권한 확인 (filter by member_id, retrospect_room_id)
[3] member_retro              -- 참여 멤버 목록 (filter by retrospect_id)
[3] member                    -- 멤버 닉네임 (filter by member_id IN)
[4] response                  -- 응답 목록 (filter by retrospect_id)
[5] response_like             -- 좋아요 수 (count, filter by response_id IN)
[6] response_comment          -- 댓글 수 (count, filter by response_id IN)
```

총 6개 테이블, 최대 7회의 DB 쿼리가 순차적으로 실행된다.
