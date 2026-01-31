# Spring 개발자를 위한 Rust API 학습 가이드 (API-013 회고 삭제)

이 가이드는 JVM/Spring 환경에 익숙한 개발자가 Rust/Axum/SeaORM으로 작성된 '회고 삭제 API'를 이해할 수 있도록 돕는 문서입니다.

## 1. 아키텍처 매핑

Spring의 계층형 아키텍처는 Rust 프로젝트에서도 유사하게 적용됩니다.

| 역할 | Spring (Java/Kotlin) | Rust (Axum/SeaORM) | 파일 위치 |
|------|---------------------|-------------------|-----------|
| **웹 계층** | `@RestController` | `handler` 함수 | `domain/retrospect/handler.rs` |
| **비즈니스 계층** | `@Service` | `Service` 구조체 impl | `domain/retrospect/service.rs` |
| **데이터 계층** | `@Repository` / `JpaRepository` | `Entity` / `ActiveModel` | `domain/retrospect/entity/` |
| **데이터 전송** | DTO Class/Record | struct (Serialize/Deserialize) | `domain/retrospect/dto.rs` |
| **의존성 주입** | `@Autowired` / 생성자 주입 | 함수 인자로 전달 (State) | `main.rs` (Router 설정) |

## 2. 코드 레벨 상세 비교

### 2.1. 컨트롤러 vs 핸들러

**Spring:**
```java
@DeleteMapping("/api/v1/retrospects/{retrospectId}")
public ResponseEntity<BaseResponse<Void>> deleteRetrospect(
    @AuthenticationPrincipal User user,
    @PathVariable Long retrospectId
) {
    retrospectService.deleteRetrospect(user.getId(), retrospectId);
    return ResponseEntity.ok(BaseResponse.success("삭제 완료"));
}
```

**Rust (handler.rs:646-668):**
```rust
#[utoipa::path(
    delete,
    path = "/api/v1/retrospects/{retrospectId}",
    params(("retrospectId" = i64, Path, description = "삭제할 회고의 고유 식별자")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "회고가 성공적으로 삭제되었습니다.", body = SuccessDeleteRetrospectResponse),
        (status = 400, description = "잘못된 Path Parameter", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 404, description = "존재하지 않는 회고이거나 접근 권한 없음 (보안상 404로 통합)", body = ErrorResponse),
        (status = 500, description = "서버 내부 오류", body = ErrorResponse)
    ),
    tag = "Retrospect"
)]
pub async fn delete_retrospect(
    user: AuthUser,                  // @AuthenticationPrincipal 대응 (Extractor)
    State(state): State<AppState>,   // 의존성 주입 (DB 커넥션 등)
    Path(retrospect_id): Path<i64>,  // @PathVariable 대응
) -> Result<Json<BaseResponse<()>>, AppError> { // ResponseEntity 대응
    if retrospect_id < 1 {
        return Err(AppError::BadRequest("retrospectId는 1 이상의 양수여야 합니다.".to_string()));
    }
    let user_id = user.user_id()?;
    RetrospectService::delete_retrospect(state, user_id, retrospect_id).await?;
    Ok(Json(BaseResponse::success_with_message((), "회고가 성공적으로 삭제되었습니다.")))
}
```

> **참고**: API 스펙에서는 403 에러(Owner/Creator 권한)를 정의하고 있으나, 구현의 Swagger에는 403이 없습니다. 현재 팀 멤버라면 누구나 삭제 가능합니다.

**핵심 차이점:**
- **Extractor**: Rust는 함수 인자 타입(`Path`, `State`, `Json` 등)을 통해 요청 데이터를 추출합니다. Spring의 어노테이션(`@PathVariable`, `@RequestBody`)과 역할이 같습니다.
- **Result 반환**: Rust는 예외(`throw`) 대신 `Result<T, E>`를 반환합니다. 에러 발생 시 `?` 연산자로 조기 리턴합니다.

### 2.2. 서비스 메서드 & 트랜잭션

**Spring:**
```java
@Transactional
public void deleteRetrospect(Long userId, Long retrospectId) {
    // 1. 조회 및 권한 체크
    Retrospect retro = retrospectRepository.findById(retrospectId)
        .orElseThrow(() -> new NotFoundException("..."));
    
    if (!memberTeamRepository.existsByMemberIdAndTeamId(userId, retro.getTeamId())) {
        throw new NotFoundException("..."); // 보안상 404
    }

    // 2. Cascade 삭제 (JPA 설정에 따르거나 수동 삭제)
    commentRepository.deleteByRetrospectId(retrospectId);
    // ...
    retrospectRepository.delete(retro);
}
```

**Rust (service.rs:1134-1282):**
```rust
/// 회고 삭제 (API-013)
///
/// TODO: 현재 스키마에 `created_by`(회고 생성자) 필드와 `member_team.role`(팀 역할) 필드가 없어
/// 팀 멤버십만 확인합니다. 스펙상 팀 Owner 또는 회고 생성자만 삭제 가능해야 하므로,
/// 스키마 마이그레이션 후 권한 분기를 추가해야 합니다.
pub async fn delete_retrospect(
    state: AppState,
    user_id: i64,
    retrospect_id: i64,
) -> Result<(), AppError> {
    info!(user_id = user_id, retrospect_id = retrospect_id, "회고 삭제 요청");

    // 1. 조회 및 팀 멤버십 확인 (헬퍼 함수 사용)
    let retrospect_model =
        Self::find_retrospect_for_member(&state, user_id, retrospect_id).await?;
    let retrospect_room_id = retrospect_model.retrospect_room_id;

    // 2. 트랜잭션 시작 (명시적)
    let txn = state.db.begin().await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    // 3. Cascade 삭제 (수동 구현 - FK 의존 역순)
    //    response_comment -> response_like -> member_response
    //    -> response -> retro_reference -> member_retro -> retrospect
    //    -> (조건부) member_retro_room -> retro_room

    // 4. 커밋
    txn.commit().await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(())
}
```

> **스펙과의 차이**: Spring 예시에서는 권한 체크(`existsByMemberIdAndTeamId`)가 간단하게 표현되었지만, 실제 Rust 구현에서도 팀 멤버십 확인만 수행합니다. 스펙에 정의된 Owner/Creator 권한 검증은 DB 스키마 미비로 구현되지 않았습니다.

**핵심 차이점:**
- **명시적 트랜잭션**: Spring의 `@Transactional` AOP 대신, Rust는 `db.begin()`과 `txn.commit()`으로 트랜잭션 범위를 명시적으로 제어합니다.
- **Async/Await**: DB I/O가 발생하는 모든 곳에 `.await?`가 붙습니다. 이는 Non-blocking I/O를 위함입니다.
- **에러 매핑**: SeaORM의 `DbErr`은 `.map_err(|e| AppError::InternalError(e.to_string()))?`로 일관되게 변환됩니다. Spring의 `DataAccessException` 계층과 유사한 역할입니다.

### 2.3. JPA vs SeaORM (데이터 접근)

**Spring Data JPA:**
```java
// 삭제 (단건)
repository.delete(entity);

// 삭제 (벌크)
@Modifying
@Query("DELETE FROM Comment c WHERE c.responseId IN :ids")
void deleteByResponseIds(List<Long> ids);
```

**SeaORM (Rust):**
```rust
// 삭제 (단건)
model.delete(&txn).await?;

// 삭제 (벌크 - delete_many)
Entity::delete_many()
    .filter(Column::ResponseId.is_in(ids))
    .exec(&txn)
    .await?;
```

**핵심 차이점:**
- **Entity vs ActiveModel**: JPA의 Entity는 객체 그 자체지만, SeaORM은 데이터 조회용 `Model`과 변경용 `ActiveModel`이 분리되어 있습니다.
- **쿼리 빌더**: SeaORM은 메서드 체이닝(`filter`, `order_by`, `limit` 등)으로 SQL을 동적으로 생성합니다. QueryDSL이나 JOOQ와 유사합니다.

## 3. Rust 특유의 개념 이해하기

### 3.1. Option vs Optional

Java의 `Optional<T>`는 Rust의 `Option<T>`와 매우 유사합니다.
- `Some(value)` == `Optional.of(value)`
- `None` == `Optional.empty()`

**코드 예시 (service.rs):**
```rust
let is_member = member_team::Entity::find()...one(&db).await?; // 결과는 Option<Model>

if is_member.is_none() { // !isPresent()
    return Err(...);
}
```

### 3.2. Result vs Exception

Java는 예외를 `throw`하여 흐름을 중단시키지만, Rust는 `Result<Success, Error>` 타입을 반환합니다.
- `Ok(val)`: 성공 값
- `Err(e)`: 실패 에러

**`?` 연산자의 마법:**
```rust
// Java
User user = repo.findById(id).orElseThrow(() -> new RuntimeException());

// Rust
let user = repo::find_by_id(id).one(&db).await? // 에러나면 즉시 리턴 (throws와 유사)
    .ok_or_else(|| AppError::NotFound(...))?;   // None이면 에러로 변환해서 리턴
```
`?`는 "성공하면 값을 꺼내고, 실패하면 함수를 즉시 종료하고 에러를 반환해라"라는 의미입니다.

### 3.3. 소유권 (Ownership)과 Clone

Java는 가비지 컬렉터(GC)가 메모리를 관리하지만, Rust는 소유권 규칙으로 관리합니다.
`delete_retrospect` 로직에서 `response_ids.clone()`이 자주 등장하는 이유입니다.

```rust
// response_ids 벡터(List)를 여러 번 사용해야 함
// 첫 번째 사용 시 소유권이 넘어가버리면(move), 두 번째부터는 사용할 수 없음
// 따라서 .clone()으로 복사본을 넘겨줌

response_comment::Entity::delete_many()
    .filter(Column::ResponseId.is_in(response_ids.clone())) // 복사해서 전달
    .exec(&txn).await?;

response_like::Entity::delete_many()
    .filter(Column::ResponseId.is_in(response_ids.clone())) // 또 복사해서 전달
    .exec(&txn).await?;
```

## 4. 학습 순서 제안

1. **`handler.rs` 읽기**: HTTP 요청이 어떻게 들어와서 파라미터가 어떻게 파싱되는지 확인하세요.
2. **`service.rs`의 `delete_retrospect` 함수 흐름 따라가기**:
   - `find_retrospect_for_member`로 권한 체크하는 부분
   - `db.begin()`으로 트랜잭션 시작하는 부분
   - `delete_many`로 연관 테이블 삭제하는 순서 (FK 제약조건 때문)
3. **`entity` 파일들 확인**: `codes/server/src/domain/retrospect/entity/` 내의 파일들을 보며 DB 테이블과 어떻게 매핑되는지 확인하세요.

## 5. 자주 묻는 질문 (FAQ)

**Q: 왜 `@Transactional` 어노테이션을 안 쓰나요?**
A: Rust의 매크로 시스템으로 비슷하게 구현할 수도 있지만, 명시적인 제어(`begin`, `commit`)를 더 선호하는 경향이 있습니다. 또한 `async` 환경에서 트랜잭션 수명 주기를 명확히 관리하기 위함입니다.

**Q: `unwrap()`은 뭔가요?**
A: Java의 `Optional.get()`과 같습니다. 값이 확실히 있다고 보장될 때만 써야 하며, 만약 `None`이면 프로그램이 패닉(종료)됩니다. 프로덕션 코드에서는 에러 처리를 위해 `?`나 `match`를 사용하는 것이 권장됩니다.

**Q: `await?`는 왜 항상 같이 다니나요?**
A: `await`는 비동기 작업이 끝날 때까지 기다리는 것이고, `?`는 그 결과가 에러인지 확인하는 것입니다. DB 작업은 비동기이면서 실패할 수 있으므로 두 키워드가 함께 쓰이는 경우가 많습니다.
