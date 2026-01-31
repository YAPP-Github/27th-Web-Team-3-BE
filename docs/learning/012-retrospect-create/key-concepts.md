# 핵심 개념: 회고 생성 (API-011)

## DTO 검증 (validator 크레이트)

Request DTO에 `#[derive(Validate)]`를 적용하여 선언적으로 검증 규칙을 정의합니다.

**파일**: `src/domain/retrospect/dto.rs:27-68`

```rust
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateRetrospectRequest {
    #[validate(range(min = 1, message = "팀 ID는 1 이상이어야 합니다"))]
    pub team_id: i64,

    #[validate(length(min = 1, max = 20, message = "프로젝트 이름은 1자 이상 20자 이하여야 합니다"))]
    pub project_name: String,

    #[validate(length(min = 10, max = 10, message = "날짜 형식이 올바르지 않습니다."))]
    pub retrospect_date: String,

    #[validate(length(min = 5, max = 5, message = "시간 형식이 올바르지 않습니다."))]
    pub retrospect_time: String,

    pub retrospect_method: RetrospectMethod,  // serde가 enum 역직렬화 담당

    #[validate(
        length(max = 10, message = "참고 URL은 최대 10개까지 등록 가능합니다"),
        custom(function = "validate_reference_url_items")
    )]
    #[serde(default)]
    pub reference_urls: Vec<String>,
}
```

| derive 매크로 | 역할 |
|--------------|------|
| `Deserialize` | JSON → Rust 구조체 변환 (serde) |
| `Validate` | `req.validate()` 호출 가능하게 함 (validator) |
| `ToSchema` | OpenAPI/Swagger 문서 자동 생성 (utoipa) |

- `#[serde(rename_all = "camelCase")]`: JSON 필드명은 camelCase(`teamId`), Rust 필드는 snake_case(`team_id`)
- `#[serde(default)]`: `reference_urls` 필드가 JSON에 없으면 `Vec::default()` (빈 배열) 사용
- `#[validate(custom(function = "validate_reference_url_items"))]`: 커스텀 검증 함수로 개별 URL 길이(2048자) 제한
- `retrospect_method`는 `#[validate]` 없음 — serde의 enum 역직렬화가 이미 유효한 값만 허용

## RetrospectMethod Enum (SeaORM + serde 연동)

DB Enum 타입과 JSON 직렬화/역직렬화를 하나의 Rust enum으로 통합합니다.

**파일**: `src/domain/retrospect/entity/retrospect.rs:7-71`

```rust
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, ToSchema)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "RetrospectMethod")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RetrospectMethod {
    #[sea_orm(string_value = "KPT")]
    Kpt,
    #[sea_orm(string_value = "FOUR_L")]
    FourL,
    // ...
}
```

- `#[serde(rename_all = "SCREAMING_SNAKE_CASE")]`: Rust의 `Kpt` → JSON에서 `"KPT"`, `FourL` → `"FOUR_L"`
- `DeriveActiveEnum`: SeaORM에서 DB의 Enum 컬럼과 자동 매핑
- `#[sea_orm(string_value = "KPT")]`: DB에 저장되는 문자열 값 지정
- `default_questions()`: 각 회고 방식별 5개 질문을 `Vec<&'static str>`로 반환

## SeaORM 트랜잭션 처리

여러 테이블에 대한 삽입을 하나의 트랜잭션으로 묶어 원자성을 보장합니다.

**파일**: `src/domain/retrospect/service.rs:87-183`

```rust
// 트랜잭션 시작
let txn = state.db.begin().await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

// RetroRoom 생성
let room_model = retro_room::ActiveModel { ... };
room_model.insert(&txn).await?;

// Retrospect 생성
let retrospect_model = retrospect::ActiveModel { ... };
let inserted = retrospect_model.insert(&txn).await?;

// Response × 5 생성
for question in method.default_questions() {
    let response_model = response::ActiveModel { ... };
    response_model.insert(&txn).await?;
}

// RetroReference × N 생성
for url in &req.reference_urls {
    let ref_model = retro_reference::ActiveModel { ... };
    ref_model.insert(&txn).await?;
}

// 트랜잭션 커밋
txn.commit().await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```

- `state.db.begin()`: `DatabaseTransaction` 반환
- 모든 insert에 `&txn`을 전달하여 같은 트랜잭션 범위 내에서 실행
- 중간에 `?`로 에러 발생 시 `txn`이 drop되면서 자동 롤백 (RAII 패턴)
- **원자성 보장**: RetroRoom, Retrospect, Response(×5), RetroReference(×N) — 하나라도 실패하면 전체 롤백

## AppError와 IntoResponse (에러 → HTTP 응답 자동 변환)

`AppError` enum이 Axum의 `IntoResponse`를 구현하여 에러가 자동으로 HTTP 응답으로 변환됩니다.

**파일**: `src/utils/error.rs`

```rust
pub enum AppError {
    RetroUrlInvalid(String),     // → "RETRO4006" / 400
    TeamNotFound(String),        // → "TEAM4041" / 404
    TeamAccessDenied(String),    // → "TEAM4031" / 403
    // ...
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_code = self.error_code().to_string();
        let message = self.message();
        error!("Error [{}]: {}", error_code, message);
        let error_response = ErrorResponse::new(error_code, message);
        (status, Json(error_response)).into_response()
    }
}
```

- 각 에러 variant마다 `error_code()`, `status_code()`, `message()` 세 가지가 매핑
- `From<ValidationErrors>`, `From<JsonRejection>` 구현으로 `?` 연산자 사용 시 자동 변환:
  - `req.validate()?` → `ValidationErrors` → `AppError::RetroProjectNameInvalid` 등
  - `Json(req)` 파싱 실패 → `JsonRejection` → `AppError::RetroMethodInvalid`

## chrono를 활용한 날짜/시간 처리

`chrono` 크레이트의 `Naive` 타입으로 날짜와 시간을 파싱하고, KST 기준 미래 시점인지 검증합니다.

**파일**: `src/domain/retrospect/service.rs:228-270`

```rust
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc};

/// 날짜 파싱: "2026-01-20" → NaiveDate
fn validate_and_parse_date(date_str: &str) -> Result<NaiveDate, AppError> {
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest(...))?;

    let today = Utc::now().date_naive();
    if date < today { return Err(...); }
    Ok(date)
}

/// 시간 파싱: "10:00" → NaiveTime
fn validate_and_parse_time(time_str: &str) -> Result<NaiveTime, AppError> {
    NaiveTime::parse_from_str(time_str, "%H:%M")
        .map_err(|_| AppError::BadRequest(...))
}

/// 날짜+시간 결합 후 KST 기준 미래 검증
fn validate_future_datetime(date: NaiveDate, time: NaiveTime) -> Result<(), AppError> {
    let input_datetime = NaiveDateTime::new(date, time);
    let now_kst = Utc::now().naive_utc() + chrono::Duration::hours(9);
    if input_datetime <= now_kst { return Err(...); }
    Ok(())
}
```

| chrono 타입 | 설명 | 예시 값 |
|-------------|------|---------|
| `NaiveDate` | 타임존 없는 날짜 | `2026-01-20` |
| `NaiveTime` | 타임존 없는 시간 | `10:00:00` |
| `NaiveDateTime` | 날짜+시간 (타임존 없음) | `2026-01-20T10:00:00` |
| `DateTime<Utc>` | UTC 기준 날짜+시간 | `Utc::now()` |

- `Naive` 접두사: 타임존 정보가 없는 순수 날짜/시간 타입 (DB 저장에 적합)
- `parse_from_str(str, format)`: 문자열을 `chrono` 타입으로 파싱 (`Result` 반환)
- `Utc::now().date_naive()`: UTC 현재 시각에서 날짜만 `NaiveDate`로 추출
- `Utc::now().naive_utc()`: UTC 현재 시각을 `NaiveDateTime`으로 변환
- `+ chrono::Duration::hours(9)`: KST(UTC+9) 오프셋을 수동 적용 (`chrono-tz` 크레이트 없이 경량 처리)
- `NaiveDateTime::new(date, time)`: `NaiveDate`와 `NaiveTime`을 결합
- SeaORM의 `DateTime` 타입은 내부적으로 `chrono::NaiveDateTime`과 동일 (엔티티 Model의 `start_time`, `created_at`, `updated_at` 필드)

## URL 검증 로직

외부 라이브러리 없이 수동으로 URL 형식을 검증합니다.

**파일**: `src/domain/retrospect/service.rs:187-226`

```rust
fn validate_reference_urls(urls: &[String]) -> Result<(), AppError> {
    // 중복 검증: HashSet으로 O(n) 중복 감지
    let unique_urls: HashSet<_> = urls.iter().collect();
    if unique_urls.len() != urls.len() { ... }

    for url in urls {
        // 길이 검증
        if url.len() > REFERENCE_URL_MAX_LENGTH { ... }

        // 스키마 검증 (if let 패턴 매칭)
        let without_scheme = if let Some(stripped) = url.strip_prefix("https://") {
            stripped
        } else if let Some(stripped) = url.strip_prefix("http://") {
            stripped
        } else {
            return Err(AppError::RetroUrlInvalid("유효하지 않은 URL 형식입니다.".to_string()));
        };

        // 호스트 존재 검증
        if without_scheme.is_empty() || !without_scheme.contains('.') { ... }
    }
}
```

- `HashSet`: 중복 감지를 O(n)으로 수행
- `if let Some(stripped) = ...`: 패턴 매칭으로 스키마 제거 후 나머지를 변수에 바인딩
- 가벼운 검증 전략: `http(s)://` 프리픽스 + `.` 포함 여부만 확인

## SeaORM ActiveModel (Builder 패턴)

`Set()`으로 설정할 필드만 명시하고 나머지는 DB 기본값을 사용합니다.

```rust
let now = Utc::now().naive_utc();  // NaiveDateTime (UTC 기준)
let start_time = NaiveDateTime::new(retrospect_date, retrospect_time);  // 날짜+시간 결합

let model = retrospect::ActiveModel {
    title: Set(req.project_name.clone()),
    team_insight: Set(None),
    retrospect_method: Set(req.retrospect_method.clone()),
    created_at: Set(now),
    updated_at: Set(now),
    start_time: Set(start_time),
    retrospect_room_id: Set(retrospect_room_id),
    team_id: Set(req.team_id),
    ..Default::default()  // 나머지 필드는 기본값(NotSet)
};
```

- `Set()`: 해당 필드에 값을 설정
- `..Default::default()`: 나머지 필드는 `NotSet` (DB 기본값/자동생성 사용)
- PK가 auto-increment인 경우 `NotSet`으로 두면 DB가 자동 생성
- `created_at`, `updated_at`, `start_time` 필드는 모두 `NaiveDateTime` 타입 (SeaORM `DateTime`과 동일)
