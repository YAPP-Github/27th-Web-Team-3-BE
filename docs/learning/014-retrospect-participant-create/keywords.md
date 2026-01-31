# 학습 키워드: 회고 참여자 등록 (API-014)

## 1. ParticipantDuplicate

**정의**: 동일 유저가 동일 회고에 중복으로 참석 등록할 때 발생하는 에러 타입

**소스**: `error.rs:72~73`

```rust
/// RETRO4091: 중복 참석 (409)
ParticipantDuplicate(String),
```

**매핑 정보**:
- 에러 코드: `RETRO4091` (`error.rs:190`)
- HTTP 상태: `409 Conflict` (`error.rs:231`)

**사용 위치**:
- 애플리케이션 레벨 중복 체크: `service.rs:381`
- DB 유니크 제약 위반 시 매핑: `service.rs:415`

**학습 포인트**:
- HTTP 409 Conflict는 리소스의 현재 상태와 요청이 충돌할 때 사용합니다.
- 멱등성(Idempotency) 관점에서, 중복 요청을 에러로 처리하여 클라이언트에 명확한 상태를 알려줍니다.
- 애플리케이션 레벨 + DB 레벨의 이중 방어로 동시성 이슈를 방지합니다.

---

## 2. RetrospectAlreadyStarted

**정의**: 이미 시작되었거나 종료된 회고에 참석을 시도할 때 발생하는 에러 타입

**소스**: `error.rs:75~76`

```rust
/// RETRO4002: 과거 회고 참석 불가 / 답변 누락 (400)
RetrospectAlreadyStarted(String),
```

**매핑 정보**:
- 에러 코드: `RETRO4002` (`error.rs:191`)
- HTTP 상태: `400 Bad Request` (`error.rs:232`)

**사용 위치**: `service.rs:367~369`

**학습 포인트**:
- 시간 기반 비즈니스 규칙의 구현 방법을 보여줍니다.
- 서버 시간 기준으로 검증하므로, 클라이언트 시간 조작에 안전합니다.
- `RETRO4002`는 "과거 회고 참석 불가"와 "답변 누락(`RetroAnswersMissing`)" 두 가지 용도로 공유됩니다 (`error.rs:191~192`).

---

## 3. NaiveDateTime 비교

**정의**: `chrono` 라이브러리의 타임존 정보가 없는 날짜/시간 타입을 사용한 시간 비교 방법

**소스**: `service.rs:365~366`

```rust
let now_kst = Utc::now().naive_utc() + chrono::Duration::hours(9);
if retrospect_model.start_time <= now_kst {
```

**변환 과정**:
1. `Utc::now()` -- `DateTime<Utc>` (UTC 타임존 포함)
2. `.naive_utc()` -- `NaiveDateTime` (타임존 제거)
3. `+ Duration::hours(9)` -- KST 시간으로 보정된 `NaiveDateTime`

**학습 포인트**:
- `NaiveDateTime`은 타임존 정보를 갖지 않으므로, 비교 시 양쪽이 동일한 기준 시간대여야 합니다.
- DB의 `start_time` 필드가 KST 기준으로 저장되어 있으므로, 서버 시간도 KST로 변환하여 비교합니다.
- `<=` 연산자는 `NaiveDateTime`에 `PartialOrd` 트레이트가 구현되어 있어 사용 가능합니다.
- `chrono::Duration::hours(9)` 방식은 고정 오프셋이라 DST(서머타임)가 없는 KST에 적합합니다.

---

## 4. find_retrospect_for_member

**정의**: 회고 조회와 팀 멤버십 확인을 한 번에 수행하는 공통 유틸리티 함수

**소스**: `service.rs:323~352`

```rust
async fn find_retrospect_for_member(
    state: &AppState,
    user_id: i64,
    retrospect_id: i64,
) -> Result<retrospect::Model, AppError>
```

**학습 포인트**:
- 여러 API에서 "회고 조회 + 멤버십 확인" 패턴이 반복되므로 공통 함수로 추출되었습니다.
- API-014, API-018, API-012, API-013, API-020 등에서 재사용됩니다.
- 보안상 "회고 없음"과 "권한 없음"을 동일한 404 에러로 통합 처리합니다.

---

## 5. member_retro (중간 테이블)

**정의**: 멤버와 회고 간의 N:M 관계를 표현하는 중간(junction/pivot) 테이블

**소스**: `member_retro.rs:23~33`

```rust
#[sea_orm(table_name = "member_retro")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub member_retro_id: i64,
    pub personal_insight: Option<String>,
    pub member_id: i64,
    pub retrospect_id: i64,
    pub status: RetrospectStatus,
    pub submitted_at: Option<DateTime>,
}
```

**학습 포인트**:
- 단순 매핑 테이블이 아닌, `status`, `personal_insight`, `submitted_at` 등 추가 정보를 갖는 풍부한 중간 테이블입니다.
- `RetrospectStatus` enum (`DRAFT` / `SUBMITTED` / `ANALYZED`)으로 참여 상태를 관리합니다 (`member_retro.rs:6~21`).
- `ActiveModelBehavior`의 기본 구현을 사용하여 별도의 생명주기 훅이 없습니다 (`member_retro.rs:67`).

---

## 6. SeaORM ActiveModel 패턴

**정의**: SeaORM에서 DB 레코드를 삽입/수정하기 위해 사용하는 가변 모델 패턴

**소스**: `service.rs:401~406`

```rust
let member_retro_model = member_retro::ActiveModel {
    member_id: Set(user_id),
    retrospect_id: Set(retrospect_id),
    personal_insight: Set(None),
    ..Default::default()
};
```

**학습 포인트**:
- `Set(value)`: 해당 필드에 값을 설정합니다.
- `..Default::default()`: 명시하지 않은 필드는 기본값(`NotSet`)으로 설정합니다.
- `NotSet` 상태인 필드는 INSERT 시 DB 기본값이 적용됩니다 (예: `member_retro_id`는 auto increment, `status`는 DB 기본값).
- `insert(&state.db)`를 호출하면 INSERT SQL이 실행되고, 생성된 레코드의 `Model`이 반환됩니다.

---

## 7. CreateParticipantResponse DTO

**정의**: 회고 참석 등록 성공 시 반환되는 응답 데이터 구조체

**소스**: `dto.rs:224~234`

```rust
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateParticipantResponse {
    pub participant_id: i64,
    pub member_id: i64,
    pub nickname: String,
}
```

**학습 포인트**:
- `#[serde(rename_all = "camelCase")]`로 JSON 직렬화 시 camelCase 변환이 자동 적용됩니다.
  - `participant_id` -> `participantId`
  - `member_id` -> `memberId`
- `ToSchema` derive 매크로는 Swagger/OpenAPI 문서 자동 생성을 위한 utoipa 지원입니다.
- `Deserialize`가 없고 `Serialize`만 있으므로, 응답 전용 DTO임을 알 수 있습니다.

---

## 8. AuthUser 미들웨어

**정의**: JWT Bearer 토큰에서 사용자 정보를 추출하는 Axum 미들웨어/Extractor

**소스**: `handler.rs:136`

```rust
pub async fn create_participant(
    user: AuthUser,
    // ...
)
```

**학습 포인트**:
- Axum의 Extractor 패턴을 활용하여 함수 인자로 자동 주입됩니다.
- `user.user_id()?`로 JWT payload의 `sub` 필드에서 `i64` 타입의 사용자 ID를 파싱합니다 (`handler.rs:148`).
- JWT 인증이 실패하면 핸들러 함수가 호출되기 전에 자동으로 401 에러를 반환합니다.
- 이 API는 Request Body가 없으므로, `Path`와 `AuthUser`만으로 요청을 처리합니다.

---

## 9. 에러 메시지 기반 분기 처리

**정의**: DB 에러의 문자열 메시지를 검사하여 적절한 AppError로 매핑하는 패턴

**소스**: `service.rs:408~419`

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

**학습 포인트**:
- SeaORM의 `DbErr`는 DB 벤더에 따라 다양한 형태의 에러를 반환합니다.
- 문자열 포함 검사(`contains`)를 통해 MySQL, PostgreSQL 등 여러 DB에서 공통으로 동작하도록 처리합니다.
- `.to_lowercase()` 변환으로 대소문자 차이에 안전하게 비교합니다.
- 이 패턴은 DB 독립적이지만, 에러 메시지 형식에 의존하므로 DB 버전 변경 시 주의가 필요합니다.
