# Spring 개발자를 위한 API-016 (임시 저장) 분석 가이드

이 문서는 Spring Framework에 익숙한 개발자를 위해 API-016(회고 답변 임시 저장)의 Rust 구현을 설명합니다.

---

## 1. Controller (Handler) 비교

**Spring (Java)**
```java
@PutMapping("/api/v1/retrospects/{retrospectId}/drafts")
public ResponseEntity<BaseResponse<DraftSaveResponse>> saveDraft(
    @AuthenticationPrincipal User user,
    @PathVariable Long retrospectId,
    @Valid @RequestBody DraftSaveRequest req
) {
    var result = retrospectService.saveDraft(user.getId(), retrospectId, req);
    return ResponseEntity.ok(BaseResponse.success(result));
}
```

**Rust (`handler.rs:229-252`)**
```rust
#[utoipa::path(put, path = "...")] // Swagger 어노테이션 (utoipa = Spring의 Swagger/SpringDoc)
pub async fn save_draft(
    user: AuthUser,                         // @AuthenticationPrincipal
    State(state): State<AppState>,          // @Autowired Service (DI)
    Path(retrospect_id): Path<i64>,         // @PathVariable
    Json(req): Json<DraftSaveRequest>,      // @RequestBody
) -> Result<Json<BaseResponse<DraftSaveResponse>>, AppError> {
    // PathVariable 검증 (Spring은 @Min(1) 등으로 가능)
    if retrospect_id < 1 {
        return Err(AppError::BadRequest(...));
    }

    // 사용자 ID 추출 (AuthUser 헬퍼 메서드 사용)
    let user_id = user.user_id()?;  // JWT의 sub 클레임을 i64로 파싱

    // Service 호출 (Rust는 static method 호출)
    let result = RetrospectService::save_draft(state, user_id, retrospect_id, req).await?;

    Ok(Json(BaseResponse::success_with_message(result, "임시 저장이 완료되었습니다.")))
}
```

## 2. Service 로직 비교 (Dirty Checking vs Explicit Update)

Spring Data JPA는 **Dirty Checking**을 통해 객체의 필드만 변경하면 트랜잭션 커밋 시 자동으로 UPDATE 쿼리가 나갑니다. 반면, SeaORM은 **ActiveModel**을 통해 명시적으로 업데이트를 수행해야 합니다.

**Spring (JPA)**
```java
@Transactional
public void saveDraft(...) {
    // 1. 엔티티 조회
    List<Response> responses = responseRepository.findByUserAndRetro(...);
    
    // 2. 필드 수정 (Dirty Checking)
    for (DraftItem draft : req.getDrafts()) {
        Response resp = responses.get(draft.getQuestionNumber() - 1);
        resp.setContent(draft.getContent()); // 여기서 끝!
    }
    // 3. 메서드 종료 시 자동 flush & commit
}
```

**Rust (SeaORM)**
```rust
// 1. 엔티티(Model) 조회
let responses = response::Entity::find()...all(&state.db).await?;

// 타임스탬프를 루프 밖에서 한 번만 생성 (모든 답변이 동일한 시간)
let now = Utc::now().naive_utc();
// 트랜잭션 시작
let txn = state.db.begin().await?;

for draft in &req.drafts {
    let idx = (draft.question_number - 1) as usize;
    let response_model = &responses[idx];

    // 2. ActiveModel로 변환 (모든 필드가 Unchanged 상태)
    let mut active: response::ActiveModel = response_model.clone().into();

    // 3. 변경할 필드만 Set으로 명시 (Unchanged -> Set)
    active.content = Set(draft.content.clone().unwrap_or_default());
    active.updated_at = Set(now);

    // 4. 명시적 Update 호출 (Set 된 필드만 SQL UPDATE에 포함)
    active.update(&txn).await?;
}

// 5. 명시적 Commit
txn.commit().await?;
```

> **핵심 차이**: Rust(SeaORM)는 "영속성 컨텍스트(Persistence Context)"가 없습니다. 조회한 객체는 DB와 연결이 끊긴 순수 데이터 덩어리(Struct)일 뿐입니다. 따라서 변경 사항을 DB에 반영하려면 `ActiveModel`을 만들고 `update` 메서드를 호출하여 SQL을 실행시켜야 합니다.

> **ActiveModel 상태**: `Model`을 `.into()`로 `ActiveModel`로 변환하면 모든 필드는 `Unchanged` 상태가 됩니다. `Set()`으로 변경한 필드만 `UPDATE` SQL에 포함됩니다. JPA의 Dirty Checking은 모든 필드를 비교하지만, SeaORM은 명시적으로 `Set()`한 필드만 업데이트합니다.

## 응답 생성 시 KST 변환

서비스 로직에서는 DB에 UTC로 저장한 `now` 값을 응답 DTO에서는 KST(+9시간)로 변환하여 반환합니다.

```rust
// UTC 시간을 KST로 변환하여 표시용으로 사용
let kst_display = (now + chrono::Duration::hours(9))
    .format("%Y-%m-%d")
    .to_string();

Ok(DraftSaveResponse {
    retrospect_id,
    updated_at: kst_display,  // KST 날짜 문자열
})
```

Spring에서는 보통 `ZonedDateTime`이나 `@JsonFormat`으로 타임존 변환을 처리하지만, Rust에서는 `chrono::Duration`을 수동으로 더해 변환합니다.

## 3. Optional 처리 (`Option` vs `Optional`)

**Spring (Java)**
```java
// DTO
String content; // null 가능

// Service
String newContent = draft.getContent() != null ? draft.getContent() : "";
```

**Rust**
```rust
// DTO
pub content: Option<String>, // Some("text") or None

// Service
// unwrap_or_default(): None이면 String의 기본값("") 반환
let new_content = draft.content.clone().unwrap_or_default();
```

## 4. 예외 처리 흐름

Spring은 `RuntimeException`을 던지면 `@ExceptionHandler`나 `GlobalExceptionHandler`가 잡아서 처리합니다. Rust는 `Result` 타입을 반환하고 `?` 연산자로 전파하면, 최상위 핸들러나 미들웨어(`IntoResponse` impl)에서 이를 HTTP 응답으로 변환합니다.

- **Spring**: `throw new CustomException(...)` -> AOP가 Catch
- **Rust**: `return Err(AppError::...)` -> 호출자가 받아서 처리하거나 `?`로 전달

## 5. 요약

1.  **명시성**: Rust는 트랜잭션, 업데이트, 에러 전파 등 모든 것이 명시적입니다. "마법"이 적은 대신 코드가 조금 더 길어질 수 있습니다.
2.  **데이터 모델**: JPA의 Entity는 DB와 매핑되면서 로직도 갖지만, SeaORM의 Model은 순수 데이터(DTO에 가까움)이고, ActiveModel이 변경을 담당합니다.
3.  **안전성**: 컴파일 타임에 `Option` 체크, 타입 변환 등을 강제하여 런타임 NullPointerException 등을 방지합니다.
