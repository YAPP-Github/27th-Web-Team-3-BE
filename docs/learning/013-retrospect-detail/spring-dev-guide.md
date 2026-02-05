# API-012 학습 가이드 (Spring 개발자용)

이 문서는 JVM/Spring 기반 개발자가 Rust/Axum/SeaORM으로 구현된 **API-012 회고 상세 조회**를 처음부터 끝까지 이해할 수 있도록 설명한다. Rust 문법, 프레임워크 개념, ORM 동작을 Spring 관점에 대응해 풀어쓴다.

---

## 1) 이 API가 하는 일 (한 문장 요약)

`GET /api/v1/retrospects/{retrospectId}` 요청을 받으면 **회고 상세 정보**(제목, 날짜, 방식, 참여 멤버, 질문 목록, 좋아요/댓글 집계)를 반환한다.

---

## 2) Spring ↔ Rust/Axum/SeaORM 대응표

| Spring 개념 | 이 프로젝트(Rust/Axum/SeaORM) | 파일 |
|---|---|---|
| Controller | Axum Handler 함수 | `codes/server/src/domain/retrospect/handler.rs` |
| Service | Service 메서드 | `codes/server/src/domain/retrospect/service.rs` |
| DTO | Serialize 가능한 struct | `codes/server/src/domain/retrospect/dto.rs` |
| Repository/JPA | SeaORM Entity + Query | `codes/server/src/domain/retrospect/entity/` |
| Security (@AuthenticationPrincipal) | `AuthUser` Extractor | `codes/server/src/utils/auth.rs` |
| 예외 처리 (@ControllerAdvice) | `AppError` + `IntoResponse` | `codes/server/src/utils/error.rs` |
| 공통 응답 포맷 | `BaseResponse<T>` | `codes/server/src/utils/response.rs` |
| ApplicationContext/Bean | `AppState` (DB, Config 보관) | `codes/server/src/state.rs` |

---

## 3) 실제 호출 흐름 (Controller → Service → DB)

**핸들러 (Handler)**  
`get_retrospect_detail`는 Spring의 `@GetMapping` 메서드와 동일한 역할을 한다.

1. Path Parameter 유효성 검사 (retrospectId >= 1)
2. JWT에서 `user_id` 추출 (`AuthUser` extractor)
3. Service 호출
4. 결과를 `BaseResponse`로 감싸서 JSON 반환

**서비스 (Service)**  
`RetrospectService::get_retrospect_detail`가 실제 비즈니스 로직을 수행한다.

1. `retrospects` 테이블에서 회고 존재 여부 확인  
2. `member_retro_room`에서 팀 접근 권한 확인  
3. 참여자 목록 조회 (`member_retro` → `member`)  
4. `response` 조회 후 질문 목록 추출
5. `response_like`, `response_comment` 집계
6. 응답 DTO 구성 (`chrono::NaiveDateTime::format()` 으로 날짜 문자열 변환 포함)

> 전체 흐름 도식과 상세 코드는 `flow.md`에 이미 정리되어 있다. 이 문서는 **Spring 개발자가 이해하기 어려운 부분**을 집중적으로 해설한다.

---

## 4) Rust 문법 빠른 이해 (이 API에서 실제로 쓰는 것만)

### 4-1. `async fn` + `.await`
- Spring의 `@Async`나 `CompletableFuture`와 달리, Rust에서는 `async fn`이 기본이다.
- DB 쿼리는 비동기로 실행되며 `.await`로 결과를 기다린다.

### 4-2. `Result<T, E>`와 `?` 연산자
- `Result`는 성공(`Ok`) 또는 실패(`Err`)를 담는다.
- `?`는 **에러가 발생하면 즉시 함수 종료**를 의미한다. (Spring의 `throw`와 유사)
  
```rust
let retrospect_model = retrospect::Entity::find_by_id(retrospect_id)
    .one(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?
    .ok_or_else(|| AppError::RetrospectNotFound("존재하지 않는 회고입니다.".to_string()))?;
```

### 4-3. `Option<T>` (nullable 대신)
- Java의 `null` 대신 Rust는 `Option<T>`를 사용한다.
- `Some(value)` 또는 `None`으로 표현된다.

```rust
let nickname = m.nickname
    .clone()
    .filter(|s| !s.is_empty())
    .unwrap_or_else(|| "Unknown".to_string());
```

### 4-4. 컬렉션 & Iterator
- `Vec<T>` = Java `List<T>`
- `HashMap` = Java `Map`
- `HashSet` = 중복 제거용 Set
- `iter().map().filter().collect()`는 스트림(Stream)과 비슷하다.

---

## 5) Axum (Spring MVC와 비교)

### 5-1. Handler 시그니처 = 요청 파싱

```rust
pub async fn get_retrospect_detail(
    user: AuthUser,               // 인증 정보 추출 (JWT)
    State(state): State<AppState>, // DB 등 상태 주입
    Path(retrospect_id): Path<i64> // @PathVariable
) -> Result<Json<BaseResponse<RetrospectDetailResponse>>, AppError>
```

Spring에서 보면:
- `AuthUser` ≈ `@AuthenticationPrincipal`
- `State(AppState)` ≈ `@Autowired` 주입
- `Path(i64)` ≈ `@PathVariable Long`

### 5-2. 응답 래핑
모든 성공 응답은 `BaseResponse<T>`로 감싸서 반환한다.  
Spring의 공통 응답 DTO 패턴과 동일하다.

---

## 6) SeaORM (JPA/Hibernate와 비교)

| JPA/Hibernate | SeaORM |
|---|---|
| `Repository.findById()` | `Entity::find_by_id().one()` |
| `findAll()` | `Entity::find().all()` |
| `where` 조건 | `.filter(Column::X.eq(value))` |
| `IN` 쿼리 | `.filter(Column::X.is_in(vec))` |
| `count()` | `.count()` |
| `@Entity` | `DeriveEntityModel` |

**중요 차이점**  
SeaORM은 기본적으로 **lazy loading 없이 명시적인 쿼리**를 사용한다.  
즉, **JOIN 대신 쿼리 두 번 + 메모리 조인** 패턴이 자주 쓰인다.

---

## 7) 이 API에서 실제 사용하는 쿼리 패턴

### 7-1. 존재 여부 확인 (단건 조회)
```rust
retrospect::Entity::find_by_id(retrospect_id).one(&state.db)
```
SQL로 보면:  
`SELECT * FROM retrospects WHERE retrospect_id = ?`

### 7-2. 접근 권한 확인
```rust
member_retro_room::Entity::find()
    .filter(MemberId.eq(user_id))
    .filter(RetrospectRoomId.eq(retrospect_room_id))
    .one(&state.db)
```
SQL로 보면:  
`SELECT * FROM member_retro_room WHERE member_id = ? AND retrospect_room_id = ?`

### 7-3. 참여자 목록 (메모리 조인)
1. `member_retro`에서 참여자 목록 조회 (정렬 유지)  
2. `member`에서 닉네임 조회 (IN 쿼리)  
3. `HashMap`으로 `member_id -> nickname` 매핑  
4. 원래 순서 유지하면서 DTO 생성

> 이 방식은 JPA의 `@OneToMany` + `JOIN FETCH`를 수동으로 구현한 것과 같다.

### 7-4. 질문 목록 추출
`response` 테이블에 질문 텍스트가 저장되어 있어, 중복 제거 후 최대 5개 추출한다.

```rust
let mut seen = HashSet::new();
let questions = responses.iter()
    .filter(|r| seen.insert(r.question.clone()))
    .take(5)
    .enumerate()
    .map(|(i, r)| RetrospectQuestionItem { index: (i + 1) as i32, content: r.question.clone() })
    .collect();
```

### 7-5. 좋아요/댓글 합계
```rust
response_like::Entity::find()
    .filter(ResponseId.is_in(response_ids))
    .count(&state.db)
```
SQL로 보면:  
`SELECT COUNT(*) FROM response_like WHERE response_id IN (...)`

---

## 8) 날짜/시간 처리 (chrono vs Java)

이 API에서 가장 주의 깊게 봐야 하는 부분 중 하나가 날짜/시간 처리이다.

### 8-1. chrono 크레이트 = Java의 java.time 패키지

| Java (java.time) | Rust (chrono) | 설명 |
|---|---|---|
| `LocalDate` | `NaiveDate` | 날짜만 (타임존 없음) |
| `LocalTime` | `NaiveTime` | 시간만 (타임존 없음) |
| `LocalDateTime` | `NaiveDateTime` | 날짜+시간 (타임존 없음) |
| `Instant` / `ZonedDateTime` | `DateTime<Utc>` | UTC 기준 날짜+시간 |
| `DateTimeFormatter` | `.format()` 메서드 | strftime 스타일 포맷팅 |

### 8-2. SeaORM의 DateTime = chrono::NaiveDateTime

SeaORM 엔티티에서 `DateTime` 타입으로 선언된 필드는 실제로 `chrono::NaiveDateTime`이다.

```rust
// entity/retrospect.rs
pub start_time: DateTime,  // = chrono::NaiveDateTime
```

Spring/JPA에서 보면 다음과 비슷하다:
```java
@Column(name = "start_time")
private LocalDateTime startTime;  // 타임존 없는 날짜+시간
```

### 8-3. 이 프로젝트의 타임존 전략

이 프로젝트는 `NaiveDateTime`에 **KST(한국 시간) 값을 직접 저장**하는 방식을 사용한다. 현재 시간과 비교할 때만 UTC에 9시간을 더해 KST로 변환한다.

```rust
// 회고 생성 시: KST 날짜/시간을 NaiveDateTime으로 조합 (service.rs:120)
let start_time = NaiveDateTime::new(retrospect_date, retrospect_time);

// 현재 시간 비교 시: UTC + 9시간 = KST (service.rs:261)
let now_kst = Utc::now().naive_utc() + chrono::Duration::hours(9);
```

Spring에서 보면 `LocalDateTime`에 KST 값을 저장하고, 현재 시간을 `ZoneId.of("Asia/Seoul")`로 구하는 것과 같다.

### 8-4. 상세 조회 시 날짜 포맷팅

```rust
// NaiveDateTime → "YYYY-MM-DD" 문자열 변환 (service.rs:938)
let start_time = retrospect_model.start_time.format("%Y-%m-%d").to_string();
```

Spring에서 보면:
```java
String startTime = retrospect.getStartTime()
    .format(DateTimeFormatter.ofPattern("yyyy-MM-dd"));
```

`NaiveDateTime::format()`은 strftime 스타일의 포맷 지시자를 사용한다 (`%Y`, `%m`, `%d`, `%H`, `%M`). DTO의 `start_time` 필드는 `String` 타입이므로, 이 변환 결과를 그대로 담아 JSON에 `"2026-01-24"` 같은 형태로 직렬화된다.

---

## 9) 응답 DTO 구조 (JSON 변환 규칙)

```rust
#[serde(rename_all = "camelCase")]
pub struct RetrospectDetailResponse {
    pub team_id: i64,
    pub title: String,
    pub start_time: String,
    pub retro_category: RetrospectMethod,
    pub members: Vec<RetrospectMemberItem>,
    pub total_like_count: i64,
    pub total_comment_count: i64,
    pub questions: Vec<RetrospectQuestionItem>,
}
```

- `snake_case` 필드명이 JSON에서는 `camelCase`로 변환된다.
- `retroCategory` 값은 `RetrospectMethod` enum이 `KPT`, `FOUR_L` 등으로 직렬화된다.

---

## 10) 에러 처리 이해 (Spring의 ExceptionHandler와 동일)

`AppError`는 Spring의 `@ControllerAdvice` + `@ExceptionHandler` 역할과 같다.  
예를 들어:

- `retrospectId < 1` → `AppError::BadRequest` (400)
- 회고 없음 → `AppError::RetrospectNotFound` (404)
- 팀 멤버 아님 → `AppError::TeamAccessDenied` (403)

각 에러는 `IntoResponse` 구현을 통해 공통 에러 응답 형태로 변환된다.

---

## 11) 이해를 위한 추천 읽기 순서

1. `docs/api-specs/012-retrospect-detail.md` (요구사항 확인)
2. `codes/server/src/domain/retrospect/handler.rs` (핸들러)
3. `codes/server/src/domain/retrospect/service.rs` (비즈니스 로직)
4. `codes/server/src/domain/retrospect/dto.rs` (응답 DTO)
5. `docs/learning/012-retrospect-detail/flow.md` (흐름 정리)
6. `docs/learning/012-retrospect-detail/key-concepts.md` (Rust/SeaORM 패턴)

---

## 12) 한 번 더 요약 (핵심만)

- Spring Controller → Axum Handler (`get_retrospect_detail`)
- Spring Service → `RetrospectService::get_retrospect_detail`
- JPA Repository → SeaORM Entity + Query
- `Result`/`Option`은 예외와 null을 대체하는 Rust 방식
- `java.time.LocalDateTime` → `chrono::NaiveDateTime` (타임존 없는 날짜+시간)
- `DateTimeFormatter` → `NaiveDateTime::format()` (strftime 스타일 포맷)
- 이 API는 6개 테이블을 조회하고, 질문/좋아요/댓글을 계산하며, 날짜를 포맷해 응답한다
