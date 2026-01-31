# Spring 개발자를 위한 Rust API 구현 가이드 (API-014)

이 문서는 **API-014 (회고 참여자 등록)**의 Rust 구현을 Spring(Java/Kotlin) 개발자의 관점에서 이해하기 쉽게 설명합니다.
Rust의 문법, 프레임워크(Axum), ORM(SeaORM)을 Spring 생태계의 개념과 매핑하여 해설합니다.

## 1. 아키텍처 매핑

Spring의 계층형 아키텍처와 유사하게 구성되어 있습니다.

| 역할 | Rust (현재 프로젝트) | Spring (JVM) | 비고 |
|------|-------------------|--------------|------|
| **Web Layer** | `handler.rs` (Axum) | `Controller` | HTTP 요청 처리 및 검증 |
| **Service Layer** | `service.rs` | `Service` | 비즈니스 로직 수행 |
| **Persistence** | `SeaORM` (Entity/ActiveModel) | `JPA / Hibernate` | 데이터베이스 접근 |
| **DTO** | `dto.rs` (Serde) | `DTO` (Jackson) | 데이터 전송 객체 |
| **Error Handling** | `Result<T, AppError>` | `try-catch` / `ExceptionHandler` | 에러 처리 방식 |

---

## 2. 코드 상세 분석

### 2.1 Handler Layer (`handler.rs`)

**Rust 코드:**
```rust
// POST /api/v1/retrospects/{retrospectId}/participants
pub async fn create_participant(
    user: AuthUser,                    // 1. 커스텀 Extractor (인증 정보)
    State(state): State<AppState>,     // 2. 의존성 주입 (DB 등 전역 상태)
    Path(retrospect_id): Path<i64>,    // 3. 경로 파라미터 추출
) -> Result<Json<BaseResponse<CreateParticipantResponse>>, AppError> { // 4. 반환 타입
    
    // 유효성 검증
    if retrospect_id < 1 {
        return Err(AppError::BadRequest("...".to_string()));
    }

    let user_id = user.user_id()?;

    // 서비스 호출
    let result = RetrospectService::create_participant(state, user_id, retrospect_id).await?;

    // 응답 래핑
    Ok(Json(BaseResponse::success_with_message(result, "...")))
}
```

**Spring 관점 해석:**

1.  **`user: AuthUser`**: Spring Security의 `@AuthenticationPrincipal`과 유사합니다. 요청 헤더의 JWT를 파싱하여 유저 정보를 주입해주는 **Extractor**입니다. Axum에서는 미들웨어 대신 이런 방식으로 핸들러 파라미터에서 직접 추출 처리를 할 수 있습니다.
2.  **`State(state)`**: Spring의 의존성 주입(`@Autowired` or 생성자 주입)과 같습니다. `AppState`에는 DB Connection Pool 등이 들어있어, 서비스 계층으로 넘겨줍니다.
3.  **`Path(retrospect_id)`**: Spring MVC의 `@PathVariable("retrospectId") Long retrospectId`와 정확히 동일합니다.
4.  **`Result<T, E>` 반환**: Spring은 예외(Exception)를 던지면(`throw`) `@ControllerAdvice`가 잡아서 처리하지만, Rust는 **성공(`Ok`)** 또는 **실패(`Err`)**를 명시적으로 반환해야 합니다. `AppError`가 발생하면 자동으로 적절한 HTTP 응답으로 변환됩니다.

---

### 2.2 Service Layer (`service.rs`)

**Rust 코드:**
```rust
pub async fn create_participant(
    state: AppState,
    user_id: i64,
    retrospect_id: i64,
) -> Result<CreateParticipantResponse, AppError> {
    
    // 1. 회고 조회 및 권한 확인 (별도 헬퍼 메소드 사용)
    let retrospect_model = Self::find_retrospect_for_member(&state, user_id, retrospect_id).await?;

    // 2. 시간 검증 (현재 시간 vs 회고 시작 시간)
    let now_kst = Utc::now().naive_utc() + chrono::Duration::hours(9);
    if retrospect_model.start_time <= now_kst {
        return Err(AppError::RetrospectAlreadyStarted("...".to_string()));
    }

    // 3. 중복 참여 확인 (DB 조회)
    let existing_participant = member_retro::Entity::find()
        .filter(member_retro::Column::MemberId.eq(user_id))
        .filter(member_retro::Column::RetrospectId.eq(retrospect_id))
        .one(&state.db) // Optional 반환
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    if existing_participant.is_some() { // Java의 if (opt.isPresent())
        return Err(AppError::ParticipantDuplicate("...".to_string()));
    }

    // ... (닉네임 추출 로직 생략) ...

    // 4. 저장 (ActiveModel 사용)
    let member_retro_model = member_retro::ActiveModel {
        member_id: Set(user_id),
        retrospect_id: Set(retrospect_id),
        personal_insight: Set(None), // null 설정
        ..Default::default()
    };

    let inserted = member_retro_model.insert(&state.db).await...;

    Ok(CreateParticipantResponse { ... })
}
```

**Spring 관점 해석:**

1.  **`find_retrospect_for_member`**: `RetrospectRepository.findByIdAndMemberId(...)` 같은 로직을 수행하는 메소드입니다. 없을 경우 예외(`Err`)를 반환하여 흐름을 중단시킵니다.
2.  **`chrono` (날짜/시간)**: Java의 `java.time.LocalDateTime`과 유사합니다. Rust는 Timezone 처리가 명시적이므로 KST(+9) 변환 로직이 보입니다.
3.  **`Entity::find()` (SeaORM)**: JPA Criteria API나 QueryDSL과 비슷하게 체이닝 방식으로 쿼리를 빌드합니다.
    *   `.filter(...)`: `where` 절
    *   `.one(&db)`: `findOne` / `fetchOne` (결과가 0개면 `None`, 1개면 `Some`)
4.  **`is_some()`**: Java `Optional.isPresent()`와 같습니다.
5.  **`ActiveModel`**: JPA의 Entity 객체 생성과 유사하지만, **Builder 패턴**처럼 동작합니다. `Set(값)`으로 변경할 필드만 지정하고, `.insert(&db)`를 호출하면 `INSERT` 쿼리가 실행됩니다. `Save` 메소드와 유사합니다.

---

### 2.3 DB 계층 (SeaORM vs JPA)

Spring 개발자가 가장 낯설어하는 부분이 ORM일 것입니다.

**JPA (Spring):**
```java
// Entity 수정
User user = userRepository.findById(1L).get();
user.setName("New Name");
// 트랜잭션 종료 시 Dirty Checking으로 자동 update
```

**SeaORM (Rust):**
```rust
// ActiveModel로 변환 후 수정
let mut user: ActiveModel = user_entity.into();
user.name = Set("New Name".to_owned());
user.update(&db).await?; // 명시적으로 update 호출 필요
```

*   **JPA**: 영속성 컨텍스트(Persistence Context)가 변경을 감지(Dirty Checking).
*   **SeaORM**: 그런 마법이 없습니다. 명시적으로 `insert`, `update`, `delete`를 호출해야 합니다. 코드는 길어질 수 있지만, **언제 쿼리가 나가는지 명확**하다는 장점이 있습니다.

---

## 3. 에러 처리 흐름 (`?` 연산자)

Rust 코드 곳곳에 있는 `?`는 Spring의 예외 전파와 비슷합니다.

```rust
let user = find_user(id).await?;
```

위 코드는 아래 Java 코드와 논리적으로 같습니다.

```java
User user;
try {
    user = findUser(id); // 예외 발생 시 catch 블록이나 상위로 throw
} catch (Exception e) {
    throw e;
}
```

`?`는 "함수 실행 결과가 에러(`Err`)라면, 즉시 현재 함수를 종료하고 그 에러를 리턴해라"라는 뜻입니다. 덕분에 `try-catch` 지옥 없이 깔끔한 코드가 가능합니다.

## 4. 요약

이 API는 Spring으로 치면 아래와 같은 흐름입니다.

1.  **Controller**: `@PostMapping`으로 요청을 받고, 토큰을 검증(`UserPrincipal`)합니다.
2.  **Service**:
    *   `RetrospectRepository`를 통해 회고가 존재하는지, 내가 팀원인지 확인합니다.
    *   `LocalDateTime`을 비교하여 이미 지난 회고인지 체크합니다.
    *   `ParticipantRepository`를 조회하여 이미 참여했는지 체크합니다. (중복 시 `409 Conflict`)
    *   모든 검증 통과 시 `save()` 하여 참여 정보를 저장합니다.
3.  **Response**: 성공 시 DTO에 담아 `200 OK`를 리턴합니다.

Rust/Axum/SeaORM 스택이라 문법은 다르지만, **계층형 아키텍처와 로직의 흐름은 Spring과 99% 동일**합니다.
