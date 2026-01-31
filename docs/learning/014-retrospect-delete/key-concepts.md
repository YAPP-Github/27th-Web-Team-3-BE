# [API-013] 핵심 개념

## Cascade 삭제 패턴

### 개념

관계형 데이터베이스에서 부모 레코드를 삭제할 때, FK(Foreign Key)로 연결된 자식 레코드를 먼저 삭제해야 한다. 이를 cascade 삭제라 하며, 이 API에서는 DB 수준의 `ON DELETE CASCADE`가 아닌 **애플리케이션 수준**에서 명시적으로 삭제 순서를 제어한다.

### 삭제 계층 구조

```
retro_room (최상위)
  └── retrospect
        ├── response
        │     ├── response_comment   ← 가장 먼저 삭제
        │     ├── response_like      ← 두 번째 삭제
        │     └── member_response    ← 세 번째 삭제
        ├── retro_reference
        └── member_retro
```

### 코드 구현

**소스**: `service.rs:1173-1230`

```rust
if !response_ids.is_empty() {
    // 4. 댓글 삭제 (response_comment) -- 자식 테이블부터 FK 의존 역순
    let comments_deleted = response_comment::Entity::delete_many()
        .filter(response_comment::Column::ResponseId.is_in(response_ids.clone()))
        .exec(&txn)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    // 5. 좋아요 삭제 (response_like)
    let likes_deleted = response_like::Entity::delete_many()
        .filter(response_like::Column::ResponseId.is_in(response_ids.clone()))
        .exec(&txn)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    // 6. 멤버 응답 매핑 삭제 (member_response)
    let member_responses_deleted = member_response::Entity::delete_many()
        .filter(member_response::Column::ResponseId.is_in(response_ids.clone()))
        .exec(&txn)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    info!(
        retrospect_id = retrospect_id,
        response_count = response_ids.len(),
        comments_deleted = comments_deleted.rows_affected,
        likes_deleted = likes_deleted.rows_affected,
        member_responses_deleted = member_responses_deleted.rows_affected,
        "연관 응답 데이터 삭제 완료"
    );
}

// 7. 응답 삭제 (response) -- 부모 테이블
let responses_deleted = response::Entity::delete_many()
    .filter(response::Column::RetrospectId.eq(retrospect_id))
    .exec(&txn)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```

**참고**: 실제 구현에서는 모든 `exec()` 호출에 `.map_err(|e| AppError::InternalError(e.to_string()))?`가 붙어 있으며, 삭제 결과를 변수에 저장하여 `rows_affected`를 구조화 로깅에 활용한다.

### 왜 애플리케이션 수준 cascade인가?

- DB의 `ON DELETE CASCADE`를 사용하면 삭제 순서를 제어할 수 없고, 삭제된 행 수를 로깅하기 어렵다
- 각 단계의 `rows_affected`를 로깅하여 운영 시 삭제 범위를 추적할 수 있다 (`service.rs:1195-1202`)
- 조건부 삭제(회고방)처럼 비즈니스 로직이 개입하는 경우 애플리케이션 수준이 유리하다

---

## 트랜잭션 내 순서 보장

### 개념

여러 테이블에 걸친 삭제는 반드시 하나의 트랜잭션 안에서 수행되어야 한다. 중간에 실패하면 이미 삭제된 데이터만 사라지고 나머지는 남아 데이터 정합성이 깨질 수 있기 때문이다.

### 코드 구현

**소스**: `service.rs:1157-1269`

```rust
// 트랜잭션 시작
let txn = state.db.begin().await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

// ... 모든 삭제 쿼리는 &txn을 통해 실행 ...

// 트랜잭션 커밋
txn.commit().await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```

### RAII 자동 Rollback

SeaORM의 `DatabaseTransaction`은 `Drop` trait을 구현하고 있어, `commit()`이 호출되지 않은 채 스코프를 벗어나면 자동으로 rollback된다.

```rust
// 만약 Step 5에서 에러 발생 시:
//   → map_err가 AppError를 반환
//   → ? 연산자로 함수가 조기 리턴
//   → txn이 drop됨 → 자동 rollback
//   → Step 1~4에서 삭제한 데이터도 원복
```

이는 Rust의 RAII(Resource Acquisition Is Initialization) 패턴을 활용한 것이다.

---

## find_retrospect_for_member 보안 패턴

### 개념

회고가 존재하지 않는 경우와 존재하지만 접근 권한이 없는 경우를 **동일한 404 에러**로 반환하여, 공격자가 회고의 존재 여부를 추측하지 못하도록 한다.

### 코드 구현

**소스**: `service.rs:323-352`

```rust
async fn find_retrospect_for_member(
    state: &AppState,
    user_id: i64,
    retrospect_id: i64,
) -> Result<retrospect::Model, AppError> {
    // 1. 회고 존재 여부 확인
    let retrospect_model = retrospect::Entity::find_by_id(retrospect_id)
        .one(&state.db).await
        .map_err(|e| AppError::InternalError(e.to_string()))?
        .ok_or_else(|| {
            AppError::RetrospectNotFound(
                "존재하지 않는 회고이거나 접근 권한이 없습니다.".to_string(),
            )
        })?;

    // 2. 팀 멤버십 확인
    let is_member = member_team::Entity::find()
        .filter(member_team::Column::MemberId.eq(user_id))
        .filter(member_team::Column::TeamId.eq(retrospect_model.team_id))
        .one(&state.db).await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    if is_member.is_none() {
        return Err(AppError::RetrospectNotFound(
            "존재하지 않는 회고이거나 접근 권한이 없습니다.".to_string(),
        ));
    }

    Ok(retrospect_model)
}
```

### 보안 포인트

- 에러 메시지가 `"존재하지 않는 회고이거나 접근 권한이 없습니다."`로 통일됨
- 회고 미존재(L332-335)와 팀 비멤버(L346-348) 모두 `AppError::RetrospectNotFound` (HTTP 404)
- 403 Forbidden 대신 404 Not Found를 반환하여 리소스 존재 여부 자체를 숨김
- 이 패턴은 GitHub API 등 주요 서비스에서도 사용하는 표준 보안 관행

### 스펙과의 차이

API 스펙(`docs/api-specs/013-retrospect-delete.md`)에서는 404 메시지가 `"존재하지 않는 회고입니다."`이지만, 실제 구현에서는 보안 패턴에 따라 `"존재하지 않는 회고이거나 접근 권한이 없습니다."`로 통합되었다. 이는 의도된 차이로, 구현이 스펙보다 보안적으로 더 강화된 형태이다.

또한 스펙에는 Owner/Creator만 삭제 가능하다고 명시하며 403 에러를 정의하고 있으나, 현재 구현에서는 팀 멤버라면 누구나 삭제 가능하다. `RetroDeleteAccessDenied` 에러 타입은 정의만 되어 있고(`error.rs:120-124`, `#[allow(dead_code)]`), DB 스키마에 `created_by`와 `member_team.role` 필드가 추가된 후 활성화될 예정이다.

### 재사용성

이 헬퍼 함수는 다른 API에서도 동일하게 사용된다:
- API-014 참석자 등록 (`service.rs:362`)
- API-016 임시 저장 (`service.rs:437`)
- API-020 답변 조회 (`service.rs:1056`)
- API-021 내보내기 (`service.rs:1863`)

---

## SeaORM delete_many 패턴

### 개념

SeaORM의 `Entity::delete_many()`는 SQL의 `DELETE FROM ... WHERE ...`에 대응하는 벌크 삭제 메서드다. 단일 모델의 `model.delete()`와 달리, 필터 조건에 맞는 여러 행을 한 번의 쿼리로 삭제한다.

### delete_many vs model.delete()

| 방식 | 용도 | SQL |
|------|------|-----|
| `Entity::delete_many().filter(...).exec()` | 조건 기반 벌크 삭제 | `DELETE FROM table WHERE condition` |
| `model.delete(&db)` | 단일 모델 삭제 (PK 기반) | `DELETE FROM table WHERE id = ?` |

### 코드 예시

**소스**: `service.rs:1175-1179` (delete_many)

```rust
let comments_deleted = response_comment::Entity::delete_many()
    .filter(response_comment::Column::ResponseId.is_in(response_ids.clone()))
    .exec(&txn)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
// comments_deleted.rows_affected로 삭제된 행 수 확인 가능
```

**소스**: `service.rs:1227-1230` (model.delete)

```rust
retrospect_model
    .delete(&txn)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```

### 반환값

`delete_many().exec()`은 `DeleteResult`를 반환하며, `rows_affected` 필드로 실제 삭제된 행 수를 확인할 수 있다. 이 값은 로깅에 활용된다 (`service.rs:1195-1202`).

---

## 조건부 회고방 삭제

### 개념

하나의 회고방(`retro_room`)에 여러 회고(`retrospect`)가 속할 수 있다. 회고를 삭제할 때, 해당 회고방을 참조하는 다른 회고가 있으면 회고방을 유지해야 한다.

### 코드 구현

**소스**: `service.rs:1232-1264`

```rust
// 11. 회고방 삭제 (같은 room을 참조하는 다른 회고가 없는 경우에만)
let other_retro_count = retrospect::Entity::find()
    .filter(retrospect::Column::RetrospectRoomId.eq(retrospect_room_id))
    .count(&txn)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

let (member_retro_rooms_deleted, room_deleted) = if other_retro_count == 0 {
    // 회고방을 참조하는 다른 회고가 없으므로 멤버-회고방 매핑과 회고방 모두 삭제
    let member_retro_rooms_deleted = member_retro_room::Entity::delete_many()
        .filter(member_retro_room::Column::RetrospectRoomId.eq(retrospect_room_id))
        .exec(&txn)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    let room_deleted = retro_room::Entity::delete_many()
        .filter(retro_room::Column::RetrospectRoomId.eq(retrospect_room_id))
        .exec(&txn)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    (
        member_retro_rooms_deleted.rows_affected,
        room_deleted.rows_affected,
    )
} else {
    warn!(
        retrospect_room_id = retrospect_room_id,
        other_retro_count = other_retro_count,
        "회고방을 공유하는 다른 회고가 존재하여 회고방 삭제를 건너뜁니다"
    );
    (0, 0)
};
```

**참고**: 실제 구현에서는 삭제 결과를 `(member_retro_rooms_deleted, room_deleted)` 튜플로 반환하여 이후 최종 로그(`service.rs:1271-1279`)에서 삭제된 행 수를 기록한다.

### 주의사항

- `count()`는 Step 6에서 현재 회고를 이미 삭제한 이후에 실행되므로, 결과가 0이면 참조하는 회고가 전혀 없음을 의미
- `warn!` 매크로로 건너뛴 이유를 운영 로그에 기록

---

## select_only + into_tuple 최적화

### 개념

전체 모델이 필요 없고 특정 컬럼만 필요할 때, `select_only().column()`과 `into_tuple()`을 사용하여 메모리 사용량과 네트워크 전송량을 줄인다.

### 코드 구현

**소스**: `service.rs:1164-1171`

```rust
let response_ids: Vec<i64> = response::Entity::find()
    .filter(response::Column::RetrospectId.eq(retrospect_id))
    .select_only()                              // SELECT * 대신
    .column(response::Column::ResponseId)       // SELECT response_id만
    .into_tuple()                               // Model 대신 i64로 역직렬화
    .all(&txn)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```

### 효과

- 전체 `response::Model`을 가져오면 모든 컬럼(content, question_order, retrospect_id 등)이 포함됨
- ID만 필요한 상황에서 `select_only().column().into_tuple()`은 `SELECT response_id FROM response WHERE retrospect_id = ?` 만 실행
- 응답이 많을수록(수십~수백 개) 성능 차이가 커짐

---

## Spring 프레임워크 비교

### Transaction 관리
- **Spring**: `@Transactional` 어노테이션을 통한 AOP 기반의 선언적 트랜잭션 관리.
- **Rust (SeaORM)**: `db.begin()`과 `txn.commit()`을 사용하는 명시적 트랜잭션 관리. `TransactionTrait`을 통해 제어하며, 스코프 기반의 RAII 패턴으로 자동 롤백을 지원한다.

### JPA vs SeaORM
- **JPA (Hibernate)**: Entity 객체 중심. 영속성 컨텍스트(1차 캐시)가 변경 감지(Dirty Checking)를 수행하여 `save()` 호출 없이도 업데이트 가능.
- **SeaORM**: `ActiveModel` 패턴 사용. 변경사항을 `ActiveModel`에 Set하고 명시적으로 `.update()`나 `.insert()`를 호출해야 함. 영속성 컨텍스트가 없으므로 쿼리가 즉시 실행됨.

### Optional vs Option / Exception vs Result
- **Java**: `Optional<T>`로 Null Safety 처리, Exception(`try-catch`)으로 에러 흐름 제어.
- **Rust**: `Option<T>` (`Some`/`None`)로 값의 부재 표현, `Result<T, E>` (`Ok`/`Err`)로 성공/실패 표현. 예외를 던지는 대신 값으로 리턴하며 `?` 연산자로 전파.
