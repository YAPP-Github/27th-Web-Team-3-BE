# 핵심 개념: 회고 참여자 등록 (API-014)

## 1. 중복 체크와 409 Conflict

### 이중 방어 전략

이 API는 중복 참석을 방지하기 위해 **두 단계**의 중복 체크를 수행합니다.

#### 1단계: 애플리케이션 레벨 체크 (service.rs:372~384)

```rust
let existing_participant = member_retro::Entity::find()
    .filter(member_retro::Column::MemberId.eq(user_id))
    .filter(member_retro::Column::RetrospectId.eq(retrospect_id))
    .one(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

if existing_participant.is_some() {
    return Err(AppError::ParticipantDuplicate(
        "이미 참석자로 등록되어 있습니다.".to_string(),
    ));
}
```

- DB에 SELECT 쿼리를 날려서 `(member_id, retrospect_id)` 조합이 이미 존재하는지 확인합니다.
- 대부분의 중복 요청을 빠르게 거부할 수 있습니다.

#### 2단계: DB 레벨 유니크 제약 (service.rs:408~419)

```rust
let inserted = member_retro_model.insert(&state.db).await.map_err(|e| {
    let error_msg = e.to_string().to_lowercase();
    if error_msg.contains("duplicate")
        || error_msg.contains("unique")
        || error_msg.contains("constraint")
    {
        AppError::ParticipantDuplicate("이미 참석자로 등록되어 있습니다.".to_string())
    } else {
        AppError::InternalError(e.to_string())
    }
})?;
```

- INSERT 시 DB 유니크 제약 위반이 발생하면 에러 메시지 문자열을 검사하여 `ParticipantDuplicate`로 매핑합니다.
- `"duplicate"`, `"unique"`, `"constraint"` 키워드 포함 여부로 판단합니다.

#### 이중 방어가 필요한 이유

1단계의 SELECT와 INSERT 사이에 **동시성 이슈(Race Condition)**가 발생할 수 있습니다. 두 요청이 동시에 SELECT를 통과한 뒤 INSERT를 시도하면, 1단계만으로는 중복을 방지할 수 없습니다. 따라서 DB 유니크 제약이 최종 방어선 역할을 합니다.

### HTTP 409 Conflict의 의미

`409 Conflict`는 요청이 리소스의 현재 상태와 충돌할 때 사용되는 상태 코드입니다. 이 API에서는 "동일 유저가 동일 회고에 이미 참석 등록된 상태"가 충돌 조건에 해당합니다.

**소스**: `error.rs:72~73, 190, 231`

```rust
// 에러 정의
ParticipantDuplicate(String),  // error.rs:73

// 에러 코드 매핑
AppError::ParticipantDuplicate(_) => "RETRO4091",  // error.rs:190

// HTTP 상태 코드 매핑
AppError::ParticipantDuplicate(_) => StatusCode::CONFLICT,  // error.rs:231
```

---

## 2. 시간 기반 검증 (과거 회고 거부)

### KST 시간 변환 로직

**소스**: `service.rs:365~370`

```rust
let now_kst = Utc::now().naive_utc() + chrono::Duration::hours(9);
if retrospect_model.start_time <= now_kst {
    return Err(AppError::RetrospectAlreadyStarted(
        "이미 시작되었거나 종료된 회고에는 참석할 수 없습니다.".to_string(),
    ));
}
```

#### 시간 비교 과정

1. `Utc::now()` -- 현재 UTC 시각을 `DateTime<Utc>` 타입으로 가져옵니다.
2. `.naive_utc()` -- 타임존 정보를 제거하여 `NaiveDateTime`으로 변환합니다.
3. `+ chrono::Duration::hours(9)` -- KST(UTC+9)로 변환합니다.
4. `retrospect_model.start_time` -- DB에 저장된 회고 시작 시간 (`NaiveDateTime` 타입).
5. `<=` 비교 -- 현재 KST 시각이 회고 시작 시간 이상이면 "이미 시작된 회고"로 판단합니다.

#### 주의 사항

- `<=` (이하) 비교를 사용하므로, 시작 시간과 정확히 같은 시점에도 참석이 불가합니다.
- DB의 `start_time` 필드는 `NaiveDateTime` 타입이며, KST 기준으로 저장되어 있다고 가정합니다.
- `chrono::Duration::hours(9)` 방식의 타임존 변환은 서머타임이 없는 한국 표준시(KST)에 적합합니다.

### RETRO4002 에러 코드

**소스**: `error.rs:75~76, 191, 232`

```rust
// 에러 정의
RetrospectAlreadyStarted(String),  // error.rs:76

// 에러 코드 매핑
AppError::RetrospectAlreadyStarted(_) => "RETRO4002",  // error.rs:191

// HTTP 상태 코드 매핑
AppError::RetrospectAlreadyStarted(_) => StatusCode::BAD_REQUEST,  // error.rs:232
```

---

## 3. member_retro 엔티티 관계

### 엔티티 구조

**소스**: `member_retro.rs:23~33`

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "member_retro")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub member_retro_id: i64,       // PK (auto increment)
    pub personal_insight: Option<String>,  // AI 분석 후 개인 인사이트 (초기값: None)
    pub member_id: i64,             // FK -> member 테이블
    pub retrospect_id: i64,         // FK -> retrospect 테이블
    pub status: RetrospectStatus,   // 참여 상태 (DRAFT/SUBMITTED/ANALYZED)
    pub submitted_at: Option<DateTime>,  // 제출 시각 (초기값: None)
}
```

### 관계도

```
member (회원)
  |
  |-- member_team (팀 소속) --> team (팀)
  |                              |
  |-- member_retro (회고 참여) --> retrospect (회고) --> team (팀)
```

- `member_retro`는 `member`와 `retrospect` 사이의 **다대다(N:M) 관계**를 나타내는 중간 테이블입니다.
- 하나의 멤버는 여러 회고에 참여할 수 있고, 하나의 회고는 여러 멤버가 참여할 수 있습니다.

### SeaORM Relation 정의

**소스**: `member_retro.rs:35~53`

```rust
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::member::Entity",
        from = "Column::MemberId",
        to = "super::member::Column::MemberId",
    )]
    Member,
    #[sea_orm(
        belongs_to = "crate::domain::retrospect::entity::retrospect::Entity",
        from = "Column::RetrospectId",
        to = "crate::domain::retrospect::entity::retrospect::Column::RetrospectId",
    )]
    Retrospect,
}
```

- `belongs_to` 관계로 `member`와 `retrospect` 엔티티에 각각 연결됩니다.
- `on_update = "NoAction"`, `on_delete = "NoAction"`으로 설정되어 있어, 연관 레코드 변경/삭제 시 자동 전파가 이루어지지 않습니다.

### 참석 등록 시 초기값

**소스**: `service.rs:401~406`

```rust
let member_retro_model = member_retro::ActiveModel {
    member_id: Set(user_id),
    retrospect_id: Set(retrospect_id),
    personal_insight: Set(None),
    ..Default::default()
};
```

| 필드 | 초기값 | 설명 |
|------|--------|------|
| `member_retro_id` | auto increment | DB 자동 생성 |
| `member_id` | `user_id` (JWT에서 추출) | 참석 유저 |
| `retrospect_id` | Path 파라미터 값 | 참석 대상 회고 |
| `personal_insight` | `None` | AI 분석 전이므로 빈 값 |
| `status` | `DRAFT` (Default) | 초기 상태 |
| `submitted_at` | `None` (Default) | 아직 미제출 |

---

## 4. 보안 설계: 404 통합 처리

**소스**: `service.rs:345~349`

```rust
if is_member.is_none() {
    return Err(AppError::RetrospectNotFound(
        "존재하지 않는 회고이거나 접근 권한이 없습니다.".to_string(),
    ));
}
```

회고가 존재하지만 접근 권한이 없는 경우에도 `404 Not Found`를 반환합니다. 이는 보안상의 이유로, 권한 없는 사용자에게 해당 회고의 존재 여부를 노출하지 않기 위한 설계입니다. `403 Forbidden`을 반환하면 "리소스는 존재하지만 접근 권한이 없다"는 정보가 노출될 수 있기 때문입니다.

---

## 5. 닉네임 추출 로직

**소스**: `service.rs:393~398`

```rust
let nickname = member_model
    .email
    .split('@')
    .next()
    .unwrap_or(&member_model.email)
    .to_string();
```

이메일 주소에서 `@` 앞부분을 추출하여 닉네임으로 사용합니다. 예를 들어 `jason@example.com`에서 `jason`을 추출합니다. `@`가 없는 예외적인 경우 전체 이메일 문자열을 그대로 사용합니다.
