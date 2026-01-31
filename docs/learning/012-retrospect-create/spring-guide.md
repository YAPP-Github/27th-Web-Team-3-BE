# Spring 개발자를 위한 API-011 (회고 생성) 분석 가이드

이 문서는 Spring Framework(Java/Kotlin)에 익숙하지만 Rust가 처음인 개발자를 위해 작성되었습니다. API-011(회고 생성) 코드를 예제로 사용하여 Rust의 웹 개발 패턴을 Spring의 개념과 1:1로 매핑하여 설명합니다.

---

## 1. 전체 아키텍처 매핑

Spring의 **Controller-Service-Repository** 3계층 아키텍처는 Rust(Axum + SeaORM)에서도 거의 동일하게 유지됩니다. 다만 구현 방식과 용어에 차이가 있습니다.

| Spring Concept | Rust (이 프로젝트) | 설명 |
|----------------|-------------------|------|
| **Framework** | Spring Boot | `Axum` + `Tokio` |
| **Controller** | `@RestController` | `Handler` (함수) |
| **Service** | `@Service` | `Service` (Struct + impl) |
| **DTO** | `record` / `class` | `struct` (+ `Serde`) |
| **Validation** | Bean Validation (`@Valid`) | `validator` 크레이트 |
| **ORM** | JPA / Hibernate | `SeaORM` |
| **Transaction** | `@Transactional` | `txn` 객체 전달 (RAII) |
| **Dependency Injection** | `@Autowired` | `State<AppState>` (Extractor) |

---

## 2. 코드 레벨 상세 비교

### 2.1. DTO와 Validation

Spring에서는 DTO에 Annotation을 붙여 검증하지만, Rust에서는 `validator` 매크로를 사용합니다.

**Spring (Java)**
```java
public record CreateRetrospectRequest(
    @Min(1) Long teamId,
    @Size(min=1, max=20) String projectName,
    @Size(max=10) List<String> referenceUrls
) {}
```

**Rust (`dto.rs`)**
```rust
#[derive(Debug, Deserialize, Validate)] // 1. Deserialize: JSON 파싱, Validate: 검증 기능 추가
#[serde(rename_all = "camelCase")]      // 2. JSON 필드명(camelCase) <-> Rust 필드명(snake_case) 자동 변환
pub struct CreateRetrospectRequest {
    #[validate(range(min = 1))]         // 3. @Min(1)과 동일
    pub team_id: i64,

    #[validate(length(min = 1, max = 20))] // 4. @Size(min=1, max=20)과 동일
    pub project_name: String,

    #[validate(custom(function = "validate_reference_url_items"))] // 5. 커스텀 검증 함수
    pub reference_urls: Vec<String>,
}
```
> **핵심 차이**: Rust는 런타임 리플렉션 대신 **컴파일 타임 매크로**(`derive`)를 사용하여 코드를 생성합니다.

### 2.2. Controller (Handler)

Spring의 `@RestController` 메서드는 Rust에서 비동기 함수(`async fn`)인 **Handler**가 됩니다.

**Spring (Java)**
```java
@PostMapping("/api/v1/retrospects")
public ResponseEntity<BaseResponse<CreateRetrospectResponse>> createRetrospect(
    @AuthenticationPrincipal User user, // 인증 유저 주입
    @Valid @RequestBody CreateRetrospectRequest req // Body 파싱 및 검증
) {
    var result = retrospectService.createRetrospect(user.getId(), req);
    return ResponseEntity.ok(BaseResponse.success(result));
}
```

**Rust (`handler.rs`)**
```rust
pub async fn create_retrospect(
    user: AuthUser,                         // 1. AuthenticationPrincipal 역할 (Extractor)
    State(state): State<AppState>,          // 2. Service/Repository 등 빈 주입 (DI)
    Json(req): Json<CreateRetrospectRequest>, // 3. @RequestBody 역할 (Extractor)
) -> Result<Json<BaseResponse<CreateRetrospectResponse>>, AppError> {
    
    // 4. 명시적 검증 호출 (@Valid 자동 동작과 달리 수동 호출 필요)
    req.validate()?; 

    let user_id = user.user_id()?;

    // 5. Service 호출 (비동기이므로 await 필수)
    let result = RetrospectService::create_retrospect(state, user_id, req).await?;

    Ok(Json(BaseResponse::success_with_message(result, "성공")))
}
```
> **Extractor 패턴**: Spring은 파라미터 타입이나 어노테이션을 보고 값을 주입하지만, Axum은 `FromRequest` 트레이트를 구현한 타입(`AuthUser`, `State`, `Json`)을 파라미터에 넣어 요청 데이터를 추출합니다.

### 2.3. Service와 Transaction

Spring의 `@Transactional`은 AOP로 동작하지만, Rust(SeaORM)에서는 트랜잭션 객체(`txn`)를 직접 생성하고 전달해야 합니다.

**Spring (Java)**
```java
@Service
public class RetrospectService {
    @Transactional // AOP 기반 트랜잭션
    public CreateRetrospectResponse createRetrospect(...) {
        // 1. 검증
        validateFutureDate(req.getDate());

        // 2. DB 저장 (JPA)
        var room = retroRoomRepository.save(new RetroRoom(...));
        var retro = retrospectRepository.save(new Retrospect(..., room));
        
        // 3. 질문 생성
        for (String q : method.getQuestions()) {
            responseRepository.save(new Response(q, retro));
        }
        
        return new CreateRetrospectResponse(...);
    }
}
```

**Rust (`service.rs`)**
```rust
impl RetrospectService {
    pub async fn create_retrospect(
        state: AppState, // DB 커넥션이 포함된 상태
        user_id: i64,
        req: CreateRetrospectRequest,
    ) -> Result<CreateRetrospectResponse, AppError> {
        
        // 1. 비즈니스 검증 (순수 함수 호출)
        Self::validate_future_datetime(...)?;

        // 2. 트랜잭션 시작 (명시적)
        let txn = state.db.begin().await?;

        // 3. DB 저장 (ActiveModel 사용 - JPA Entity와 유사)
        let room_model = retro_room::ActiveModel { ... };
        let room = room_model.insert(&txn).await?; // txn 전달

        let retrospect_model = retrospect::ActiveModel { ... };
        let retro = retrospect_model.insert(&txn).await?; // txn 전달

        // 4. 질문 생성
        let questions = req.retrospect_method.default_questions();
        for question in questions {
            let response_model = response::ActiveModel { ... };
            response_model.insert(&txn).await?; // txn 전달
        }

        // 5. 트랜잭션 커밋
        txn.commit().await?;

        Ok(CreateRetrospectResponse { ... })
    }
}
```
> **트랜잭션 관리**: Rust에서는 `txn` 객체가 스코프를 벗어나면(Drop) 자동으로 **Rollback** 됩니다. 따라서 `commit()`을 호출하지 않고 에러가 발생해 함수가 종료되면 안전하게 롤백됩니다. 이는 Spring의 예외 발생 시 롤백과 유사한 효과를 냅니다.

### 2.4. 날짜/시간 처리 (chrono vs java.time)

Spring(Java)의 `java.time` API와 Rust의 `chrono` 크레이트는 매우 유사한 구조를 가지고 있습니다.

**Spring (Java)**
```java
import java.time.*;

// 문자열 → LocalDate / LocalTime 파싱
LocalDate date = LocalDate.parse("2026-01-20");        // ISO_LOCAL_DATE
LocalTime time = LocalTime.parse("10:00");              // ISO_LOCAL_TIME
LocalDateTime dateTime = LocalDateTime.of(date, time);  // 결합

// KST 기준 현재 시각
ZonedDateTime nowKst = ZonedDateTime.now(ZoneId.of("Asia/Seoul"));

// 미래 검증
if (dateTime.isBefore(nowKst.toLocalDateTime())) {
    throw new IllegalArgumentException("미래 시점이어야 합니다");
}
```

**Rust (`service.rs`)**
```rust
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc};

// 문자열 → NaiveDate / NaiveTime 파싱
let date = NaiveDate::parse_from_str("2026-01-20", "%Y-%m-%d")?;
let time = NaiveTime::parse_from_str("10:00", "%H:%M")?;
let datetime = NaiveDateTime::new(date, time);  // 결합

// KST 기준 현재 시각 (UTC + 9시간, chrono-tz 없이 수동 오프셋)
let now_kst = Utc::now().naive_utc() + chrono::Duration::hours(9);

// 미래 검증
if datetime <= now_kst {
    return Err(AppError::BadRequest("미래 시점이어야 합니다".to_string()));
}
```

| Java (java.time) | Rust (chrono) | 설명 |
|-------------------|---------------|------|
| `LocalDate` | `NaiveDate` | 타임존 없는 날짜 |
| `LocalTime` | `NaiveTime` | 타임존 없는 시간 |
| `LocalDateTime` | `NaiveDateTime` | 타임존 없는 날짜+시간 |
| `ZonedDateTime` | `DateTime<Tz>` | 타임존 포함 날짜+시간 |
| `Instant.now()` | `Utc::now()` | UTC 현재 시각 |
| `LocalDate.parse(str)` | `NaiveDate::parse_from_str(str, fmt)` | 문자열 → 날짜 |
| `LocalDateTime.of(date, time)` | `NaiveDateTime::new(date, time)` | 날짜+시간 결합 |
| `DateTimeFormatter` | `format!("%Y-%m-%d")` | 포맷팅 |

> **핵심 차이**: Java는 `ZoneId.of("Asia/Seoul")`로 타임존을 명시하지만, 이 프로젝트에서는 `chrono-tz` 크레이트 없이 `UTC + 9시간`으로 KST를 수동 계산합니다. 또한 SeaORM의 `DateTime` 타입은 `NaiveDateTime`과 동일하여 DB 컬럼에 타임존 없이 저장됩니다.

---

## 3. Rust 특유의 개념 이해하기

Spring 개발자가 가장 헷갈려하는 Rust 개념들을 이 프로젝트 문맥에서 설명합니다.

### 3.1. `Result<T, E>`와 `?` 연산자

Java의 `try-catch` 대신 Rust는 반환값으로 성공(`Ok`)과 실패(`Err`)를 명시합니다.

```rust
// Java
try {
    req.validate();
} catch (ValidationException e) {
    throw new ResponseStatusException(HttpStatus.BAD_REQUEST, ...);
}

// Rust
req.validate()?; // 에러가 나면 즉시 함수를 종료하고 Err를 반환해라!
```
- `?` 연산자: "성공하면 값을 꺼내고, 실패하면 에러를 리턴하며 함수 종료"
- 이 프로젝트의 `AppError`는 `IntoResponse`를 구현하고 있어, `Err(AppError)`가 리턴되면 자동으로 적절한 HTTP 응답(400, 404, 500 등)으로 변환됩니다.

### 3.2. `Option<T>`

Java의 `Optional<T>`와 동일합니다. `null` 대신 사용합니다.

```rust
// DB 조회 결과: 있을 수도 있고 없을 수도 있음
let team_exists = team::Entity::find_by_id(req.team_id).one(&state.db).await?;

if team_exists.is_none() { // Java: if (teamOptional.isEmpty())
    return Err(AppError::TeamNotFound(...));
}
```

### 3.3. `ActiveModel` vs `Entity` vs `Model` (SeaORM)

JPA는 Entity 클래스 하나로 다 처리하지만, SeaORM은 3가지로 나뉩니다.

1.  **Entity**: 테이블 그 자체에 대한 메타데이터 (Repository 역할). `team::Entity::find()` 처럼 사용.
2.  **Model**: DB에서 읽어온 **읽기 전용** 데이터 객체 (DTO와 유사). `SELECT` 결과.
3.  **ActiveModel**: 데이터를 수정하거나 생성할 때 사용하는 객체 (Setter 역할). `INSERT/UPDATE` 시 사용.

```rust
// 데이터 삽입 시 (ActiveModel 사용)
let active_model = retro_room::ActiveModel {
    title: Set(req.project_name.clone()), // Set()으로 값 설정
    ..Default::default()                  // 나머지는 기본값(NULL/AutoIncrement)
};
active_model.insert(&txn).await?;
```

### 3.4. `clone()`

Java에서는 객체 참조가 기본적으로 공유되지만, Rust는 **소유권(Ownership)** 규칙 때문에 값을 다른 곳에서 쓰려면 복제해야 할 때가 있습니다.

```rust
title: Set(req.project_name.clone())
```
- `req`에 있는 `project_name` 문자열의 소유권을 `ActiveModel`로 넘겨줘야 하는데, `req`를 나중에도 쓸 수 있으니 복제본을 넘겨주는 것입니다. Spring에서는 신경 쓰지 않아도 되는 부분입니다.

---

## 4. API-011 구현의 핵심 흐름

이 API는 **"하나의 트랜잭션 안에서 4가지 테이블에 Insert"**하는 것이 핵심입니다.

1.  **URL 검증**: 정규식 대신 `strip_prefix` + `if let` 패턴 매칭으로 `http/https` 체크.
2.  **날짜/시간 검증 (chrono)**:
    - `NaiveDate::parse_from_str()` → 날짜 문자열 파싱 + 오늘 이후인지 확인
    - `NaiveTime::parse_from_str()` → 시간 문자열 파싱
    - `NaiveDateTime::new(date, time)` → 날짜+시간 결합
    - `Utc::now().naive_utc() + Duration::hours(9)` → KST 기준 현재와 비교하여 미래인지 검증
3.  **트랜잭션 (`txn`)**:
    - `RetroRoom` (초대 링크용) 생성 (`INVITATION_BASE_URL` 환경변수 + UUID v4로 초대 URL 생성) -> ID 획득
    - `Retrospect` (회고 본체) 생성 (`NaiveDateTime`으로 `start_time` 저장, 위의 Room ID를 외래키로 사용)
    - `Response` (질문 5개) 생성 (Enum의 `default_questions()` 템플릿 사용)
    - `RetroReference` (참고 URL) 생성
4.  **커밋**: 모든 과정이 성공해야 DB에 반영.

이 가이드가 Rust 코드의 낯설음을 덜고 비즈니스 로직을 파악하는 데 도움이 되길 바랍니다.
