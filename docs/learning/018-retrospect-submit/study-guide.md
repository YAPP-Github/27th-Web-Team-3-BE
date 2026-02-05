# Spring 개발자를 위한 Rust API 학습 가이드 (API-017 회고 최종 제출)

이 가이드는 JVM/Spring 환경에 익숙한 개발자가 Rust/Axum/SeaORM으로 작성된 '회고 최종 제출 API'를 이해할 수 있도록 돕는 문서입니다.

## 1. 아키텍처 매핑

Spring의 계층형 아키텍처는 Rust 프로젝트에서도 유사하게 적용됩니다.

| 역할 | Spring (Java/Kotlin) | Rust (Axum/SeaORM) | 파일 위치 |
|------|---------------------|-------------------|-----------|
| **웹 계층** | `@RestController` | `handler` 함수 | `domain/retrospect/handler.rs` |
| **비즈니스 계층** | `@Service` | `Service` 구조체 impl | `domain/retrospect/service.rs` |
| **도메인 모델** | `@Entity` / Enum | `Entity` / `ActiveEnum` | `domain/member/entity/member_retro.rs` |
| **데이터 전송** | DTO Class/Record | struct (Serialize/Deserialize) | `domain/retrospect/dto.rs` |
| **예외 처리** | `@ExceptionHandler` | `AppError` enum | `utils/error.rs` |

## 2. 코드 레벨 상세 비교

### 2.1. 컨트롤러 vs 핸들러

**Spring:**
```java
@PostMapping("/api/v1/retrospects/{retrospectId}/submit")
public ResponseEntity<BaseResponse<SubmitRetrospectResponse>> submitRetrospect(
    @AuthenticationPrincipal User user,
    @PathVariable Long retrospectId,
    @RequestBody @Valid SubmitRetrospectRequest req
) {
    SubmitRetrospectResponse result = retrospectService.submitRetrospect(user.getId(), retrospectId, req);
    return ResponseEntity.ok(BaseResponse.success(result, "회고 제출이 성공적으로 완료되었습니다."));
}
```

**Rust (handler.rs):**
```rust
#[utoipa::path(...)]
pub async fn submit_retrospect(
    user: AuthUser,                  // @AuthenticationPrincipal 대응
    State(state): State<AppState>,   // 의존성 주입
    Path(retrospect_id): Path<i64>,  // @PathVariable 대응
    Json(req): Json<SubmitRetrospectRequest>, // @RequestBody 대응
) -> Result<Json<BaseResponse<SubmitRetrospectResponse>>, AppError> {
    if retrospect_id < 1 { ... } // 간단한 입력값 검증

    let user_id = user.user_id()?;
    let result = RetrospectService::submit_retrospect(state, user_id, retrospect_id, req).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "회고 제출이 성공적으로 완료되었습니다.",
    )))
}
```

**핵심 차이점:**
- **Extractor**: Rust는 함수 인자 타입(`Path`, `Json` 등)을 통해 요청 데이터를 추출합니다. Spring의 `@RequestBody`, `@PathVariable`과 역할이 같습니다.
- **Valid**: Spring은 `@Valid`로 DTO 필드 검증을 자동화하지만, Rust는 서비스 레이어에서 명시적으로 검증 함수(`validate_answers`)를 호출하는 경우가 많습니다 (특히 비즈니스 로직이 포함된 검증).

### 2.2. 서비스 메서드 & 트랜잭션 & 락(Lock)

**Spring:**
```java
@Transactional
public SubmitRetrospectResponse submitRetrospect(Long userId, Long retrospectId, SubmitRequest req) {
    // 1. 비즈니스 검증
    validateAnswers(req.getAnswers());

    // 2. 조회 및 락 (PESSIMISTIC_WRITE)
    MemberRetro memberRetro = memberRetroRepository.findByMemberIdAndRetrospectId(userId, retrospectId)
        .orElseThrow(() -> new NotFoundException(...));
    
    // 비관적 락 적용 (SELECT ... FOR UPDATE)
    entityManager.lock(memberRetro, LockModeType.PESSIMISTIC_WRITE);

    // 3. 상태 체크
    if (memberRetro.getStatus() != Status.DRAFT) {
        throw new ForbiddenException("이미 제출이 완료된 회고입니다.");
    }

    // 4. 답변 업데이트 (Dirty Checking)
    List<Response> responses = responseRepository.findByMemberId(userId);
    // ... 답변 내용 업데이트 ...

    // 5. 상태 변경
    memberRetro.setStatus(Status.SUBMITTED);
    memberRetro.setSubmittedAt(LocalDateTime.now());
    
    return new SubmitRetrospectResponse(...);
}
```

**Rust (service.rs):**
```rust
pub async fn submit_retrospect(state: AppState, user_id: i64, retro_id: i64, req: SubmitRequest) -> ... {
    // 1. 비즈니스 검증
    Self::validate_answers(&req.answers)?;

    // 2. 트랜잭션 시작
    let txn = state.db.begin().await?;

    // 3. 조회 및 락 (lock_exclusive)
    let member_retro_model = member_retro::Entity::find()
        .filter(...)
        .lock_exclusive() // SELECT ... FOR UPDATE
        .one(&txn).await?
        .ok_or_else(|| AppError::RetrospectNotFound(...))?;

    // 4. 상태 체크
    if member_retro_model.status == RetrospectStatus::Submitted {
        return Err(AppError::RetroAlreadySubmitted(...));
    }

    // 5. 답변 업데이트 (ActiveModel)
    for answer in &req.answers {
        let mut active: response::ActiveModel = response_model.clone().into();
        active.content = Set(answer.content.clone());
        active.update(&txn).await?;
    }

    // 6. 상태 변경
    let mut active_mr: member_retro::ActiveModel = member_retro_model.clone().into();
    active_mr.status = Set(RetrospectStatus::Submitted);
    active_mr.update(&txn).await?;

    // 7. 커밋
    txn.commit().await?;
    
    Ok(...)
}
```

**핵심 차이점:**
- **락(Lock)**: Spring JPA의 `@Lock(PESSIMISTIC_WRITE)` 대신 Rust SeaORM은 `.lock_exclusive()` 메서드를 사용하여 `SELECT ... FOR UPDATE` 쿼리를 생성합니다.
- **업데이트**: JPA는 객체 상태만 변경하면 트랜잭션 종료 시 자동 반영(Dirty Checking)되지만, SeaORM은 `ActiveModel`을 생성하고 `.update(&txn)`을 명시적으로 호출해야 합니다.

### 2.3. 검증 로직 (Validation)

**Spring:**
```java
if (answers.size() != 5) {
    throw new BadRequestException("...");
}
if (answer.getContent().trim().isEmpty()) {
    throw new BadRequestException("...");
}
```

**Rust:**
```rust
if answers.len() != 5 {
    return Err(AppError::RetroAnswersMissing(...));
}
if answer.content.trim().is_empty() {
    return Err(AppError::RetroAnswerWhitespaceOnly(...));
}
```
Rust의 `trim()`은 원본 문자열을 변경하지 않고 공백이 제거된 슬라이스(`&str`)를 반환합니다. `is_empty()`로 공백 전용 여부를 체크합니다.

## 3. Rust 특유의 개념 이해하기

### 3.1. HashSet을 이용한 완전성 검사

제출된 답변의 질문 번호가 1~5번이 모두 포함되어 있는지 확인할 때 Rust의 `HashSet`과 `Range`를 활용합니다.

```rust
// 제출된 질문 번호들 수집
let question_numbers: HashSet<i32> = answers.iter().map(|a| a.question_number).collect();

// 기대하는 질문 번호 집합 (1, 2, 3, 4, 5)
let expected: HashSet<i32> = (1..=5).collect(); // 1..=5는 1 이상 5 이하 범위

if question_numbers != expected {
    // 집합 비교로 누락/중복 한 번에 체크
    return Err(AppError::RetroAnswersMissing(...));
}
```
Java의 `Set.equals()`와 유사하지만, Rust의 Range 문법(`1..=5`)과 `collect()`를 사용해 더 간결하게 표현됩니다.

### 3.2. 상태 머신 (Enum)

Rust의 enum은 단순 상수 집합이 아니라 타입 시스템의 일부입니다. SeaORM의 `DeriveActiveEnum`을 통해 DB의 문자열 값과 Rust의 enum이 매핑됩니다.

```rust
#[derive(EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "RetrospectStatus")]
pub enum RetrospectStatus {
    #[sea_orm(string_value = "DRAFT")]
    Draft,
    #[sea_orm(string_value = "SUBMITTED")]
    Submitted,
    // ...
}
```
Spring의 `Enumerated(EnumType.STRING)`과 유사합니다.

### 3.3. UTC 저장과 KST 변환

Rust `chrono` 라이브러리를 사용하여 시간을 다룹니다.

```rust
// 1. 현재 UTC 시간 생성 (DB 저장용)
let now = Utc::now().naive_utc(); 

// 2. KST 변환 (API 응답용)
// Duration::hours(9)를 더해 한국 시간 계산
let kst_display = (now + chrono::Duration::hours(9))
    .format("%Y-%m-%d") // Java의 DateTimeFormatter와 유사
    .to_string();
```
Spring에서는 보통 `ZoneId.of("Asia/Seoul")`을 사용하지만, 여기서는 UTC 기준 오프셋 연산을 직접 수행하는 패턴을 사용했습니다.

## 4. 학습 순서 제안

1. **`handler.rs` 읽기**: HTTP 요청 바디(`SubmitRetrospectRequest`)가 어떻게 들어오는지 확인하세요.
2. **`service.rs`의 `validate_answers` 함수 확인**: 트랜잭션 시작 전에 어떤 비즈니스 검증을 수행하는지 보세요.
3. **`service.rs`의 `submit_retrospect` 함수 흐름 따라가기**:
   - `lock_exclusive()`로 동시성 제어하는 부분
   - `response` 테이블과 `member_retro` 테이블을 각각 업데이트하는 부분
   - 트랜잭션 커밋 순서
4. **`dto.rs` 확인**: 요청/응답 객체의 구조를 확인하세요.

## 5. 자주 묻는 질문 (FAQ)

**Q: 왜 `ActiveModel`을 쓸 때 `clone().into()`를 하나요?**
A: `Entity::find()`로 가져온 `Model`은 읽기 전용 구조체입니다. 이를 수정 가능한 `ActiveModel`로 변환하려면 `into()`가 필요한데, 원본 데이터를 유지하거나 소유권 문제를 피하기 위해 `clone()` 후 변환하는 패턴을 자주 사용합니다.

**Q: `lock_exclusive()`는 DB 락을 거나요?**
A: 네, 실행되는 SQL에 `FOR UPDATE` 구문을 추가합니다. 트랜잭션이 끝날 때까지 해당 행을 다른 트랜잭션이 수정하지 못하도록 막습니다. 이는 중복 제출을 방지하는 핵심 메커니즘입니다.

**Q: 답변이 5개가 아니면 어떻게 되나요?**
A: `validate_answers` 함수에서 `answers.len() != 5` 체크에 걸려 `RETRO4002` (400 Bad Request) 에러가 발생하고 트랜잭션은 시작조차 되지 않습니다.
