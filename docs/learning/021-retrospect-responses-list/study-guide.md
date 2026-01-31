# 학습 가이드: API-020 회고 답변 카테고리별 조회

이 문서는 **JVM 기반 Spring만 익숙한 개발자**가 API-020의 모든 코드를 읽고 이해할 수 있도록 작성한 학습 가이드입니다. Rust 문법, Axum 프레임워크, SeaORM을 **Spring 개념에 1:1 매핑**해서 설명합니다.

---

## 1) 한눈에 보는 기술 매핑 (Spring <-> Rust)

| Spring (JVM) | 이 프로젝트 (Rust) | 파일 위치 |
|---|---|---|
| `@RestController` | `handler.rs`의 함수 | `domain/retrospect/handler.rs:569` |
| `@Service` | `service.rs`의 함수 | `domain/retrospect/service.rs:1843` |
| `JpaRepository` / QueryDSL | SeaORM `Entity::find()` | `domain/retrospect/entity/` |
| `@PathVariable` | `Path<i64>` | `handler.rs:572` |
| `@RequestParam` (묶음) | `Query<ResponsesQueryParams>` | `handler.rs:573` |
| `ResponseEntity<T>` | `Json<BaseResponse<T>>` | `utils/response.rs` |
| `@ControllerAdvice` | `AppError` + `IntoResponse` | `utils/error.rs` |
| Spring Security Filter | `AuthUser` extractor | `utils/auth.rs` |
| `@Autowired` / DI | `State(state): State<AppState>` | `handler.rs:571` |
| `enum` (Java) | `enum` (Rust) | `dto.rs:548` |
| `@JsonProperty` | `#[serde(rename)]` | `dto.rs:551-567` |
| `Optional<T>` | `Option<T>` | `dto.rs:620-622` |
| `throws Exception` | `Result<T, AppError>` | 거의 모든 함수 |
| `Stream API` | Iterator + `collect()` | `service.rs:1917, 2062` |
| `HashMap` (Java) | `HashMap` | `service.rs:1898, 2012, 2045, 2059` |
| `Pageable` / `Slice` | 커서 기반 직접 구현 | `service.rs:1976-1992` |
| `@Schema` (springdoc) | `ToSchema` derive | `dto.rs:548, 627, 643` |
| `@Parameter` (springdoc) | `IntoParams` derive | `dto.rs:614` |

---

## 2) API-020 기능 요약

- **엔드포인트**: `GET /api/v1/retrospects/{retrospectId}/responses`
- **기능**: 회고 답변을 질문 카테고리(ALL, QUESTION_1~5)별로 조회
- **인증**: JWT Bearer 토큰 필수
- **페이지네이션**: 커서 기반 (response_id 기준), `size + 1` 조회

**Spring으로 작성했다면 이런 느낌:**

```java
@GetMapping("/api/v1/retrospects/{retrospectId}/responses")
public ResponseEntity<BaseResponse<ResponsesListResponse>> listResponses(
        @AuthenticationPrincipal UserDetails user,
        @PathVariable Long retrospectId,
        @RequestParam String category,
        @RequestParam(required = false) Long cursor,
        @RequestParam(defaultValue = "10") Integer size) {
    ResponseCategory cat = ResponseCategory.valueOf(category);
    ResponsesListResponse result = retrospectService.listResponses(
        user.getId(), retrospectId, cat, cursor, size);
    return ResponseEntity.ok(BaseResponse.success(result));
}
```

이것의 **Rust/Axum 버전**이 이 API의 코드이다.

---

## 3) 요청 흐름 (Spring 기준으로 이해하기)

```
클라이언트
  GET /api/v1/retrospects/42/responses?category=QUESTION_1&cursor=500&size=10
  Authorization: Bearer {token}
        |
        v
  [1] AuthUser Extractor          <- Spring Security Filter 역할
        JWT 토큰 검증, user_id 추출
        |
        v
  [2] Handler (list_responses)    <- @RestController 메서드 역할
        Path/Query 파라미터 파싱, 검증
        |
        v
  [3] Service (list_responses)    <- @Service 메서드 역할
        DB 조회, 필터링, 페이지네이션
        |
        v
  [4] SeaORM Entity::find()       <- JpaRepository 역할
        SQL 생성/실행 (총 7회 쿼리)
        |
        v
  [5] BaseResponse 래핑           <- ResponseEntity 역할
        JSON 직렬화
```

### 단계별 소스 파일 위치

| 단계 | Spring 대응 | 파일 |
|------|------------|------|
| JWT 인증 | Security Filter | `src/utils/auth.rs` |
| 핸들러 | Controller | `src/domain/retrospect/handler.rs:569-622` |
| 서비스 | Service | `src/domain/retrospect/service.rs:1843-2101` |
| Entity/ORM | Repository + Entity | `src/domain/retrospect/entity/` |
| DTO | DTO/VO | `src/domain/retrospect/dto.rs:543-661` |
| 응답 래핑 | ResponseEntity | `src/utils/response.rs` |
| 에러 처리 | @ControllerAdvice | `src/utils/error.rs` |

---

## 4) Rust 문법 핵심 (이 API를 읽기 위한 최소한의 지식)

### 4-1. `FromStr` trait = Java의 `valueOf()` / `parse()`

Rust의 `FromStr` trait을 구현하면 `.parse()` 메서드를 사용할 수 있다. Java enum의 `valueOf()`와 비슷하다.

```rust
// Rust
impl std::str::FromStr for ResponseCategory {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ALL" => Ok(ResponseCategory::All),
            "QUESTION_1" => Ok(ResponseCategory::Question1),
            // ...
            _ => Err(format!("유효하지 않은 카테고리: {}", s)),
        }
    }
}

// 사용
let category: ResponseCategory = "QUESTION_1".parse()?;
```

```java
// Java 대응
ResponseCategory category = ResponseCategory.valueOf("QUESTION_1");
// 잘못된 값이면 IllegalArgumentException
```

핵심 차이: Java의 `valueOf()`는 예외를 던지지만, Rust의 `parse()`는 `Result`를 반환한다. 호출자가 에러를 명시적으로 처리해야 한다.

### 4-2. `Option<T>` 패턴들

이 API에서 `Option`은 3가지 패턴으로 사용된다.

**기본값 제공 (unwrap_or)**
```rust
// Rust
let size = params.size.unwrap_or(10);
```
```java
// Java
int size = Optional.ofNullable(params.getSize()).orElse(10);
```

**조건부 실행 (if let Some)**
```rust
// Rust
if let Some(cursor) = params.cursor {
    if cursor < 1 { return Err(...); }
}
```
```java
// Java
if (params.getCursor() != null) {
    if (params.getCursor() < 1) { throw new BadRequestException(...); }
}
```

**체이닝 (and_then)**
```rust
// Rust
let user_name = member_id
    .and_then(|mid| member_map.get(&mid))     // Option<i64> -> Option<&Model>
    .and_then(|m| m.nickname.clone())          // Option<&Model> -> Option<String>
    .unwrap_or_default();                       // None이면 빈 문자열
```
```java
// Java
String userName = Optional.ofNullable(memberId)
    .map(mid -> memberMap.get(mid))
    .map(m -> m.getNickname())
    .orElse("");
```

`and_then`은 Java의 `flatMap`에 해당한다. `map`은 `Option<Option<T>>`을 만들지만, `and_then`은 `Option<T>`로 평탄화한다.

### 4-3. `RangeInclusive::contains()` = 범위 검증

```rust
// Rust
if !(1..=100).contains(&size) {
    return Err(AppError::BadRequest(...));
}
```
```java
// Java
if (size < 1 || size > 100) {
    throw new BadRequestException(...);
}
```

`1..=100`은 `RangeInclusive<i64>` 타입이다. 가독성이 높고, 범위 경계가 한눈에 보인다.

### 4-4. `Iterator::take()` = Java `Stream.limit()`

```rust
// Rust
let page_responses: Vec<&response::Model> = fetched.iter()
    .take(size as usize)   // 최대 size개만 가져옴
    .collect();
```
```java
// Java
List<Response> pageResponses = fetched.stream()
    .limit(size)           // 최대 size개만 가져옴
    .collect(Collectors.toList());
```

### 4-5. `HashSet`으로 중복 제거

```rust
// Rust
let member_ids: Vec<i64> = response_to_member
    .values()
    .copied()
    .collect::<HashSet<i64>>()   // 중복 제거
    .into_iter()
    .collect();                   // 다시 Vec으로
```
```java
// Java
List<Long> memberIds = new ArrayList<>(
    new HashSet<>(responseToMember.values())  // 중복 제거
);
```

---

## 5) Handler 코드 상세 (Spring Controller 역할)

**파일**: `handler.rs:569-622`

```rust
pub async fn list_responses(
    user: AuthUser,                              // [1] JWT 인증
    State(state): State<AppState>,               // [2] DI (의존성 주입)
    Path(retrospect_id): Path<i64>,              // [3] 경로 파라미터
    Query(params): Query<ResponsesQueryParams>,   // [4] 쿼리 파라미터
) -> Result<Json<BaseResponse<ResponsesListResponse>>, AppError> {
    // [5] retrospectId 검증
    if retrospect_id < 1 { ... }

    // [6] category 파싱 (FromStr)
    let category: ResponseCategory = params.category.parse().map_err(|_| {
        AppError::RetroCategoryInvalid("유효하지 않은 카테고리 값입니다.".to_string())
    })?;

    // [7] cursor/size 검증
    if let Some(cursor) = params.cursor {
        if cursor < 1 { ... }
    }
    let size = params.size.unwrap_or(10);
    if !(1..=100).contains(&size) { ... }

    // [8] user_id 추출 + 서비스 호출
    let user_id = user.user_id()?;
    let result = RetrospectService::list_responses(
        state, user_id, retrospect_id, category, params.cursor, size,
    ).await?;
    Ok(Json(BaseResponse::success_with_message(result, "답변 리스트 조회를 성공했습니다.")))
}
```

### [3] `Path(retrospect_id): Path<i64>` -- @PathVariable

URL 경로의 `{retrospectId}` 부분을 `i64`로 추출한다.

```java
// Spring 대응
@PathVariable Long retrospectId
```

### [4] `Query(params): Query<ResponsesQueryParams>` -- @RequestParam 묶음

URL의 `?category=QUESTION_1&cursor=500&size=10` 전체를 구조체로 역직렬화한다.

```rust
// Axum - 쿼리 파라미터를 구조체로 한번에 역직렬화
#[derive(Debug, Deserialize, IntoParams)]
pub struct ResponsesQueryParams {
    pub category: String,        // 필수 (String으로 받아서 수동 파싱)
    pub cursor: Option<i64>,     // 선택적
    pub size: Option<i64>,       // 선택적 (기본값 10)
}
```

```java
// Spring 대응 - 개별 파라미터로
@RequestParam String category,
@RequestParam(required = false) Long cursor,
@RequestParam(defaultValue = "10") Integer size
```

### [6] category를 String으로 받는 설계 결정

`ResponsesQueryParams`에서 `category`를 `ResponseCategory` enum이 아닌 `String`으로 정의한 이유:

```rust
// 현재 방식: String으로 받고 수동 파싱 → 커스텀 에러 코드 지정 가능
pub category: String,
// 핸들러에서:
let category: ResponseCategory = params.category.parse().map_err(|_| {
    AppError::RetroCategoryInvalid(...)  // RETRO4004 에러 코드
})?;

// 대안: enum으로 직접 역직렬화 → Axum이 자동 400 반환 (커스텀 코드 불가)
pub category: ResponseCategory,
// 잘못된 값 → QueryRejection → 일반 400 에러 (에러 코드 지정 불가)
```

```java
// Spring에서도 비슷한 상황:
// @RequestParam으로 enum을 직접 받으면 MethodArgumentTypeMismatchException → 400
// 직접 String으로 받아서 변환하면 커스텀 예외 코드 지정 가능
```

---

## 6) Service 코드 상세 (비즈니스 로직)

**파일**: `service.rs:1843-2101`

### 6-1. 함수 시그니처

```rust
pub async fn list_responses(
    state: AppState,           // DB 커넥션 등 전역 상태
    user_id: i64,              // JWT에서 추출한 사용자 ID
    retrospect_id: i64,        // 경로 파라미터
    category: ResponseCategory, // 파싱 완료된 카테고리 enum
    cursor: Option<i64>,       // 커서 (없으면 첫 페이지)
    size: i64,                 // 페이지 크기
) -> Result<ResponsesListResponse, AppError> {
```

```java
// Spring 대응
public ResponsesListResponse listResponses(
        Long userId, Long retrospectId,
        ResponseCategory category, Long cursor, int size)
        throws AppException {
```

### 6-2. 회고 존재 + 팀 멤버십 확인

```rust
let _retrospect_model =
    Self::find_retrospect_for_member(&state, user_id, retrospect_id).await?;
```

```java
// Spring 대응
Retrospect retrospect = retrospectRepository.findById(retrospectId)
    .orElseThrow(() -> new NotFoundException("회고를 찾을 수 없습니다."));
if (!teamMemberRepository.existsByTeamIdAndMemberId(retrospect.getTeamId(), userId)) {
    throw new NotFoundException("회고를 찾을 수 없습니다.");  // 보안상 404
}
```

보안상 "존재하지 않음"과 "권한 없음"을 구분하지 않고 404를 반환한다. 공격자가 리소스 존재 여부를 추측하는 것을 방지하기 위해서다.

### 6-3. 질문 순서 결정 로직

```rust
// 1. member_response에서 모든 응답-멤버 관계 조회
let first_member_responses = member_response::Entity::find()
    .filter(member_response::Column::ResponseId.is_in(all_response_ids))
    .order_by_asc(member_response::Column::ResponseId)
    .all(&state.db).await?;

// 2. member_id별로 그룹화
let mut member_response_map: HashMap<i64, Vec<i64>> = HashMap::new();
for mr in &first_member_responses {
    member_response_map.entry(mr.member_id).or_default().push(mr.response_id);
}

// 3. 가장 작은 member_id의 응답 순서를 기준으로 질문 순서 확정
let first_member_id = member_response_map.keys().min().copied();
let question_response_ids: Vec<i64> = first_member_id
    .and_then(|mid| member_response_map.get(&mid))
    .cloned()
    .unwrap_or_default();

// 4. 질문 텍스트를 response_id 순서대로 추출
let question_texts: Vec<String> = question_response_ids
    .iter()
    .filter_map(|rid| response_map.get(rid).map(|r| r.question.clone()))
    .collect();
```

```java
// Spring 대응
Map<Long, List<Long>> memberResponseMap = memberResponses.stream()
    .collect(Collectors.groupingBy(
        MemberResponse::getMemberId,
        Collectors.mapping(MemberResponse::getResponseId, Collectors.toList())
    ));

Long firstMemberId = memberResponseMap.keySet().stream().min(Long::compareTo).orElse(null);
List<Long> questionResponseIds = firstMemberId != null
    ? memberResponseMap.get(firstMemberId) : List.of();

List<String> questionTexts = questionResponseIds.stream()
    .map(rid -> responseMap.get(rid).getQuestion())
    .filter(Objects::nonNull)
    .collect(Collectors.toList());
```

이 방식으로 `QUESTION_1`은 첫 번째 멤버의 첫 번째 응답 질문, `QUESTION_2`는 두 번째 응답 질문에 매핑된다.

### 6-4. 카테고리별 필터링

```rust
let target_response_ids: Vec<i64> = match category.question_index() {
    Some(idx) => {
        if idx >= question_texts.len() {
            return Ok(ResponsesListResponse { responses: vec![], has_next: false, next_cursor: None });
        }
        let target_question = &question_texts[idx];
        all_responses.iter()
            .filter(|r| &r.question == target_question)
            .map(|r| r.response_id)
            .collect()
    }
    None => {
        all_responses.iter().map(|r| r.response_id).collect()
    }
};
```

```java
// Spring 대응
List<Long> targetResponseIds;
if (category == ResponseCategory.ALL) {
    targetResponseIds = allResponses.stream()
        .map(Response::getResponseId).collect(toList());
} else {
    int idx = category.getQuestionIndex();
    if (idx >= questionTexts.size()) {
        return new ResponsesListResponse(List.of(), false, null);
    }
    String targetQuestion = questionTexts.get(idx);
    targetResponseIds = allResponses.stream()
        .filter(r -> r.getQuestion().equals(targetQuestion))
        .map(Response::getResponseId).collect(toList());
}
```

`question_index()`가 `None`(ALL)이면 전체 응답, `Some(idx)`이면 해당 인덱스의 질문 텍스트와 일치하는 응답만 필터링한다.

### 6-5. 빈 답변 필터링

```rust
let valid_response_ids: Vec<i64> = target_response_ids
    .iter()
    .filter(|rid| {
        response_map.get(rid)
            .map(|r| !r.content.trim().is_empty())
            .unwrap_or(false)
    })
    .copied()
    .collect();
```

```java
// Spring 대응
List<Long> validResponseIds = targetResponseIds.stream()
    .filter(rid -> {
        Response r = responseMap.get(rid);
        return r != null && !r.getContent().trim().isEmpty();
    })
    .collect(Collectors.toList());
```

`content`가 비어있거나 공백만 있는 응답을 제외한다. 의미 있는 답변만 사용자에게 노출한다.

### 6-6. 커서 기반 페이지네이션 (핵심)

```rust
// 1. 기본 쿼리: valid_response_ids에 해당하는 응답을 response_id 내림차순으로
let mut query = response::Entity::find()
    .filter(response::Column::ResponseId.is_in(valid_response_ids))
    .order_by_desc(response::Column::ResponseId);

// 2. 커서가 있으면 해당 ID보다 작은 것만 (= 이전 항목들)
if let Some(cursor_id) = cursor {
    query = query.filter(response::Column::ResponseId.lt(cursor_id));
}

// 3. size + 1개 조회 (다음 페이지 판단용)
let fetched = query
    .limit(Some((size + 1) as u64))
    .all(&state.db).await?;

// 4. has_next 판단 + 실제 페이지 크기로 자르기
let has_next = fetched.len() as i64 > size;
let page_responses: Vec<&response::Model> = fetched.iter().take(size as usize).collect();
```

```java
// Spring 대응 (JPA + Slice)
Pageable pageable = PageRequest.of(0, size + 1, Sort.by(DESC, "responseId"));
Specification<Response> spec = (root, query, cb) -> {
    List<Predicate> predicates = new ArrayList<>();
    predicates.add(root.get("responseId").in(validResponseIds));
    if (cursor != null) {
        predicates.add(cb.lessThan(root.get("responseId"), cursor));
    }
    return cb.and(predicates.toArray(new Predicate[0]));
};
List<Response> fetched = responseRepository.findAll(spec, pageable).getContent();

boolean hasNext = fetched.size() > size;
List<Response> pageResponses = fetched.subList(0, Math.min(fetched.size(), size));
```

#### size + 1 조회 Trick 설명

| 상황 | size=10, DB에 데이터 15건 | size=10, DB에 데이터 8건 |
|------|-------------------------|------------------------|
| 조회 | 11건 조회 (10+1) | 8건 조회 |
| has_next | `11 > 10` = true | `8 > 10` = false |
| 실제 응답 | 10건 (take로 자름) | 8건 (전부) |
| 별도 COUNT 쿼리 | 불필요 | 불필요 |

이 기법은 `COUNT(*)` 쿼리를 절약하여 DB 부하를 줄인다.

### 6-7. 부가 정보 조회

```rust
// 닉네임: member_response -> member 테이블 경유
let response_to_member: HashMap<i64, i64> = member_responses_for_page
    .iter().map(|mr| (mr.response_id, mr.member_id)).collect();

let member_map: HashMap<i64, &member::Model> =
    members.iter().map(|m| (m.member_id, m)).collect();
```

```java
// Spring 대응
Map<Long, Long> responseToMember = memberResponses.stream()
    .collect(Collectors.toMap(
        MemberResponse::getResponseId,
        MemberResponse::getMemberId
    ));

Map<Long, Member> memberMap = members.stream()
    .collect(Collectors.toMap(Member::getMemberId, Function.identity()));
```

### 6-8. 집계 쿼리 (좋아요/댓글 수)

```rust
// SeaORM의 집계 쿼리: SELECT response_id, COUNT(*) FROM response_like GROUP BY response_id
let like_counts: Vec<(i64, i64)> = response_like::Entity::find()
    .filter(response_like::Column::ResponseId.is_in(page_response_ids.clone()))
    .select_only()
    .column(response_like::Column::ResponseId)
    .column_as(response_like::Column::ResponseLikeId.count(), "count")
    .group_by(response_like::Column::ResponseId)
    .into_tuple()
    .all(&state.db).await?;

let like_count_map: HashMap<i64, i64> = like_counts.into_iter().collect();
```

```java
// Spring JPA 대응
@Query("SELECT r.responseId, COUNT(l.id) FROM ResponseLike l " +
       "WHERE l.responseId IN :ids GROUP BY l.responseId")
List<Object[]> countLikesByResponseIds(@Param("ids") List<Long> ids);

// 또는 QueryDSL
Map<Long, Long> likeCounts = queryFactory
    .select(responseLike.responseId, responseLike.count())
    .from(responseLike)
    .where(responseLike.responseId.in(pageResponseIds))
    .groupBy(responseLike.responseId)
    .fetch()
    .stream()
    .collect(Collectors.toMap(t -> t.get(0, Long.class), t -> t.get(1, Long.class)));
```

#### SeaORM 집계 API 분해

| 메서드 | SQL 대응 | 설명 |
|--------|---------|------|
| `.select_only()` | 커스텀 SELECT 시작 | 자동 `SELECT *` 비활성화 |
| `.column(Col)` | `SELECT col` | 특정 컬럼 선택 |
| `.column_as(expr, alias)` | `SELECT expr AS alias` | 표현식 + 별칭 |
| `.count()` | `COUNT(col)` | 집계 함수 |
| `.group_by(Col)` | `GROUP BY col` | 그룹화 |
| `.into_tuple()` | 결과를 튜플로 | `Vec<(i64, i64)>` 반환 |

### 6-9. DTO 변환

```rust
let response_items: Vec<ResponseListItem> = page_responses
    .iter()
    .map(|r| {
        let member_id = response_to_member.get(&r.response_id).copied();
        let user_name = member_id
            .and_then(|mid| member_map.get(&mid))
            .and_then(|m| m.nickname.clone())
            .unwrap_or_default();

        ResponseListItem {
            response_id: r.response_id,
            user_name,
            content: r.content.clone(),
            like_count: like_count_map.get(&r.response_id).copied().unwrap_or(0),
            comment_count: comment_count_map.get(&r.response_id).copied().unwrap_or(0),
        }
    })
    .collect();
```

```java
// Spring 대응
List<ResponseListItem> responseItems = pageResponses.stream()
    .map(r -> {
        Long memberId = responseToMember.get(r.getResponseId());
        String userName = Optional.ofNullable(memberId)
            .map(memberMap::get)
            .map(Member::getNickname)
            .orElse("");
        return new ResponseListItem(
            r.getResponseId(),
            userName,
            r.getContent(),
            likeCounts.getOrDefault(r.getResponseId(), 0L),
            commentCounts.getOrDefault(r.getResponseId(), 0L)
        );
    })
    .collect(Collectors.toList());
```

### 6-10. nextCursor 계산

```rust
let next_cursor = if has_next {
    response_items.last().map(|r| r.response_id)
} else {
    None
};
```

```java
// Spring 대응
Long nextCursor = hasNext
    ? responseItems.get(responseItems.size() - 1).getResponseId()
    : null;
```

다음 페이지가 있으면 마지막 아이템의 `response_id`를 커서로 반환한다. 클라이언트는 이 값을 다음 요청의 `cursor` 파라미터로 전달한다.

---

## 7) DTO 코드 상세

**파일**: `dto.rs:543-661`

### 7-1. ResponseCategory enum (Java enum과 비교)

```rust
// Rust
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
pub enum ResponseCategory {
    #[serde(rename = "ALL")]
    All,
    #[serde(rename = "QUESTION_1")]
    Question1,
    // ... Question2 ~ Question5
}
```

```java
// Java 대응
public enum ResponseCategory {
    ALL(null),
    QUESTION_1(0),
    QUESTION_2(1),
    QUESTION_3(2),
    QUESTION_4(3),
    QUESTION_5(4);

    private final Integer questionIndex;

    ResponseCategory(Integer index) { this.questionIndex = index; }
    public Integer getQuestionIndex() { return questionIndex; }
}
```

#### API-019의 StorageRangeFilter와 비교

| 특성 | ResponseCategory (API-020) | StorageRangeFilter (API-019) |
|------|---------------------------|------------------------------|
| `Default` derive | 없음 | 있음 (`#[default] All`) |
| `FromStr` 구현 | 수동 구현 | 없음 (serde 역직렬화 의존) |
| 쿼리 파라미터 타입 | `String` (수동 파싱) | `Option<StorageRangeFilter>` (자동 역직렬화) |
| 에러 처리 | 커스텀 코드 (RETRO4004) | Axum 자동 400 |
| `Display` trait | 구현 | 구현 |

같은 프로젝트 내에서도 요구사항에 따라 다른 전략을 선택한다. API-020은 커스텀 에러 코드가 필요하므로 String + FromStr 방식을 사용하고, API-019는 기본값 처리가 중요하므로 Default + Option 방식을 사용한다.

### 7-2. ResponsesQueryParams

```rust
#[derive(Debug, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct ResponsesQueryParams {
    pub category: String,        // 필수 (없으면 Axum이 400 반환)
    pub cursor: Option<i64>,     // 선택적 (없으면 None)
    pub size: Option<i64>,       // 선택적 (없으면 None -> 기본값 10)
}
```

- `category`가 `String`이지 `Option<String>`이 아니므로, 쿼리스트링에 `category`가 없으면 Axum이 자동으로 400을 반환한다.
- `cursor`와 `size`는 `Option`이므로 생략 가능하다.

### 7-3. Response DTO 구조체

```rust
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResponseListItem {
    pub response_id: i64,     // JSON: "responseId"
    pub user_name: String,    // JSON: "userName"
    pub content: String,      // JSON: "content"
    pub like_count: i64,      // JSON: "likeCount"
    pub comment_count: i64,   // JSON: "commentCount"
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResponsesListResponse {
    pub responses: Vec<ResponseListItem>,   // JSON: "responses"
    pub has_next: bool,                     // JSON: "hasNext"
    pub next_cursor: Option<i64>,           // JSON: "nextCursor" (null 가능)
}
```

`next_cursor`가 `Option<i64>`이므로 마지막 페이지에서는 JSON에 `"nextCursor": null`로 직렬화된다.

---

## 8) 에러 처리

### 이 API에서 발생하는 에러 종류

| 상황 | 에러 타입 | HTTP 코드 | 에러 코드 |
|------|----------|----------|----------|
| retrospectId < 1 | `BadRequest` | 400 | COMMON400 |
| 잘못된 category 값 | `RetroCategoryInvalid` | 400 | RETRO4004 |
| cursor < 1 | `BadRequest` | 400 | COMMON400 |
| size 범위 초과 | `BadRequest` | 400 | COMMON400 |
| JWT 인증 실패 | `Unauthorized` | 401 | AUTH001 |
| 회고 미존재/권한 없음 | `NotFound` | 404 | TEAM4031 |
| DB 조회 오류 | `InternalError` | 500 | COMMON500 |

### category 에러와 일반 BadRequest의 구분

```rust
// 일반 요청 오류 (COMMON400)
AppError::BadRequest("size는 1~100 범위의 정수여야 합니다.".to_string())

// 카테고리 특정 에러 (RETRO4004)
AppError::RetroCategoryInvalid("유효하지 않은 카테고리 값입니다.".to_string())
```

category 검증만 별도 에러 코드(RETRO4004)를 사용하는 이유는, 클라이언트가 "카테고리 값이 잘못됨"을 일반 요청 오류와 구분하여 처리할 수 있도록 하기 위해서다.

---

## 9) Swagger 문서 생성

**파일**: `handler.rs:549-568`

```rust
#[utoipa::path(
    get,
    path = "/api/v1/retrospects/{retrospectId}/responses",
    params(
        ("retrospectId" = i64, Path, description = "조회를 진행할 회고 세션 고유 ID"),
        ResponsesQueryParams      // IntoParams derive한 타입
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, body = SuccessResponsesListResponse),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    ),
    tag = "Retrospect"
)]
```

```java
// Spring 대응 (springdoc-openapi)
@Operation(summary = "회고 답변 카테고리별 조회",
           security = @SecurityRequirement(name = "bearer_auth"))
@ApiResponses({
    @ApiResponse(responseCode = "200",
        content = @Content(schema = @Schema(implementation = ResponsesListResponse.class))),
    @ApiResponse(responseCode = "400", content = @Content(schema = @Schema(implementation = ErrorResponse.class)))
})
@Parameters({
    @Parameter(name = "retrospectId", in = ParameterIn.PATH),
    @Parameter(name = "category", in = ParameterIn.QUERY),
    @Parameter(name = "cursor", in = ParameterIn.QUERY),
    @Parameter(name = "size", in = ParameterIn.QUERY)
})
```

`params()`에 `ResponsesQueryParams`를 직접 전달하면 `IntoParams` derive 덕분에 Swagger UI에 각 쿼리 파라미터가 자동으로 표시된다.

---

## 10) Offset vs Cursor 페이지네이션 비교 (실무 관점)

이 API가 Offset이 아닌 Cursor 방식을 선택한 이유를 실무 관점에서 비교한다.

### Offset 방식
```sql
SELECT * FROM response WHERE retrospect_id = 42
ORDER BY response_id DESC
LIMIT 10 OFFSET 990;
-- 실제로는 1000행을 읽고 990행을 버림
```

### Cursor 방식 (이 API)
```sql
SELECT * FROM response WHERE retrospect_id = 42
  AND response_id < 500  -- cursor
ORDER BY response_id DESC
LIMIT 11;  -- size + 1
-- response_id 인덱스를 타고 바로 접근
```

| 비교 항목 | Offset | Cursor |
|----------|--------|--------|
| 100페이지 조회 시 | 앞의 990행 스캔 후 버림 | 인덱스로 바로 이동 |
| 실시간 데이터 추가 시 | 중복/누락 발생 가능 | 커서 기준이므로 안전 |
| 임의 페이지 이동 | 가능 (OFFSET 계산) | 불가능 (순차 탐색만) |
| 적합한 UI | 페이지 번호 네비게이션 | 무한 스크롤, "더 보기" |

이 API는 회고 답변을 무한 스크롤로 보여주므로 Cursor 방식이 적합하다.

---

## 11) 이해 체크리스트

다음 질문에 답할 수 있으면 API-020을 이해한 것이다.

### 기본 흐름
- [ ] `category`를 enum이 아닌 String으로 받는 이유는?
- [ ] `Path`와 `Query` extractor의 차이는?
- [ ] `FromStr` trait을 구현하면 어떤 메서드를 사용할 수 있는가?

### 서비스 로직
- [ ] `find_retrospect_for_member`가 하는 역할은? 왜 404를 반환하는가?
- [ ] 질문 순서를 왜 첫 번째 멤버의 응답 기준으로 결정하는가?
- [ ] `size + 1` 조회 트릭이 왜 필요한가? COUNT 쿼리와 비교하면?
- [ ] `has_next`와 `next_cursor`는 어떻게 계산되는가?
- [ ] 좋아요/댓글 수는 어떤 SQL 패턴으로 집계되는가?

### Rust 문법
- [ ] `and_then`과 `map`의 차이는? (Option 관점에서)
- [ ] `(1..=100).contains(&size)`에서 `1..=100`의 타입은?
- [ ] `.take(size as usize)`는 Java의 어떤 메서드에 대응하는가?
- [ ] `HashSet`으로 중복을 제거한 후 다시 `Vec`으로 변환하는 이유는?

### 설계 결정
- [ ] Offset 대신 Cursor 방식을 선택한 이유는?
- [ ] 빈 답변(공백만)을 필터링하는 이유는?
- [ ] API-019와 API-020이 카테고리/필터 파싱에서 다른 전략을 쓰는 이유는?

---

## 12) 용어 대응표 (빠른 참조)

| Rust 용어 | Spring/Java 대응 | 이 API에서의 사용 |
|-----------|-----------------|-----------------|
| `FromStr` trait | `valueOf()` | category 문자열 -> enum 변환 |
| `parse()` | `valueOf()` 호출 | `params.category.parse()` |
| `and_then()` | `flatMap()` (Optional) | 닉네임 체이닝 |
| `unwrap_or(10)` | `orElse(10)` | size 기본값 |
| `take(n)` | `limit(n)` (Stream) | 페이지 크기 제한 |
| `(1..=100)` | `IntStream.rangeClosed(1,100)` | size 범위 검증 |
| `HashSet` | `HashSet` | member_id 중복 제거 |
| `HashMap` | `HashMap` | ID 기반 매핑 (6곳) |
| `into_tuple()` | 없음 (Object[] 사용) | 집계 결과 수신 |
| `select_only()` | `@Query` 커스텀 | 커스텀 SELECT |
| `group_by()` | `GROUP BY` | 집계 쿼리 |
| `copied()` | 없음 (autoboxing) | `&i64` -> `i64` 변환 |
| `filter_map()` | `map().filter()` | 변환 + 필터 동시 |

---

함께 보면 좋은 문서:
- `flow.md`: 단계별 동작 흐름도
- `key-concepts.md`: 커서 기반 페이지네이션, FromStr, size+1 등 핵심 개념 심화
- `keywords.md`: 개별 키워드별 학습 자료 링크
