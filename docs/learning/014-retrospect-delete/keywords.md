# [API-013] 학습 키워드

## TransactionTrait

SeaORM에서 트랜잭션을 시작하기 위한 trait. `DatabaseConnection`에 구현되어 있으며, `begin()` 메서드로 `DatabaseTransaction` 객체를 반환한다.

```rust
use sea_orm::TransactionTrait;

let txn = state.db.begin().await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```

- **코드 위치**: `service.rs:9` (import), `service.rs:1157-1161` (사용)
- **참고**: `commit()` 또는 `rollback()`을 명시적으로 호출하거나, drop 시 자동 rollback

---

## DeleteMany (delete_many)

SeaORM의 `Entity::delete_many()` 메서드. 조건 기반 벌크 삭제를 수행하며, `DELETE FROM ... WHERE ...` SQL 문을 생성한다. `filter()`로 조건을 지정하고 `exec()`으로 실행한다.

```rust
response_comment::Entity::delete_many()
    .filter(response_comment::Column::ResponseId.is_in(response_ids.clone()))
    .exec(&txn)
    .await?;
```

- **코드 위치**: `service.rs:1175`, `1182`, `1189`, `1206`, `1213`, `1220`, `1241`, `1247`
- **반환값**: `DeleteResult { rows_affected: u64 }` -- 삭제된 행 수 확인 가능

---

## Cascade Delete (애플리케이션 수준)

FK 의존 관계가 있는 여러 테이블을 삭제할 때, 자식 테이블부터 역순으로 삭제하는 패턴. DB의 `ON DELETE CASCADE` 대신 애플리케이션 코드에서 명시적으로 삭제 순서를 제어한다.

- **코드 위치**: `service.rs:1173-1264` (전체 cascade 삭제 로직)
- **삭제 순서**: `response_comment` -> `response_like` -> `member_response` -> `response` -> `retro_reference` -> `member_retro` -> `retrospect` -> `member_retro_room` -> `retro_room`
- **장점**: 삭제 행 수 로깅, 조건부 삭제 등 세밀한 제어 가능

---

## RAII Rollback

Rust의 RAII(Resource Acquisition Is Initialization) 패턴을 활용한 자동 rollback. SeaORM의 `DatabaseTransaction`은 `Drop` trait을 구현하여, `commit()`이 호출되지 않은 채 스코프를 벗어나면 자동으로 rollback을 수행한다.

```rust
let txn = state.db.begin().await?;
// ... 중간에 에러 발생 시 ? 연산자로 함수 조기 리턴
//     → txn이 drop됨 → 자동 rollback
txn.commit().await?;  // 여기까지 도달해야만 커밋
```

- **코드 위치**: `service.rs:1157` (begin), `service.rs:1267` (commit)
- **핵심**: 명시적 rollback 호출이 불필요하며, 에러 경로에서 데이터 정합성이 자동 보장됨

---

## ModelTrait::delete

SeaORM에서 이미 조회된 단일 모델 인스턴스를 PK 기반으로 삭제하는 메서드. `Entity::delete_many()`와 달리 특정 한 행만 삭제한다.

```rust
retrospect_model
    .delete(&txn)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```

- **코드 위치**: `service.rs:1227-1230`
- **import**: `service.rs:8` (`ModelTrait`)
- **참고**: `find_retrospect_for_member`에서 이미 조회된 모델을 재사용하므로 추가 SELECT 없이 바로 DELETE 가능

---

## is_in 필터

SeaORM의 `ColumnTrait::is_in()` 메서드. SQL의 `WHERE column IN (...)` 절을 생성한다. `Vec<T>`를 인자로 받아 여러 값을 한 번에 필터링한다.

```rust
response_comment::Entity::delete_many()
    .filter(response_comment::Column::ResponseId.is_in(response_ids.clone()))
```

- **코드 위치**: `service.rs:1176`, `1183`, `1190`
- **주의**: `response_ids.clone()`을 사용하는 이유는 같은 Vec을 여러 delete_many에서 재사용하기 위함. Rust의 소유권 규칙 때문에 clone 필요.

---

## select_only + into_tuple

SeaORM에서 특정 컬럼만 조회하여 메모리를 절약하는 패턴. `select_only()`로 `SELECT *`를 비활성화하고, `column()`으로 필요한 컬럼만 지정한 뒤, `into_tuple()`로 모델 대신 기본 타입으로 역직렬화한다.

```rust
let response_ids: Vec<i64> = response::Entity::find()
    .filter(response::Column::RetrospectId.eq(retrospect_id))
    .select_only()
    .column(response::Column::ResponseId)
    .into_tuple()
    .all(&txn).await?;
```

- **코드 위치**: `service.rs:1164-1171`
- **생성되는 SQL**: `SELECT response_id FROM response WHERE retrospect_id = ?`
- **참고**: `into_model::<T>()`과 달리 struct 정의 없이 기본 타입만으로 결과를 받을 수 있음

---

## PaginatorTrait::count

SeaORM에서 `SELECT COUNT(*)` 쿼리를 실행하는 메서드. 조건부 삭제 결정을 위해 사용된다.

```rust
let other_retro_count = retrospect::Entity::find()
    .filter(retrospect::Column::RetrospectRoomId.eq(retrospect_room_id))
    .count(&txn)
    .await?;
```

- **코드 위치**: `service.rs:1233-1237`
- **import**: `service.rs:8` (`PaginatorTrait`)
- **용도**: 회고방을 공유하는 다른 회고가 있는지 확인하여 조건부 삭제 결정

---

## AppError::RetrospectNotFound (404 통합 패턴)

존재하지 않는 리소스와 접근 권한이 없는 리소스를 동일한 404 에러로 반환하는 보안 패턴. 공격자가 리소스 존재 여부를 추측하지 못하도록 한다.

```rust
// 회고 미존재
.ok_or_else(|| AppError::RetrospectNotFound("존재하지 않는 회고이거나 접근 권한이 없습니다.".to_string()))?;

// 팀 비멤버
if is_member.is_none() {
    return Err(AppError::RetrospectNotFound("존재하지 않는 회고이거나 접근 권한이 없습니다.".to_string()));
}
```

- **코드 위치**: `service.rs:332-335` (미존재), `service.rs:345-348` (비멤버)
- **에러 코드**: `RETRO4041` (`error.rs:189`)
- **HTTP 상태**: 404 Not Found (`error.rs:230`)
- **스펙과의 차이**: API 스펙의 404 메시지는 `"존재하지 않는 회고입니다."`이지만 구현에서는 보안상 `"존재하지 않는 회고이거나 접근 권한이 없습니다."`로 통합됨

---

## RetroDeleteAccessDenied (미래 확장 -- 스펙과의 주요 차이점)

회고 삭제 권한을 세밀하게 구분하기 위해 정의된 에러 타입. 현재는 `#[allow(dead_code)]`로 미사용 상태이며, `retrospects.created_by` 및 `member_team.role` 스키마 추가 후 활성화 예정이다.

```rust
/// RETRO4031: 회고 삭제 권한 없음 (403)
/// TODO: 현재 미사용. retrospects.created_by / member_team.role 스키마 추가 후
/// 팀 Owner 또는 회고 생성자만 삭제 가능하도록 권한 분기 시 활성화 예정
#[allow(dead_code)]
RetroDeleteAccessDenied(String),
```

- **코드 위치**: `error.rs:120-124` (정의), `error.rs:165` (message), `error.rs:206` (code: RETRO4031), `error.rs:247` (status: 403)
- **현재 상태**: 팀 멤버라면 누구나 삭제 가능. 향후 Owner/Creator만 삭제 가능하도록 변경 예정
- **스펙과의 차이**: API 스펙(`docs/api-specs/013-retrospect-delete.md`)에서는 403 에러와 Owner/Creator 권한 검증을 정의하고 있으나, DB 스키마에 `created_by`와 `role` 필드가 없어 현재 구현에서는 403이 발생하지 않음. `service.rs:1136-1138`의 TODO 주석에 이 사실이 명시되어 있음

---

## tracing 구조화 로깅

`tracing` 크레이트의 `info!`, `warn!` 매크로를 사용한 구조화된 로깅. 키-값 쌍으로 컨텍스트를 기록하여 로그 분석 도구에서 필터링이 가능하다.

```rust
info!(
    retrospect_id = retrospect_id,
    responses_deleted = responses_deleted.rows_affected,
    references_deleted = references_deleted.rows_affected,
    member_retros_deleted = member_retros_deleted.rows_affected,
    "회고 및 연관 데이터 삭제 완료"
);
```

- **코드 위치**: `service.rs:1144-1148` (삭제 요청), `service.rs:1195-1202` (응답 데이터 삭제), `service.rs:1271-1279` (최종 완료)
- **import**: `service.rs:11` (`tracing::{info, warn}`)
