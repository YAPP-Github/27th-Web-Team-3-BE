# 동작 흐름

API-020 회고 답변 카테고리별 조회의 요청 수신부터 응답 반환까지의 전체 흐름을 설명합니다.

---

## 1단계: 핸들러 진입 및 파라미터 추출

클라이언트 요청이 라우터를 통해 `list_responses` 핸들러에 도달합니다.
Axum의 `Path`, `Query` 추출자가 경로 파라미터와 쿼리 파라미터를 자동으로 역직렬화합니다.

```rust
pub async fn list_responses(
    user: AuthUser,
    State(state): State<AppState>,
    Path(retrospect_id): Path<i64>,
    Query(params): Query<ResponsesQueryParams>,
) -> Result<Json<BaseResponse<ResponsesListResponse>>, AppError> {
```

> 소스: `handler.rs:569-574`

---

## 2단계: retrospectId 검증

경로 파라미터 `retrospect_id`가 1 이상의 양수인지 확인합니다.
유효하지 않으면 `AppError::BadRequest`를 반환합니다.

```rust
if retrospect_id < 1 {
    return Err(AppError::BadRequest(
        "retrospectId는 1 이상의 양수여야 합니다.".to_string(),
    ));
}
```

> 소스: `handler.rs:576-580`

---

## 3단계: category 파싱 및 검증

쿼리 파라미터로 전달된 `category` 문자열을 `ResponseCategory` enum으로 파싱합니다.
`FromStr` trait의 `parse()` 메서드를 사용하며, 유효하지 않은 값이면 `RETRO4004` 에러를 반환합니다.

```rust
let category: ResponseCategory = params.category.parse().map_err(|_| {
    AppError::RetroCategoryInvalid("유효하지 않은 카테고리 값입니다.".to_string())
})?;
```

> 소스: `handler.rs:583-585`

유효한 category 값: `ALL`, `QUESTION_1`, `QUESTION_2`, `QUESTION_3`, `QUESTION_4`, `QUESTION_5`

---

## 4단계: cursor / size 검증

- **cursor**: 선택적 파라미터로, 값이 있으면 1 이상이어야 합니다.
- **size**: 선택적 파라미터로, 값이 없으면 기본값 10을 사용합니다. 1~100 범위만 허용합니다.

```rust
// cursor 검증 (있을 경우 1 이상)
if let Some(cursor) = params.cursor {
    if cursor < 1 {
        return Err(AppError::BadRequest(
            "cursor는 1 이상의 양수여야 합니다.".to_string(),
        ));
    }
}

// size 검증 (있을 경우 1~100)
let size = params.size.unwrap_or(10);
if !(1..=100).contains(&size) {
    return Err(AppError::BadRequest(
        "size는 1~100 범위의 정수여야 합니다.".to_string(),
    ));
}
```

> 소스: `handler.rs:588-602`

핵심 포인트:
- `unwrap_or(10)`: `Option<i64>`에서 값이 없으면 기본값 10 사용
- `(1..=100).contains(&size)`: 범위 포함 확인에 Range의 `contains` 메서드 활용

---

## 5단계: 서비스 호출

핸들러는 검증 완료된 파라미터를 서비스 레이어로 전달합니다.

```rust
let result = RetrospectService::list_responses(
    state,
    user_id,
    retrospect_id,
    category,
    params.cursor,
    size,
)
.await?;
```

> 소스: `handler.rs:608-616`

---

## 6단계: 회고 조회 및 팀 멤버십 확인 (서비스)

해당 회고가 존재하는지, 요청한 사용자가 해당 팀의 멤버인지 확인합니다.
권한이 없으면 `TEAM4031` 에러를 반환합니다.

```rust
let _retrospect_model =
    Self::find_retrospect_for_member(&state, user_id, retrospect_id).await?;
```

> 소스: `service.rs:1862-1863`

---

## 7단계: 전체 응답 데이터 조회

해당 회고의 모든 response 레코드를 `response_id` 오름차순으로 조회합니다.
결과가 비어 있으면 즉시 빈 응답을 반환합니다.

```rust
let all_responses = response::Entity::find()
    .filter(response::Column::RetrospectId.eq(retrospect_id))
    .order_by_asc(response::Column::ResponseId)
    .all(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```

> 소스: `service.rs:1866-1871`

---

## 8단계: 질문 텍스트 순서 결정

첫 번째 참여자(member_id가 가장 작은)의 응답 세트를 기준으로 질문 순서를 확정합니다.
`member_response` 테이블을 조회하여 member_id별로 그룹화한 뒤, 가장 작은 member_id의 응답 ID 목록을 사용합니다.

```rust
let first_member_id = member_response_map.keys().min().copied();
let question_response_ids: Vec<i64> = first_member_id
    .and_then(|mid| member_response_map.get(&mid))
    .cloned()
    .unwrap_or_default();
```

> 소스: `service.rs:1907-1911`

이렇게 얻은 `question_response_ids`를 기반으로 질문 텍스트를 순서대로 추출합니다.

> 소스: `service.rs:1917-1920`

---

## 9단계: 카테고리별 응답 필터링

`ResponseCategory::question_index()`를 호출하여 해당 카테고리에 맞는 질문 인덱스를 얻고,
그 질문에 해당하는 응답만 필터링합니다.

```rust
let target_response_ids: Vec<i64> = match category.question_index() {
    Some(idx) => {
        // 특정 질문의 답변만 필터
        let target_question = &question_texts[idx];
        all_responses
            .iter()
            .filter(|r| &r.question == target_question)
            .map(|r| r.response_id)
            .collect()
    }
    None => {
        // ALL: 모든 응답
        all_responses.iter().map(|r| r.response_id).collect()
    }
};
```

> 소스: `service.rs:1923-1945`

추가로 공백만 있는 빈 답변(content가 비어있거나 공백만인 응답)을 필터링합니다.

> 소스: `service.rs:1956-1965`

---

## 10단계: 커서 기반 페이지네이션

`response_id` 내림차순으로 정렬하며, cursor가 있으면 해당 ID보다 작은(=이전) 응답만 조회합니다.
**`size + 1`개를 조회**하여 다음 페이지 존재 여부를 판단합니다.

```rust
let mut query = response::Entity::find()
    .filter(response::Column::ResponseId.is_in(valid_response_ids))
    .order_by_desc(response::Column::ResponseId);

if let Some(cursor_id) = cursor {
    query = query.filter(response::Column::ResponseId.lt(cursor_id));
}

// size + 1개 조회하여 다음 페이지 존재 여부 확인
let fetched = query
    .limit(Some((size + 1) as u64))
    .all(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```

> 소스: `service.rs:1976-1989`

---

## 11단계: has_next / next_cursor 계산

조회된 레코드 수가 `size`보다 많으면 다음 페이지가 존재합니다.
실제 응답에는 `size`개만 포함하며, 마지막 항목의 `response_id`가 다음 커서가 됩니다.

```rust
let has_next = fetched.len() as i64 > size;
let page_responses: Vec<&response::Model> = fetched.iter().take(size as usize).collect();

// ...

let next_cursor = if has_next {
    response_items.last().map(|r| r.response_id)
} else {
    None
};
```

> 소스: `service.rs:1991-1992`, `service.rs:2082-2086`

---

## 12단계: 부가 정보 조회 (멤버 닉네임, 좋아요 수, 댓글 수)

페이지에 포함된 응답들에 대해 다음 정보를 추가 조회합니다:

1. **멤버 닉네임**: `member_response` -> `member` 테이블을 조인하여 작성자 닉네임 획득
2. **좋아요 수**: `response_like` 테이블에서 `response_id`별 COUNT 집계
3. **댓글 수**: `response_comment` 테이블에서 `response_id`별 COUNT 집계

> 소스: `service.rs:2004-2059`

---

## 13단계: DTO 변환 및 응답 반환

조회된 데이터를 `ResponseListItem` DTO로 변환하고, `ResponsesListResponse`에 담아 반환합니다.

```rust
Ok(ResponsesListResponse {
    responses: response_items,
    has_next,
    next_cursor,
})
```

> 소스: `service.rs:2096-2100`

핸들러에서는 `BaseResponse::success_with_message`로 래핑하여 최종 응답을 반환합니다.

> 소스: `handler.rs:618-621`

---

## 전체 흐름 요약

```
클라이언트 요청
    |
    v
[핸들러] retrospectId 검증 (handler.rs:576)
    |
    v
[핸들러] category 파싱 - FromStr (handler.rs:583)
    |
    v
[핸들러] cursor/size 검증 (handler.rs:588-602)
    |
    v
[서비스] 회고 존재 + 팀 멤버십 확인 (service.rs:1862)
    |
    v
[서비스] 전체 응답 데이터 조회 (service.rs:1866)
    |
    v
[서비스] 질문 순서 결정 (service.rs:1883-1920)
    |
    v
[서비스] 카테고리별 필터링 (service.rs:1923-1945)
    |
    v
[서비스] 빈 답변 필터링 (service.rs:1956-1965)
    |
    v
[서비스] 커서 기반 페이지네이션 - size+1 조회 (service.rs:1976-1989)
    |
    v
[서비스] has_next / next_cursor 계산 (service.rs:1991, 2082)
    |
    v
[서비스] 멤버 닉네임 / 좋아요 / 댓글 수 조회 (service.rs:2004-2059)
    |
    v
[서비스] DTO 변환 (service.rs:2062-2079)
    |
    v
[핸들러] BaseResponse 래핑 후 응답 (handler.rs:618)
```
