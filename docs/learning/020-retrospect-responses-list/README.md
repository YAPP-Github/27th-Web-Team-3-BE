# [API-020] 회고 답변 카테고리별 조회 API 학습 노트

## 개요
- **엔드포인트**: `GET /api/v1/retrospects/{retrospectId}/responses`
- **역할**: 특정 회고 세션의 답변 리스트를 질문 카테고리별로 필터링하여 조회하며, 커서 기반 페이지네이션으로 무한 스크롤 UI를 지원한다
- **인증**: Bearer 토큰 필요

## 아키텍처 구조

```
Client  ──GET /{retrospectId}/responses?category=QUESTION_1&cursor=500&size=10──>  Handler (list_responses)
                                  |
                                  +-- AuthUser: JWT에서 user_id 추출
                                  +-- Path(retrospect_id): 경로 파라미터 파싱
                                  +-- Query(params): 쿼리 파라미터 파싱
                                  |
                                  v
                              Handler 검증
                                  |
                                  +-- 1. retrospect_id >= 1 검증
                                  +-- 2. category 문자열 -> ResponseCategory enum 파싱 (FromStr)
                                  +-- 3. cursor 범위 검증 (있을 경우 >= 1)
                                  +-- 4. size 기본값 처리 (unwrap_or(10)) + 범위 검증 (1~100)
                                  |
                                  v
                              Service (list_responses)
                                  |
                                  +-- 1. 회고 존재 확인 + 팀 멤버십 검증
                                  +-- 2. 해당 회고의 전체 응답 조회 (response_id 오름차순)
                                  +-- 3. 질문 순서 결정 (첫 번째 멤버의 응답 기준)
                                  +-- 4. 카테고리별 대상 응답 필터링
                                  +-- 5. 빈 답변 필터링 (content.trim().is_empty())
                                  +-- 6. 커서 기반 페이지네이션 (size+1 조회)
                                  +-- 7. 부가 정보 조회 (닉네임, 좋아요, 댓글)
                                  +-- 8. DTO 변환 + nextCursor 계산
                                  |
                                  v
                          ResponsesListResponse { responses, has_next, next_cursor }
```

## 핵심 코드 분석

### 1. Handler 계층

**소스**: `codes/server/src/domain/retrospect/handler.rs:569-622`

```rust
pub async fn list_responses(
    user: AuthUser,
    State(state): State<AppState>,
    Path(retrospect_id): Path<i64>,
    Query(params): Query<ResponsesQueryParams>,
) -> Result<Json<BaseResponse<ResponsesListResponse>>, AppError> {
    if retrospect_id < 1 {
        return Err(AppError::BadRequest(
            "retrospectId는 1 이상의 양수여야 합니다.".to_string(),
        ));
    }

    let category: ResponseCategory = params.category.parse().map_err(|_| {
        AppError::RetroCategoryInvalid("유효하지 않은 카테고리 값입니다.".to_string())
    })?;

    if let Some(cursor) = params.cursor {
        if cursor < 1 {
            return Err(AppError::BadRequest(
                "cursor는 1 이상의 양수여야 합니다.".to_string(),
            ));
        }
    }

    let size = params.size.unwrap_or(10);
    if !(1..=100).contains(&size) {
        return Err(AppError::BadRequest(
            "size는 1~100 범위의 정수여야 합니다.".to_string(),
        ));
    }

    let user_id = user.user_id()?;
    let result = RetrospectService::list_responses(
        state, user_id, retrospect_id, category, params.cursor, size,
    ).await?;
    Ok(Json(BaseResponse::success_with_message(result, "답변 리스트 조회를 성공했습니다.")))
}
```

- `Path(retrospect_id)`: URL 경로의 `{retrospectId}`를 `i64`로 추출한다.
- `Query(params)`: 쿼리스트링 전체를 `ResponsesQueryParams` 구조체로 역직렬화한다.
- `params.category.parse()`: `category`를 `String`으로 받고 핸들러에서 직접 `FromStr`로 파싱한다. 이렇게 하면 파싱 실패 시 `RetroCategoryInvalid`(RETRO4004) 커스텀 에러 코드를 반환할 수 있다. enum 타입으로 직접 역직렬화하면 Axum이 400을 자동 반환하여 커스텀 에러 코드 지정이 불가능하다.

### 2. DTO 설계

**소스**: `codes/server/src/domain/retrospect/dto.rs:543-661`

| 구조체 | 역할 | 라인 |
|--------|------|------|
| `ResponseCategory` | 카테고리 필터 enum (ALL, QUESTION_1~5) | dto.rs:548-568 |
| `ResponsesQueryParams` | 쿼리 파라미터 (category, cursor, size) | dto.rs:614-623 |
| `ResponseListItem` | 개별 답변 아이템 (id, 이름, 내용, 좋아요, 댓글) | dto.rs:626-639 |
| `ResponsesListResponse` | 최종 응답 (responses, has_next, next_cursor) | dto.rs:642-651 |

### 3. ResponseCategory enum

**소스**: `codes/server/src/domain/retrospect/dto.rs:548-611`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
pub enum ResponseCategory {
    #[serde(rename = "ALL")]
    All,
    #[serde(rename = "QUESTION_1")]
    Question1,
    // ... Question2 ~ Question5
}

impl ResponseCategory {
    pub fn question_index(&self) -> Option<usize> {
        match self {
            ResponseCategory::All => None,
            ResponseCategory::Question1 => Some(0),
            ResponseCategory::Question2 => Some(1),
            ResponseCategory::Question3 => Some(2),
            ResponseCategory::Question4 => Some(3),
            ResponseCategory::Question5 => Some(4),
        }
    }
}

impl std::str::FromStr for ResponseCategory {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ALL" => Ok(ResponseCategory::All),
            "QUESTION_1" => Ok(ResponseCategory::Question1),
            // ... Question2 ~ Question5
            _ => Err(format!("유효하지 않은 카테고리: {}", s)),
        }
    }
}
```

- `question_index()`: `Option<usize>`를 반환하여 "전체 조회"(`None`)와 "특정 질문 조회"(`Some(idx)`)를 타입 레벨에서 구분한다.
- `FromStr` trait: `.parse()` 메서드를 사용할 수 있게 한다. 핸들러에서 `String` -> `ResponseCategory` 변환 시 사용한다.
- `Display` trait: `tracing` 로그에서 `%category` 포맷 지시자로 사용한다.

### 4. Service 비즈니스 로직

**소스**: `codes/server/src/domain/retrospect/service.rs:1843-2101`

**4-1. 회고 존재 확인 및 팀 멤버십 검증 (line 1862-1863)**
```rust
let _retrospect_model =
    Self::find_retrospect_for_member(&state, user_id, retrospect_id).await?;
```
회고가 존재하지 않거나 사용자가 해당 팀의 멤버가 아니면 에러를 반환한다. 보안상 존재하지 않는 경우와 권한이 없는 경우를 구분하지 않고 404를 반환한다.

**4-2. 전체 응답 조회 (line 1866-1871)**
```rust
let all_responses = response::Entity::find()
    .filter(response::Column::RetrospectId.eq(retrospect_id))
    .order_by_asc(response::Column::ResponseId)
    .all(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```
해당 회고의 모든 응답을 `response_id` 오름차순으로 조회한다. 빈 결과면 즉시 빈 응답을 반환한다.

**4-3. 질문 순서 결정 (line 1883-1920)**
```rust
let first_member_id = member_response_map.keys().min().copied();
let question_response_ids: Vec<i64> = first_member_id
    .and_then(|mid| member_response_map.get(&mid))
    .cloned()
    .unwrap_or_default();
```
`member_response` 테이블에서 `member_id`가 가장 작은(첫 번째 등록된) 멤버의 응답 순서를 기준으로 질문 텍스트 순서를 확정한다. 이 순서가 `QUESTION_1` ~ `QUESTION_5` 인덱스에 매핑된다.

**4-4. 카테고리별 필터링 (line 1923-1945)**
```rust
let target_response_ids: Vec<i64> = match category.question_index() {
    Some(idx) => {
        let target_question = &question_texts[idx];
        all_responses
            .iter()
            .filter(|r| &r.question == target_question)
            .map(|r| r.response_id)
            .collect()
    }
    None => {
        all_responses.iter().map(|r| r.response_id).collect()
    }
};
```
`question_index()`가 `Some(idx)`이면 해당 인덱스의 질문 텍스트와 일치하는 응답만 필터링한다. `None`(ALL)이면 전체 응답을 대상으로 한다.

**4-5. 빈 답변 필터링 (line 1956-1965)**
```rust
let valid_response_ids: Vec<i64> = target_response_ids
    .iter()
    .filter(|rid| {
        response_map
            .get(rid)
            .map(|r| !r.content.trim().is_empty())
            .unwrap_or(false)
    })
    .copied()
    .collect();
```
`content`가 비어있거나 공백만 있는 응답을 제외한다. `trim().is_empty()`로 공백만 있는 경우도 빈 답변으로 처리한다.

**4-6. 커서 기반 페이지네이션 (line 1976-1992)**
```rust
let mut query = response::Entity::find()
    .filter(response::Column::ResponseId.is_in(valid_response_ids))
    .order_by_desc(response::Column::ResponseId);

if let Some(cursor_id) = cursor {
    query = query.filter(response::Column::ResponseId.lt(cursor_id));
}

let fetched = query
    .limit(Some((size + 1) as u64))
    .all(&state.db).await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

let has_next = fetched.len() as i64 > size;
let page_responses: Vec<&response::Model> = fetched.iter().take(size as usize).collect();
```
- `response_id` 내림차순(최신순)으로 정렬한다.
- cursor가 있으면 `response_id < cursor_id` 조건을 추가한다.
- `size + 1`개를 조회하여 `has_next`를 판단한다. 실제 응답에는 `size`개만 포함한다.

**4-7. 부가 정보 조회 (line 2004-2059)**
```rust
// member_response -> member 조인으로 닉네임 조회
let response_to_member: HashMap<i64, i64> = member_responses_for_page
    .iter().map(|mr| (mr.response_id, mr.member_id)).collect();

// 좋아요 수 집계 (GROUP BY + COUNT)
let like_counts: Vec<(i64, i64)> = response_like::Entity::find()
    .filter(response_like::Column::ResponseId.is_in(page_response_ids.clone()))
    .select_only()
    .column(response_like::Column::ResponseId)
    .column_as(response_like::Column::ResponseLikeId.count(), "count")
    .group_by(response_like::Column::ResponseId)
    .into_tuple()
    .all(&state.db).await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```
- 닉네임: `member_response` -> `member` 테이블 경유
- 좋아요 수: `response_like` 테이블에서 `GROUP BY response_id + COUNT`
- 댓글 수: `response_comment` 테이블에서 `GROUP BY response_id + COUNT`

**4-8. nextCursor 계산 (line 2082-2086)**
```rust
let next_cursor = if has_next {
    response_items.last().map(|r| r.response_id)
} else {
    None
};
```
다음 페이지가 있으면 마지막 아이템의 `response_id`를 `nextCursor`로 반환한다. 클라이언트는 이 값을 다음 요청의 `cursor` 파라미터로 전달한다.

## 사용된 Rust 패턴

### 1. FromStr Trait을 활용한 수동 파싱
```rust
impl std::str::FromStr for ResponseCategory {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> { ... }
}
// 사용: params.category.parse()
```
`FromStr`을 구현하면 `.parse()` 메서드를 사용할 수 있다. 쿼리 파라미터를 `String`으로 받고 핸들러에서 수동 파싱하면 커스텀 에러 코드를 지정할 수 있다.

### 2. size + 1 조회 Trick
```rust
let fetched = query.limit(Some((size + 1) as u64)).all(&state.db).await?;
let has_next = fetched.len() as i64 > size;
let page_responses = fetched.iter().take(size as usize).collect();
```
별도의 `COUNT` 쿼리 없이 다음 페이지 존재 여부를 판단한다. DB 요청을 1회로 줄여 성능을 향상시킨다.

### 3. Option 체이닝 (and_then)
```rust
let user_name = member_id
    .and_then(|mid| member_map.get(&mid))
    .and_then(|m| m.nickname.clone())
    .unwrap_or_default();
```
여러 단계의 `Option` 변환을 체이닝하여, 하나라도 `None`이면 빈 문자열을 반환한다. `map`은 `Option<Option<T>>`을 만들지만, `and_then`은 `Option<T>`로 평탄화한다.

### 4. RangeInclusive를 활용한 범위 검증
```rust
if !(1..=100).contains(&size) {
    return Err(AppError::BadRequest(...));
}
```
`1..=100`은 `RangeInclusive<i64>` 타입이다. `contains()`로 범위 검증을 간결하게 수행한다. `if size < 1 || size > 100`과 동일하지만 가독성이 높다.

### 5. SeaORM GROUP BY + COUNT 집계
```rust
let like_counts: Vec<(i64, i64)> = response_like::Entity::find()
    .select_only()
    .column(response_like::Column::ResponseId)
    .column_as(response_like::Column::ResponseLikeId.count(), "count")
    .group_by(response_like::Column::ResponseId)
    .into_tuple()
    .all(&state.db).await?;
```
`select_only()` + `column_as()` + `group_by()` + `into_tuple()`로 SQL의 `SELECT response_id, COUNT(response_like_id) FROM ... GROUP BY response_id`를 구성한다. 결과를 `Vec<(i64, i64)>` 튜플로 받아 `HashMap`으로 변환한다.

## DB 쿼리 흐름

총 7회의 DB 쿼리가 발생한다 (빈 결과 조기 반환 시 줄어들 수 있음).

```
Query 1: 회고 존재 + 팀 멤버십 확인
         find_retrospect_for_member()
         소스: service.rs:1862

Query 2: 해당 회고의 전체 응답 조회
         SELECT * FROM response WHERE retrospect_id = ? ORDER BY response_id ASC
         소스: service.rs:1866-1871

Query 3: member_response 조회 (질문 순서 결정용)
         SELECT * FROM member_response WHERE response_id IN (...) ORDER BY response_id ASC
         소스: service.rs:1883-1895

Query 4: 커서 기반 페이지네이션 조회
         SELECT * FROM response WHERE response_id IN (...)
           AND response_id < ? ORDER BY response_id DESC LIMIT size+1
         소스: service.rs:1976-1989

Query 5: 페이지 내 응답의 member_response + member 조회 (닉네임)
         소스: service.rs:2006-2031

Query 6: 좋아요 수 집계
         SELECT response_id, COUNT(*) FROM response_like
           WHERE response_id IN (...) GROUP BY response_id
         소스: service.rs:2034-2043

Query 7: 댓글 수 집계
         SELECT response_id, COUNT(*) FROM response_comment
           WHERE response_id IN (...) GROUP BY response_id
         소스: service.rs:2048-2057
```

## 학습 포인트

### 새롭게 알게 된 점
1. **FromStr trait으로 커스텀 에러 코드 제어**: DTO에서 `String`으로 받고 핸들러에서 수동 파싱하면, Axum 자동 역직렬화 실패(400)가 아닌 커스텀 에러 코드(RETRO4004)를 지정할 수 있다.
2. **size + 1 조회 기법**: 별도의 `COUNT(*)` 쿼리 없이 `LIMIT size+1`로 다음 페이지 존재를 판단한다. 페이지네이션의 대표적 최적화 패턴이다.
3. **커서 기반 페이지네이션**: Offset 방식(`OFFSET N LIMIT M`)은 페이지가 깊어질수록 성능이 저하된다. 커서 방식(`WHERE id < cursor LIMIT M`)은 인덱스 기반이므로 항상 일정한 성능을 보장한다.
4. **SeaORM의 select_only + into_tuple 패턴**: 집계 쿼리(GROUP BY + COUNT)를 ORM에서 타입 안전하게 구성하고, 결과를 튜플로 받아 `HashMap`으로 변환하는 패턴이다.
5. **Option::and_then을 활용한 체이닝**: 여러 단계의 `Option` 변환을 `and_then`으로 평탄화하면 중첩 `match`/`if let` 없이 깔끔한 코드를 작성할 수 있다.

### 설계 결정의 Trade-offs

| 결정 | 장점 | 단점 |
|------|------|------|
| category를 String으로 받고 수동 파싱 | 커스텀 에러 코드(RETRO4004) 지정 가능 | 타입 안전성을 핸들러 단계까지 포기 |
| 질문 순서를 첫 번째 멤버 기준으로 결정 | 고정된 질문 순서 보장 | 첫 번째 멤버의 응답이 없으면 빈 결과 |
| size + 1 조회로 has_next 판단 | COUNT 쿼리 불필요, DB 1회 호출 | 1건 더 조회하므로 미세한 오버헤드 |
| 빈 답변(공백만) 필터링 | 의미 있는 답변만 표시 | 어플리케이션 레벨 필터링으로 유효 건수 변동 |
| 커서 기반 페이지네이션 | 대규모 데이터에서 일정한 성능 | 임의 페이지 이동 불가 |
| 집계 쿼리(GROUP BY + COUNT) | DB 수준 집계로 정확한 결과 | 추가 쿼리 2회 (좋아요 + 댓글) |

## 학습 문서 안내

| 문서 | 내용 |
|------|------|
| [flow.md](./flow.md) | 요청부터 응답까지의 전체 동작 흐름 |
| [key-concepts.md](./key-concepts.md) | 커서 기반 페이지네이션, FromStr trait, size+1 trick 등 핵심 개념 |
| [keywords.md](./keywords.md) | 학습해야 할 Rust/웹 개발 키워드 목록 |
| [study-guide.md](./study-guide.md) | Spring 개발자를 위한 학습 가이드 |

## 참고
- 핸들러: `codes/server/src/domain/retrospect/handler.rs` (line 545-622)
- 서비스: `codes/server/src/domain/retrospect/service.rs` (line 1843-2101)
- DTO: `codes/server/src/domain/retrospect/dto.rs` (line 543-661)
- API 스펙: `docs/api-specs/020-retrospect-responses-list.md`
