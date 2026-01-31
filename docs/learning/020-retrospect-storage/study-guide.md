# 학습 가이드: API-019 보관함 조회

이 문서는 **JVM 기반 Spring만 익숙한 개발자**가 API-019의 모든 코드를 읽고 이해할 수 있도록 작성한 학습 가이드입니다. Rust 문법, Axum 프레임워크, SeaORM을 **Spring 개념에 1:1 매핑**해서 설명합니다.

---

## 1) 한눈에 보는 기술 매핑 (Spring ↔ Rust)

| Spring (JVM) | 이 프로젝트 (Rust) | 파일 위치 |
|---|---|---|
| `@RestController` | `handler.rs`의 함수 | `domain/retrospect/handler.rs:368` |
| `@Service` | `service.rs`의 함수 | `domain/retrospect/service.rs:688` |
| `JpaRepository` / QueryDSL | SeaORM `Entity::find()` | `domain/retrospect/entity/` |
| `@RequestParam` | `Query<StorageQueryParams>` | `handler.rs:371` |
| `@PathVariable` | `Path<i64>` | (이 API에서는 미사용) |
| `@RequestBody` | `Json<T>` | (이 API는 GET이라 미사용) |
| `ResponseEntity<T>` | `Json<BaseResponse<T>>` | `utils/response.rs` |
| `@ControllerAdvice` | `AppError` + `IntoResponse` | `utils/error.rs` |
| Spring Security Filter | `AuthUser` extractor | `utils/auth.rs` |
| `@Autowired` / DI | `State(state): State<AppState>` | `handler.rs:370` |
| `enum` (Java) | `enum` (Rust) | `dto.rs:277` |
| `@JsonProperty` | `#[serde(rename)]` | `dto.rs:280` |
| Swagger `@Schema` | `ToSchema` derive | `dto.rs:277` |
| Swagger `@Parameter` | `IntoParams` derive | `dto.rs:318` |
| `HashMap` (Java) | `HashMap` / `BTreeMap` | `service.rs:742, 748` |
| `Stream API` | Iterator + `collect()` | `service.rs:726, 788` |
| `Optional<T>` | `Option<T>` | `dto.rs:322` |
| `throws Exception` | `Result<T, AppError>` | 거의 모든 함수 |

---

## 2) API-019 기능 요약

- **엔드포인트**: `GET /api/v1/retrospects/storage?range=3_MONTHS`
- **기능**: 사용자가 제출 완료한 회고를 연도별로 그룹화하여 조회
- **인증**: JWT Bearer 토큰 필수
- **필터**: `ALL`, `3_MONTHS`, `6_MONTHS`, `1_YEAR` (기본값: `ALL`)

**Spring으로 작성했다면 이런 느낌:**

```java
@GetMapping("/api/v1/retrospects/storage")
public ResponseEntity<BaseResponse<StorageResponse>> getStorage(
        @AuthenticationPrincipal UserDetails user,
        @RequestParam(defaultValue = "ALL") StorageRangeFilter range) {
    StorageResponse result = retrospectService.getStorage(user.getId(), range);
    return ResponseEntity.ok(BaseResponse.success(result));
}
```

이것의 **Rust/Axum 버전**이 이 API의 코드이다.

---

## 3) 요청 흐름 (Spring 기준으로 이해하기)

```
클라이언트
  GET /api/v1/retrospects/storage?range=3_MONTHS
  Authorization: Bearer {token}
        |
        v
  [1] AuthUser Extractor          ← Spring Security Filter 역할
        JWT 토큰 검증, user_id 추출
        |
        v
  [2] Handler (get_storage)       ← @RestController 메서드 역할
        파라미터 파싱, 서비스 호출
        |
        v
  [3] Service (get_storage)       ← @Service 메서드 역할
        DB 조회, 비즈니스 로직
        |
        v
  [4] SeaORM Entity::find()       ← JpaRepository 역할
        SQL 생성/실행
        |
        v
  [5] BaseResponse 래핑           ← ResponseEntity 역할
        JSON 직렬화
```

### 단계별 소스 파일 위치

| 단계 | Spring 대응 | 파일 |
|------|------------|------|
| JWT 인증 | Security Filter | `src/utils/auth.rs` |
| 핸들러 | Controller | `src/domain/retrospect/handler.rs:368-383` |
| 서비스 | Service | `src/domain/retrospect/service.rs:688-805` |
| Entity/ORM | Repository + Entity | `src/domain/retrospect/entity/retrospect.rs` |
| DTO | DTO/VO | `src/domain/retrospect/dto.rs:272-367` |
| 응답 래핑 | ResponseEntity | `src/utils/response.rs` |
| 에러 처리 | @ControllerAdvice | `src/utils/error.rs` |

---

## 4) Rust 문법 핵심 (이 API를 읽기 위한 최소한의 지식)

### 4-1. `Result<T, E>` = Java의 checked exception

Rust에는 `try-catch`가 없다. 대신 **모든 에러를 `Result` 타입으로 반환**한다.

```rust
// Rust
fn get_storage(...) -> Result<StorageResponse, AppError> {
    // 성공: Ok(값)
    // 실패: Err(에러)
}
```

```java
// Spring 대응
StorageResponse getStorage(...) throws AppException {
    // 성공: return 값;
    // 실패: throw new AppException(...);
}
```

### 4-2. `?` 연산자 = Java의 `throw`

`?`는 **"에러면 즉시 리턴, 성공이면 값 추출"** 하는 연산자이다.

```rust
// Rust
let member_retros = member_retro_query
    .all(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
//                                                      ↑ 에러면 여기서 Err(AppError) 반환
```

```java
// Spring 대응
List<MemberRetro> memberRetros;
try {
    memberRetros = memberRetroRepository.findAll(spec);
} catch (Exception e) {
    throw new AppException(ErrorCode.INTERNAL_ERROR, e.getMessage());
}
```

`?` 하나가 try-catch + throw를 대체한다.

### 4-3. `Option<T>` = Java의 `Optional<T>`

```rust
// Rust
pub range: Option<StorageRangeFilter>  // 있을 수도, 없을 수도

// None이면 기본값 사용
let range_filter = params.range.unwrap_or_default();
```

```java
// Spring 대응
Optional<StorageRangeFilter> range;

StorageRangeFilter rangeFilter = range.orElse(StorageRangeFilter.ALL);
```

### 4-4. `match` = Java의 `switch` (강화 버전)

```rust
// Rust
match self {
    StorageRangeFilter::All => None,
    StorageRangeFilter::ThreeMonths => Some(90),
    StorageRangeFilter::SixMonths => Some(180),
    StorageRangeFilter::OneYear => Some(365),
}
```

```java
// Java 대응
switch (this) {
    case ALL -> null;
    case THREE_MONTHS -> 90;
    case SIX_MONTHS -> 180;
    case ONE_YEAR -> 365;
}
```

Rust의 `match`는 **모든 케이스를 반드시 처리**해야 컴파일된다. 빠뜨리면 컴파일 에러.

### 4-5. `if let Some(x) = ...` = Optional 패턴 매칭

```rust
// Rust
if let Some(days) = range_filter.days() {
    // days가 있는 경우에만 이 블록 실행
    let cutoff = Utc::now().naive_utc() - Duration::days(days);
    // ...
}
// None이면 이 블록을 건너뜀 (필터 없이 전체 조회)
```

```java
// Spring 대응
Integer days = rangeFilter.getDays();  // null 가능
if (days != null) {
    LocalDateTime cutoff = LocalDateTime.now().minusDays(days);
    // ...
}
```

### 4-6. `async/await` = Java의 `CompletableFuture` (간소화)

Rust의 비동기는 Spring WebFlux의 Mono/Flux와 개념이 비슷하지만, **문법은 동기 코드처럼** 쓸 수 있다.

```rust
// Rust - 비동기 함수
pub async fn get_storage(...) -> Result<StorageResponse, AppError> {
    let retros = entity::find()
        .all(&state.db)
        .await?;  // DB 호출을 기다림
}
```

```java
// Spring (동기)
public StorageResponse getStorage(...) {
    List<Retrospect> retros = repository.findAll(spec);
}
```

Axum은 내부적으로 Tokio 런타임을 사용한다. Spring Boot가 Tomcat 스레드풀을 쓰는 것처럼, Axum은 Tokio 태스크풀을 사용한다.

### 4-7. 소유권(Ownership) — Rust만의 개념

Java에서는 모든 객체가 힙에 있고 GC가 관리한다. **Rust에는 GC가 없다.** 대신 **소유권** 시스템으로 메모리를 관리한다.

```rust
let retrospects = vec![retro1, retro2, retro3];

// into_iter()는 소유권을 "이동"시킴 (이후 retrospects 사용 불가)
let items: Vec<Item> = retrospects.into_iter()
    .map(|r| r.into())
    .collect();
// 여기서 retrospects를 다시 쓰면 컴파일 에러

// iter()는 "빌림" (이후에도 retrospects 사용 가능)
for r in &retrospects { ... }  // 빌려서 읽기만
```

```java
// Java에서는 이런 구분이 없다
List<Retrospect> retrospects = List.of(retro1, retro2, retro3);
// 언제든 다시 사용 가능 (GC가 관리)
retrospects.stream().map(r -> ...).collect(toList());
retrospects.forEach(r -> ...); // 문제 없음
```

이 API에서 소유권이 등장하는 곳:
- `retrospect_ids.clone()` (service.rs:730, 737): 같은 데이터를 두 번 사용해야 해서 복사
- `year_groups.into_iter()` (service.rs:789): BTreeMap을 소비하며 순회

### 4-8. `clone()` — 값 복사

```rust
// Rust - 소유권 때문에 같은 값을 두 곳에서 쓰려면 clone 필요
let retrospect_ids: Vec<i64> = member_retros.iter().map(|mr| mr.retrospect_id).collect();

// 첫 번째 사용
retrospect::Entity::find()
    .filter(Column::RetrospectId.is_in(retrospect_ids.clone()))  // clone: 복사본 전달
    ...

// 두 번째 사용
member_retro::Entity::find()
    .filter(Column::RetrospectId.is_in(retrospect_ids.clone()))  // 또 clone
    ...
```

```java
// Java에서는 clone 없이 같은 List를 여러 번 전달해도 됨
List<Long> retrospectIds = memberRetros.stream()
    .map(MemberRetro::getRetrospectId).collect(toList());

retrospectRepo.findAllByIdIn(retrospectIds);  // 그냥 전달
memberRetroRepo.findAllByRetrospectIdIn(retrospectIds);  // 다시 전달 (문제 없음)
```

---

## 5) Handler 코드 상세 (Spring Controller 역할)

**파일**: `handler.rs:368-383`

```rust
pub async fn get_storage(
    user: AuthUser,                              // [1] JWT 인증
    State(state): State<AppState>,               // [2] DI (의존성 주입)
    Query(params): Query<StorageQueryParams>,     // [3] 쿼리 파라미터
) -> Result<Json<BaseResponse<StorageResponse>>, AppError> {  // [4] 반환 타입
    let user_id = user.user_id()?;               // [5] 사용자 ID 추출
    let result = RetrospectService::get_storage(state, user_id, params).await?;  // [6] 서비스 호출
    Ok(Json(BaseResponse::success_with_message(result, "보관함 조회를 성공했습니다.")))  // [7] 응답
}
```

### [1] `user: AuthUser` — Spring Security 역할

함수 파라미터에 `AuthUser`를 넣는 것만으로 JWT 인증이 자동 수행된다. Axum은 `FromRequestParts` trait을 구현한 타입을 자동으로 요청에서 추출한다.

```java
// Spring 대응
@GetMapping("/storage")
public ResponseEntity<?> getStorage(
        @AuthenticationPrincipal UserPrincipal user,  // Security Filter가 자동 주입
        ...
)
```

인증 실패 시 `AuthUser` 추출이 실패하여 `401 Unauthorized`가 자동 반환된다. 핸들러 코드에 인증 로직이 전혀 없는 이유이다.

### [2] `State(state): State<AppState>` — 의존성 주입

`AppState`는 DB 커넥션 풀, 설정 등을 가진 전역 상태이다.

```java
// Spring 대응
@Autowired
private DataSource dataSource;

@Autowired
private AppConfig appConfig;
```

Spring은 `@Autowired`로 빈을 주입하고, Axum은 `State` extractor로 전역 상태를 주입한다. Spring이 IoC 컨테이너에 빈을 등록하는 것처럼, Axum은 `Router::with_state(state)`로 상태를 등록한다.

### [3] `Query(params): Query<StorageQueryParams>` — @RequestParam

URL의 `?range=3_MONTHS` 부분을 자동으로 구조체로 파싱한다.

```java
// Spring 대응
@RequestParam(required = false, defaultValue = "ALL")
StorageRangeFilter range
```

차이점: Spring은 `@RequestParam`으로 **개별 파라미터**를 받지만, Axum의 `Query`는 **구조체 전체**로 한번에 역직렬화한다.

```rust
// Axum - 쿼리 파라미터를 구조체로 한번에 역직렬화
#[derive(Deserialize)]
pub struct StorageQueryParams {
    pub range: Option<StorageRangeFilter>,
    // 파라미터가 더 있으면 여기에 추가
}
```

```java
// Spring - 개별 파라미터 또는 DTO로
// 방법 1: 개별
@RequestParam StorageRangeFilter range

// 방법 2: DTO
StorageQueryParams params  // Spring도 커스텀 DTO 바인딩 가능
```

### [4] 반환 타입: `Result<Json<BaseResponse<StorageResponse>>, AppError>`

이 타입을 분해하면:

| 부분 | 의미 | Spring 대응 |
|------|------|------------|
| `Result<OK, ERR>` | 성공 또는 에러 | `try { ... } catch { ... }` |
| `Json<T>` | JSON 직렬화 | `@ResponseBody` |
| `BaseResponse<T>` | 표준 응답 래핑 | `ResponseEntity<BaseResponse<T>>` |
| `AppError` | 에러 타입 | Custom Exception |

### [5] `user.user_id()?` — Claims에서 ID 추출

`?`가 붙어있으므로: ID 추출 실패 시 자동으로 에러 응답이 반환된다.

### [6] 서비스 호출

```rust
let result = RetrospectService::get_storage(state, user_id, params).await?;
```

- `RetrospectService::get_storage(...)` → Spring의 `retrospectService.getStorage(...)`
- `.await` → 비동기 호출 대기
- `?` → 에러 시 자동 반환

### [7] 성공 응답 래핑

```rust
Ok(Json(BaseResponse::success_with_message(result, "보관함 조회를 성공했습니다.")))
```

```java
// Spring 대응
return ResponseEntity.ok(BaseResponse.success(result, "보관함 조회를 성공했습니다."));
```

---

## 6) DTO 코드 상세

**파일**: `dto.rs:272-367`

### 6-1. StorageRangeFilter enum (Java enum과 비교)

```rust
// Rust enum
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, ToSchema)]
pub enum StorageRangeFilter {
    #[serde(rename = "ALL")]
    #[default]
    All,
    #[serde(rename = "3_MONTHS")]
    ThreeMonths,
    #[serde(rename = "6_MONTHS")]
    SixMonths,
    #[serde(rename = "1_YEAR")]
    OneYear,
}
```

```java
// Java enum 대응
public enum StorageRangeFilter {
    @JsonProperty("ALL")
    ALL(null),

    @JsonProperty("3_MONTHS")
    THREE_MONTHS(90),

    @JsonProperty("6_MONTHS")
    SIX_MONTHS(180),

    @JsonProperty("1_YEAR")
    ONE_YEAR(365);

    private final Integer days;

    StorageRangeFilter(Integer days) { this.days = days; }
    public Integer getDays() { return days; }
}
```

#### `#[derive(...)]` = Java의 어노테이션 + lombok 결합

| Rust derive | 역할 | Java 대응 |
|-------------|------|----------|
| `Debug` | 디버그 출력 | `@ToString` (lombok) |
| `Default` | 기본값 제공 | 없음 (생성자로 처리) |
| `Clone` | 값 복사 가능 | `Cloneable` |
| `PartialEq, Eq` | `==` 비교 가능 | `equals()` |
| `Deserialize` | JSON → 구조체 변환 | Jackson `@JsonCreator` |
| `Serialize` | 구조체 → JSON 변환 | Jackson 기본 동작 |
| `ToSchema` | Swagger 스키마 생성 | `@Schema` (springdoc) |

#### `#[serde(rename = "3_MONTHS")]` = `@JsonProperty("3_MONTHS")`

Rust에서 변수/타입 이름은 숫자로 시작할 수 없다. `3Months`는 불가능하므로 `ThreeMonths`로 이름짓고, JSON에서는 `"3_MONTHS"`로 매핑한다.

#### `#[default]` = 기본값 지정

```rust
#[default]
All,  // StorageRangeFilter::default() → All
```

이것은 `unwrap_or_default()`와 조합되어, 쿼리 파라미터가 없을 때 `All`을 기본값으로 사용한다.

### 6-2. `days()` 메서드 — Option 반환 패턴

```rust
impl StorageRangeFilter {
    pub fn days(&self) -> Option<i64> {
        match self {
            StorageRangeFilter::All => None,          // 전체: 필터 없음
            StorageRangeFilter::ThreeMonths => Some(90),
            StorageRangeFilter::SixMonths => Some(180),
            StorageRangeFilter::OneYear => Some(365),
        }
    }
}
```

`None`은 "필터를 적용하지 않음"을 의미한다. `Some(90)`은 "90일 필터"를 의미한다.

```java
// Java 대응 (nullable Integer)
public Integer getDays() {
    return switch (this) {
        case ALL -> null;         // 필터 없음
        case THREE_MONTHS -> 90;
        case SIX_MONTHS -> 180;
        case ONE_YEAR -> 365;
    };
}
```

Rust의 `Option`은 **null이 존재하지 않는** 언어에서 "없을 수 있음"을 표현하는 타입이다. Java의 `Optional`과 같은 목적이지만, Rust에서는 모든 곳에서 사용하는 기본 방식이다.

### 6-3. Response DTO 구조체

```rust
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]      // ← 모든 필드를 camelCase로
pub struct StorageRetrospectItem {
    pub retrospect_id: i64,             // JSON: "retrospectId"
    pub display_date: String,           // JSON: "displayDate"
    pub title: String,                  // JSON: "title"
    pub retrospect_method: RetrospectMethod,  // JSON: "retrospectMethod"
    pub member_count: i64,              // JSON: "memberCount"
}
```

```java
// Java 대응
@Getter
public class StorageRetrospectItem {
    private Long retrospectId;
    private String displayDate;
    private String title;
    private RetrospectMethod retrospectMethod;
    private Long memberCount;
}
```

`#[serde(rename_all = "camelCase")]`는 **Rust의 `snake_case` 필드명을 JSON의 `camelCase`로 자동 변환**한다. Java는 기본이 camelCase라 이런 변환이 필요 없지만, Rust는 `snake_case`가 관례이므로 serde 어노테이션으로 매핑한다.

### 6-4. Swagger용 DTO

```rust
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessStorageResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: StorageResponse,
}
```

이 타입은 실제 비즈니스 로직에서는 사용되지 않고, **Swagger 문서 생성 전용**이다. `BaseResponse<T>`가 제네릭이라 Swagger(utoipa)가 자동으로 스키마를 생성하지 못하기 때문에, 구체적인 타입으로 별도 정의한다.

```java
// Spring 대응: springdoc은 제네릭 타입도 자동 처리하므로 별도 타입이 필요 없다.
// Rust의 utoipa는 제네릭을 지원하지 않아 이 우회가 필요하다.
```

---

## 7) Service 코드 상세 (비즈니스 로직)

**파일**: `service.rs:688-805`

### 7-1. 함수 시그니처

```rust
pub async fn get_storage(
    state: AppState,           // DB 커넥션 등 전역 상태
    user_id: i64,              // JWT에서 추출한 사용자 ID
    params: StorageQueryParams, // 쿼리 파라미터
) -> Result<StorageResponse, AppError> {
```

```java
// Spring 대응
public StorageResponse getStorage(Long userId, StorageQueryParams params)
        throws AppException {
```

### 7-2. 기본값 처리

```rust
let range_filter = params.range.unwrap_or_default();
```

| `params.range` 값 | 결과 |
|---|---|
| `Some(ThreeMonths)` | `ThreeMonths` |
| `None` (쿼리 파라미터 생략) | `All` (`#[default]` 지정) |

```java
// Spring 대응
StorageRangeFilter rangeFilter = Optional.ofNullable(params.getRange())
    .orElse(StorageRangeFilter.ALL);
```

### 7-3. SeaORM 쿼리 작성 (JPA/QueryDSL 대응)

```rust
let mut member_retro_query = member_retro::Entity::find()
    .filter(member_retro::Column::MemberId.eq(user_id))
    .filter(
        member_retro::Column::Status
            .is_in([RetrospectStatus::Submitted, RetrospectStatus::Analyzed]),
    );
```

```java
// Spring JPA 대응 (QueryDSL)
QMemberRetro mr = QMemberRetro.memberRetro;
BooleanBuilder builder = new BooleanBuilder()
    .and(mr.memberId.eq(userId))
    .and(mr.status.in(SUBMITTED, ANALYZED));
```

```sql
-- 실제 SQL
SELECT * FROM member_retro
WHERE member_id = ?
  AND status IN ('SUBMITTED', 'ANALYZED')
```

#### SeaORM ↔ JPA 메서드 대응표

| SeaORM | JPA/Spring Data | SQL |
|--------|----------------|-----|
| `Entity::find()` | `repository.findAll(spec)` | `SELECT *` |
| `Entity::find_by_id(id)` | `repository.findById(id)` | `WHERE id = ?` |
| `.filter(Col.eq(v))` | `.and(field.eq(v))` (QueryDSL) | `WHERE col = ?` |
| `.filter(Col.is_in([...]))` | `.and(field.in(...))` | `WHERE col IN (...)` |
| `.filter(Col.gte(v))` | `.and(field.goe(v))` | `WHERE col >= ?` |
| `.order_by_desc(Col)` | `Sort.by(DESC, "col")` | `ORDER BY col DESC` |
| `.all(&db).await` | `.findAll()` | 실행 (리스트) |
| `.one(&db).await` | `.findOne()` | 실행 (단건) |

### 7-4. 조건부 필터 추가 (동적 쿼리)

```rust
if let Some(days) = range_filter.days() {
    let cutoff = Utc::now().naive_utc() - chrono::Duration::days(days);
    member_retro_query =
        member_retro_query.filter(member_retro::Column::SubmittedAt.gte(cutoff));
}
```

`range_filter`가 `All`이면 `days()`는 `None`을 반환하고, 이 블록은 실행되지 않는다. 즉, 필터 조건이 추가되지 않아 **전체 기간 조회**가 된다.

```java
// Spring 대응 (동적 쿼리)
if (rangeFilter.getDays() != null) {
    LocalDateTime cutoff = LocalDateTime.now().minusDays(rangeFilter.getDays());
    builder.and(mr.submittedAt.goe(cutoff));
}
```

핵심 포인트: `mut`(mutable) 키워드. Rust에서 변수는 **기본이 불변**이다.

```rust
let mut member_retro_query = ...;  // mut: 이 변수는 나중에 수정할 수 있음
// ...
member_retro_query = member_retro_query.filter(...);  // 조건 추가 (재할당)
```

Java에서는 변수가 기본적으로 가변(mutable)이다. Rust에서는 변경할 변수에 명시적으로 `mut`을 붙여야 한다. `let`만 쓰면 `final`과 같다.

### 7-5. 쿼리 실행과 에러 처리

```rust
let member_retros = member_retro_query
    .all(&state.db)        // DB에서 전체 조회 실행
    .await                 // 비동기 대기
    .map_err(|e| AppError::InternalError(e.to_string()))?;  // 에러 변환 + 전파
```

**`.map_err(|e| ...)?`** 패턴을 분해하면:

1. `.all(&state.db).await` → `Result<Vec<Model>, DbErr>` 반환
2. `.map_err(|e| AppError::InternalError(...))` → `DbErr`를 `AppError`로 변환
3. `?` → `Err`면 즉시 함수를 빠져나감 (= Java의 `throw`)

```java
// Spring 대응
List<MemberRetro> memberRetros;
try {
    memberRetros = memberRetroRepository.findAll(spec);
} catch (DataAccessException e) {
    throw new InternalServerException(e.getMessage());
}
```

### 7-6. 조기 반환 (Early Return)

```rust
if member_retros.is_empty() {
    return Ok(StorageResponse { years: vec![] });
}
```

빈 결과면 즉시 빈 응답을 반환한다. 불필요한 추가 쿼리를 방지하는 최적화이다.

```java
// Spring 대응
if (memberRetros.isEmpty()) {
    return new StorageResponse(List.of());
}
```

### 7-7. Iterator 체이닝 (Java Stream 대응)

```rust
let retrospect_ids: Vec<i64> = member_retros.iter().map(|mr| mr.retrospect_id).collect();
```

```java
// Java Stream 대응
List<Long> retrospectIds = memberRetros.stream()
    .map(MemberRetro::getRetrospectId)
    .collect(Collectors.toList());
```

| Rust Iterator | Java Stream | 의미 |
|---|---|---|
| `.iter()` | `.stream()` | 이터레이터 생성 (빌림) |
| `.into_iter()` | `.stream()` (소비형) | 이터레이터 생성 (소유권 이동) |
| `.map(\|x\| ...)` | `.map(x -> ...)` | 각 요소 변환 |
| `.filter(\|x\| ...)` | `.filter(x -> ...)` | 조건 필터링 |
| `.filter_map(\|x\| ...)` | `.map().filter().map()` | 변환 + 필터 동시에 |
| `.collect()` | `.collect(toList())` | 결과 수집 |
| `.rev()` | 없음 (reverse 직접) | 역순 |
| `.for_each(\|x\| ...)` | `.forEach(x -> ...)` | 각 요소 처리 |

### 7-8. N+1 쿼리 방지 — HashMap 집계

```rust
// 배치 쿼리: 모든 회고의 참여자를 한번에 조회
let all_member_retros_for_count = member_retro::Entity::find()
    .filter(member_retro::Column::RetrospectId.is_in(retrospect_ids.clone()))
    .all(&state.db).await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

// 어플리케이션 레벨 집계
let mut member_counts: HashMap<i64, i64> = HashMap::new();
for mr in &all_member_retros_for_count {
    *member_counts.entry(mr.retrospect_id).or_insert(0) += 1;
}
```

```java
// Spring 대응
List<MemberRetro> allMemberRetros = memberRetroRepository
    .findAllByRetrospectIdIn(retrospectIds);

Map<Long, Long> memberCounts = allMemberRetros.stream()
    .collect(Collectors.groupingBy(
        MemberRetro::getRetrospectId,
        Collectors.counting()
    ));
```

#### Entry API 상세 설명

```rust
*member_counts.entry(mr.retrospect_id).or_insert(0) += 1;
```

이 한 줄을 풀어쓰면:

1. `entry(mr.retrospect_id)` — 키로 진입점 획득
2. `.or_insert(0)` — 키가 없으면 0으로 초기화, 있으면 기존 값의 **가변 참조** 반환
3. `*... += 1` — 해당 값에 1 더하기 (`*`는 참조를 역참조하여 실제 값에 접근)

```java
// Java 대응
memberCounts.merge(mr.getRetrospectId(), 1L, Long::sum);
```

### 7-9. BTreeMap으로 연도별 그룹화

```rust
let mut year_groups: BTreeMap<i32, Vec<StorageRetrospectItem>> = BTreeMap::new();

for retro in &retrospects {
    let year = /* 연도 추출 */;
    let item = StorageRetrospectItem { ... };
    year_groups.entry(year).or_default().push(item);
}
```

```java
// Java 대응
TreeMap<Integer, List<StorageRetrospectItem>> yearGroups = new TreeMap<>();

for (Retrospect retro : retrospects) {
    int year = /* 연도 추출 */;
    StorageRetrospectItem item = new StorageRetrospectItem(...);
    yearGroups.computeIfAbsent(year, k -> new ArrayList<>()).add(item);
}
```

#### BTreeMap = Java의 TreeMap

| Rust | Java | 특성 |
|------|------|------|
| `BTreeMap` | `TreeMap` | 키 정렬 보장, O(log n) |
| `HashMap` | `HashMap` | 순서 없음, O(1) |

이 API에서 `BTreeMap`을 쓰는 이유: **연도(key)가 자동 정렬**되므로 `.rev()`만 호출하면 내림차순(최신 연도 우선)이 된다.

### 7-10. 날짜 처리 (UTC → KST)

```rust
let kst_offset = chrono::Duration::hours(9);

let display_date = submitted_dates
    .get(&retro.retrospect_id)           // HashMap에서 submitted_at 조회
    .map(|dt| (*dt + kst_offset).format("%Y-%m-%d").to_string())  // KST 변환 + 포맷
    .unwrap_or_else(|| {                 // submitted_at이 없으면 created_at 사용
        (retro.created_at + kst_offset).format("%Y-%m-%d").to_string()
    });
```

```java
// Java 대응
String displayDate = Optional.ofNullable(submittedDates.get(retro.getRetrospectId()))
    .map(dt -> dt.plusHours(9).format(DateTimeFormatter.ofPattern("yyyy-MM-dd")))
    .orElseGet(() -> retro.getCreatedAt().plusHours(9)
        .format(DateTimeFormatter.ofPattern("yyyy-MM-dd")));
```

- DB에 UTC로 저장된 시각을 **+9시간**하여 KST(한국 시간)로 변환
- `NaiveDateTime`은 타임존 정보가 없는 날짜/시각 (Java의 `LocalDateTime`에 대응)

### 7-11. 최종 정렬 + 응답 변환

```rust
let mut years: Vec<StorageYearGroup> = year_groups
    .into_iter()     // BTreeMap 소비 (소유권 이동), 키 오름차순
    .rev()           // 역순 → 키 내림차순 (2026 → 2025 → ...)
    .map(|(year, mut items)| {
        items.sort_by(|a, b| b.display_date.cmp(&a.display_date));  // 그룹 내 최신순
        StorageYearGroup {
            year_label: format!("{}년", year),
            retrospects: items,
        }
    })
    .collect();
```

```java
// Java 대응
List<StorageYearGroup> years = yearGroups.descendingMap().entrySet().stream()
    .map(entry -> {
        List<StorageRetrospectItem> items = entry.getValue();
        items.sort(Comparator.comparing(
            StorageRetrospectItem::getDisplayDate).reversed());
        return new StorageYearGroup(entry.getKey() + "년", items);
    })
    .collect(Collectors.toList());
```

- `BTreeMap.into_iter().rev()` = Java의 `TreeMap.descendingMap()`
- `format!("{}년", year)` = Java의 `year + "년"` (String.format)

---

## 8) 에러 처리 (Spring @ControllerAdvice 역할)

**파일**: `error.rs`

### AppError enum = Custom Exception 클래스들을 하나의 enum으로 통합

```rust
pub enum AppError {
    BadRequest(String),        // 400
    InternalError(String),     // 500
    Unauthorized(String),      // 401
    NotFound(String),          // 404
    // ...
}
```

```java
// Spring 대응: 각각 별도 Exception 클래스
public class BadRequestException extends RuntimeException { ... }
public class NotFoundException extends RuntimeException { ... }
public class UnauthorizedException extends RuntimeException { ... }
```

Rust에서는 `enum` 하나로 모든 에러 종류를 표현한다. Java에서는 Exception 상속 계층을 만드는데, Rust enum은 **모든 variant를 한 파일에서 관리**한다.

### `IntoResponse` trait = @ExceptionHandler

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();     // HTTP 상태 코드
        let error_code = self.error_code();  // 비즈니스 에러 코드
        let message = self.message();        // 에러 메시지
        (status, Json(ErrorResponse::new(error_code, message))).into_response()
    }
}
```

```java
// Spring 대응
@ControllerAdvice
public class GlobalExceptionHandler {
    @ExceptionHandler(AppException.class)
    public ResponseEntity<ErrorResponse> handleAppException(AppException e) {
        return ResponseEntity
            .status(e.getStatus())
            .body(new ErrorResponse(e.getCode(), e.getMessage()));
    }
}
```

핵심 차이: Spring은 **별도의 ExceptionHandler 클래스**에서 처리하지만, Rust는 에러 타입 자체가 `IntoResponse`를 구현하여 **자동으로 HTTP 응답으로 변환**된다.

### `From` trait = 자동 에러 변환

```rust
impl From<QueryRejection> for AppError {
    fn from(rejection: QueryRejection) -> Self {
        AppError::BadRequest(rejection.to_string())
    }
}
```

이 구현 덕분에 `Query<StorageQueryParams>` 파싱이 실패하면 `QueryRejection` → `AppError::BadRequest`로 **자동 변환**되어 400 응답이 된다. Java에서 Spring이 `MethodArgumentTypeMismatchException`을 던지고 ExceptionHandler에서 잡는 것과 유사하다.

---

## 9) Swagger 문서 생성

**파일**: `handler.rs:353-367`

```rust
#[utoipa::path(
    get,                                                    // HTTP 메서드
    path = "/api/v1/retrospects/storage",                   // 경로
    params(StorageQueryParams),                             // 쿼리 파라미터 스키마
    security(("bearer_auth" = [])),                         // JWT 인증 필요
    responses(
        (status = 200, body = SuccessStorageResponse),       // 성공 응답 스키마
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    ),
    tag = "Retrospect"                                      // API 그룹
)]
```

```java
// Spring 대응 (springdoc-openapi)
@Operation(summary = "보관함 조회",
           security = @SecurityRequirement(name = "bearer_auth"))
@ApiResponses({
    @ApiResponse(responseCode = "200", content = @Content(schema = @Schema(implementation = StorageResponse.class))),
    @ApiResponse(responseCode = "400", content = @Content(schema = @Schema(implementation = ErrorResponse.class)))
})
@Parameters({
    @Parameter(name = "range", in = ParameterIn.QUERY, schema = @Schema(implementation = StorageRangeFilter.class))
})
```

- `utoipa` = Rust의 Swagger 라이브러리 (Spring의 `springdoc-openapi`에 대응)
- `ToSchema` derive = `@Schema` 어노테이션
- `IntoParams` derive = `@Parameter` 어노테이션

---

## 10) 이해 체크리스트

다음 질문에 답할 수 있으면 API-019를 이해한 것이다.

### 기본 흐름
- [ ] 요청이 Handler에 도달하기 전에 JWT 인증은 어디에서 수행되는가?
- [ ] `Query(params)`는 URL의 어떤 부분을 어떻게 파싱하는가?
- [ ] `range` 파라미터를 생략하면 기본값은 무엇이고, 어디에서 결정되는가?

### 서비스 로직
- [ ] 왜 3번의 DB 쿼리가 필요한가? (각각 무엇을 조회하는가?)
- [ ] N+1 문제를 어떻게 방지했는가?
- [ ] `BTreeMap`을 사용한 이유는? `HashMap`을 썼으면 어떤 추가 작업이 필요했을까?

### Rust 문법
- [ ] `?` 연산자는 어떤 역할을 하는가?
- [ ] `Option<T>`와 `Result<T, E>`의 차이는?
- [ ] `mut`은 왜 필요한가? (Java와 뭐가 다른가?)
- [ ] `.clone()`은 왜 필요한가? 소유권과 어떤 관계가 있는가?
- [ ] `into_iter()` vs `iter()`의 차이는?

### 에러 처리
- [ ] `AppError`는 Spring의 어떤 구조에 대응하는가?
- [ ] `IntoResponse` trait은 Spring의 어떤 메커니즘과 유사한가?
- [ ] 쿼리 파라미터에 잘못된 값(`?range=INVALID`)을 보내면 어떤 과정을 거쳐 400 응답이 되는가?

---

## 11) 용어 대응표 (빠른 참조)

| Rust 용어 | Spring/Java 대응 | 한줄 설명 |
|-----------|-----------------|----------|
| `trait` | `interface` | 동작을 정의하는 계약 |
| `impl` | `implements` / `class` 메서드 | trait 구현 또는 메서드 정의 |
| `derive` | `@Lombok` + `@JsonProperty` 등 | 컴파일 시 코드 자동 생성 |
| `enum` (Rust) | `enum` + `sealed class` | 가능한 상태를 타입으로 열거 |
| `struct` | `class` (DTO) | 데이터 구조체 |
| `Result<T, E>` | `try-catch` / throws | 성공 또는 실패를 표현 |
| `Option<T>` | `Optional<T>` | 값이 있거나 없거나 |
| `Vec<T>` | `List<T>` (ArrayList) | 가변 길이 배열 |
| `HashMap<K, V>` | `HashMap<K, V>` | 해시 기반 맵 |
| `BTreeMap<K, V>` | `TreeMap<K, V>` | 정렬된 맵 |
| `String` | `String` | 소유 문자열 |
| `&str` | `String` (읽기 전용 참조) | 빌린 문자열 슬라이스 |
| `i64` | `long` | 64비트 정수 |
| `bool` | `boolean` | 불리언 |
| `pub` | `public` | 공개 접근 |
| `mod` | `package` | 모듈 (파일/디렉토리 단위) |
| `use` | `import` | 모듈 가져오기 |
| `fn` | 메서드/함수 | 함수 정의 |
| `let` | `final var` | 불변 변수 선언 |
| `let mut` | `var` | 가변 변수 선언 |
| `&` | 참조 (자동) | 빌림 (borrowing) |
| `clone()` | 없음 (자동) | 값 깊은 복사 |
| `await` | `.get()` / `.block()` | 비동기 결과 대기 |
| crate | Maven artifact | 라이브러리 패키지 |
| `Cargo.toml` | `pom.xml` / `build.gradle` | 의존성/빌드 설정 |

---

함께 보면 좋은 문서:
- `flow.md`: 단계별 동작 흐름도
- `key-concepts.md`: 핵심 Rust/Axum/SeaORM 개념 심화
- `keywords.md`: 개별 키워드별 학습 자료 링크
