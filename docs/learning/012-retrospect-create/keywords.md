# 학습 키워드: 회고 생성 (API-011)

## 1. validator 크레이트 (DTO 검증)

Request DTO에 선언적으로 검증 규칙을 적용하는 크레이트.

```rust
// dto.rs:27-68
#[derive(Validate)]
pub struct CreateRetrospectRequest {
    #[validate(range(min = 1))]
    pub team_id: i64,
    #[validate(length(min = 1, max = 20))]
    pub project_name: String,
    #[validate(custom(function = "validate_reference_url_items"))]
    pub reference_urls: Vec<String>,
}
```

- `#[validate(range(min = 1))]`: 숫자 범위 검증
- `#[validate(length(min = 1, max = 20))]`: 문자열 길이 검증
- `#[validate(custom(function = "..."))]`: 커스텀 검증 함수 연결
- `req.validate()?`로 호출, 실패 시 `ValidationErrors` 반환

## 2. serde default 어트리뷰트

JSON에 필드가 없을 때 기본값을 사용하는 serde 기능.

```rust
// dto.rs:64
#[serde(default)]
pub reference_urls: Vec<String>,  // JSON에 없으면 빈 배열
```

- `#[serde(default)]`: `Default::default()` 호출 (`Vec` → 빈 배열, `Option` → `None`)
- `#[serde(default = "함수명")]`: 커스텀 기본값 함수 지정 가능
- 선택적 필드에 `Option<T>` 대신 사용하면 null 처리 없이 깔끔

## 3. SeaORM DeriveActiveEnum (DB Enum 매핑)

Rust enum을 DB의 Enum 타입과 자동으로 매핑하는 SeaORM derive 매크로.

```rust
// entity/retrospect.rs:7-71
#[derive(DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "RetrospectMethod")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RetrospectMethod {
    #[sea_orm(string_value = "KPT")]
    Kpt,
    #[sea_orm(string_value = "FOUR_L")]
    FourL,
}
```

- `rs_type = "String"`: Rust 측에서 String으로 표현
- `db_type = "Enum"`: DB 측에서 Enum 타입
- `string_value`: 각 variant의 DB 저장값
- serde와 함께 사용하면 JSON ↔ Rust ↔ DB 3단 변환 자동 처리

## 4. SeaORM 트랜잭션 (begin/commit)

여러 DB 작업을 하나의 원자적 단위로 묶는 패턴.

```rust
// service.rs:87-183
let txn = state.db.begin().await?;

// 모든 insert에 &txn 전달
room_model.insert(&txn).await?;
retrospect_model.insert(&txn).await?;
// ...

txn.commit().await?;
```

- `begin()`: `DatabaseTransaction` 반환
- `&txn`을 `&DatabaseConnection` 대신 전달하여 트랜잭션 범위 지정
- `commit()` 호출 전에 에러 발생하면 `txn` drop 시 자동 롤백 (RAII)
- 명시적 rollback 호출 불필요

## 5. RAII (Resource Acquisition Is Initialization)

리소스의 생명주기를 변수의 스코프에 바인딩하는 Rust 핵심 패턴.

```rust
let txn = state.db.begin().await?;  // 리소스 획득 (트랜잭션 시작)
// ... 작업 수행 ...
txn.commit().await?;                // 명시적 완료

// commit() 전에 에러 발생 → txn이 drop됨 → 자동 rollback
```

- 트랜잭션 외에도 파일 핸들, 뮤텍스 락, 네트워크 커넥션 등에 적용
- Rust에서는 GC 없이 스코프 기반 소멸자(`Drop` trait)로 자원 정리
- `?` 연산자와 결합하면 에러 시 자동으로 자원 정리 + 에러 전파

## 6. `?` 연산자 + From trait 체인

에러 타입을 자동 변환하면서 전파하는 Rust의 핵심 에러 처리 패턴.

```
req.validate()?
  → ValidationErrors 발생
  → From<ValidationErrors> for AppError 호출
  → AppError::RetroProjectNameInvalid 반환
  → IntoResponse for AppError
  → HTTP 400 + {"code": "RETRO4001", ...}
```

- `?` 하나로 검증 실패 → HTTP 에러 응답까지 자동 변환
- `From` trait을 구현하면 `?`가 자동으로 타입 변환 수행
- 여러 에러 타입을 하나의 `AppError`로 통합 가능

## 7. `&'static str` (정적 문자열 참조)

컴파일 타임에 바이너리에 포함된 문자열 리터럴에 대한 참조.

```rust
// entity/retrospect.rs
pub fn default_questions(&self) -> Vec<&'static str> {
    match self {
        Self::Kpt => vec![
            "Keep: 잘한 점은 무엇인가요?",
            "Problem: 문제점은 무엇인가요?",
            // ...
        ],
    }
}
```

- `'static` 라이프타임: 프로그램 전체 실행 기간 동안 유효
- 문자열 리터럴은 자동으로 `&'static str`
- `String`과 달리 힙 할당 없이 바이너리의 읽기 전용 메모리 참조
- 상수성 데이터에 적합 (질문 템플릿, 에러 메시지 등)

## 8. HashSet을 이용한 중복 감지

컬렉션에서 중복 요소를 O(n)으로 감지하는 패턴.

```rust
// service.rs:190-192
let unique_urls: HashSet<_> = urls.iter().collect();
if unique_urls.len() != urls.len() {
    return Err(AppError::RetroUrlInvalid("중복된 URL이 있습니다.".to_string()));
}
```

- `iter().collect()`: 이터레이터를 HashSet으로 수집 (중복 자동 제거)
- 원본 길이와 HashSet 길이 비교로 중복 존재 여부 판단
- O(n) 시간 복잡도 (이중 루프 O(n²) 대비 효율적)

## 9. strip_prefix + if let 패턴 매칭 (문자열 접두사 제거)

문자열에서 특정 접두사를 제거하고 나머지를 반환하는 표준 라이브러리 메서드. 실제 구현에서는 `if let` 패턴 매칭으로 처리합니다.

```rust
// service.rs (실제 구현)
let without_scheme = if let Some(stripped) = url.strip_prefix("https://") {
    stripped
} else if let Some(stripped) = url.strip_prefix("http://") {
    stripped
} else {
    return Err(AppError::RetroUrlInvalid("유효하지 않은 URL 형식입니다.".to_string()));
};
```

- `strip_prefix()`: `Option<&str>` 반환 (접두사가 없으면 `None`)
- `if let Some(stripped) = ...`: 패턴 매칭으로 `Some`인 경우 내부 값을 `stripped`에 바인딩
- `else if let`: 첫 번째 접두사가 없을 때 두 번째 접두사 시도
- `else`: 두 접두사 모두 없으면 에러 반환

## 10. Axum Extractor 순서

Axum 핸들러에서 extractor의 선언 순서가 중요합니다.

```rust
pub async fn create_retrospect(
    user: AuthUser,                          // (1) 헤더에서 JWT 추출
    State(state): State<AppState>,           // (2) 공유 상태 추출
    Json(req): Json<CreateRetrospectRequest>,// (3) 바디 소비 (마지막)
) -> Result<...>
```

- `Body`를 소비하는 extractor (`Json`, `Form`)는 반드시 마지막에 위치
- `FromRequestParts` 구현체 (`AuthUser`, `State`, `Path`)는 순서 무관
- `FromRequest` 구현체 (`Json`)는 Body를 소비하므로 하나만 가능

## 11. UUID를 활용한 초대 URL 생성

고유한 초대 링크를 위해 UUID v4를 `INVITATION_BASE_URL` 환경변수와 조합합니다.

```rust
// service.rs:96-102 (실제 구현)
let base_url = std::env::var("INVITATION_BASE_URL")
    .unwrap_or_else(|_| "https://retro.example.com".to_string());
let invitation_url = format!(
    "{}/room/{}",
    base_url.trim_end_matches('/'),
    uuid::Uuid::new_v4()
);

let retro_room_model = retro_room::ActiveModel {
    invitation_url: Set(invitation_url),
    ..Default::default()
};
```

- `Uuid::new_v4()`: 랜덤 기반 UUID 생성 (충돌 확률 무시 가능)
- `std::env::var()`: 환경변수 조회 (`Result` 반환)
- `unwrap_or_else()`: 환경변수 미설정 시 기본값 사용
- `trim_end_matches('/')`: base URL 끝의 슬래시 제거하여 중복 방지
- 최종 URL 예시: `https://retro.example.com/room/550e8400-e29b-41d4-a716-446655440000`

## 12. 테스트 헬퍼 함수 + struct spread

테스트에서 기본 유효 요청을 만들고 특정 필드만 오버라이드하는 패턴.

```rust
fn create_valid_request() -> CreateRetrospectRequest {
    CreateRetrospectRequest {
        team_id: 1,
        project_name: "테스트 프로젝트".to_string(),
        // ... 모든 필드에 유효한 기본값
    }
}

#[test]
fn should_fail_validation_when_project_name_is_empty() {
    let request = CreateRetrospectRequest {
        project_name: "".to_string(),
        ..create_valid_request()  // 나머지는 기본값
    };
    // ...
}
```

- `..create_valid_request()`: struct update syntax로 나머지 필드 채움
- 테스트마다 검증 대상 필드만 변경하여 의도 명확화
- 테스트 코드 중복 최소화

## 13. chrono 크레이트 (날짜/시간 처리)

Rust의 표준 날짜/시간 처리 크레이트. 이 프로젝트에서는 `Naive` 타입(타임존 없음)을 사용하여 날짜와 시간을 파싱하고, KST 오프셋을 수동으로 적용합니다.

```rust
// service.rs:2-3, 228-270
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc};

// 문자열 → NaiveDate 파싱
let date = NaiveDate::parse_from_str("2026-01-20", "%Y-%m-%d")?;

// 문자열 → NaiveTime 파싱
let time = NaiveTime::parse_from_str("10:00", "%H:%M")?;

// 날짜+시간 결합 → NaiveDateTime
let datetime = NaiveDateTime::new(date, time);  // 2026-01-20T10:00:00

// UTC 현재 시각 → NaiveDateTime
let now_utc = Utc::now().naive_utc();

// KST 현재 시각 (UTC + 9시간)
let now_kst = Utc::now().naive_utc() + chrono::Duration::hours(9);

// UTC 기준 오늘 날짜 → NaiveDate
let today = Utc::now().date_naive();
```

- `NaiveDate` / `NaiveTime` / `NaiveDateTime`: 타임존 없는 순수 날짜/시간 타입
- `parse_from_str(str, format)`: 포맷 문자열로 파싱 (`%Y`=연도, `%m`=월, `%d`=일, `%H`=시, `%M`=분)
- `Utc::now()`: UTC 기준 현재 `DateTime<Utc>` 반환
- `.naive_utc()`: `DateTime<Utc>` → `NaiveDateTime` 변환 (타임존 정보 제거)
- `.date_naive()`: `DateTime<Utc>` → `NaiveDate` 변환 (시간 제거)
- `chrono::Duration::hours(9)`: 9시간 분량의 Duration 생성 (KST 오프셋)
- Cargo.toml에서 `chrono = { version = "0.4", features = ["serde"] }`로 serde 직렬화 지원 활성화

## 14. SeaORM DateTime과 chrono NaiveDateTime

SeaORM의 `DateTime` 타입은 내부적으로 `chrono::NaiveDateTime`과 동일합니다.

```rust
// entity/retrospect.rs:73-86
#[derive(DeriveEntityModel)]
#[sea_orm(table_name = "retrospects")]
pub struct Model {
    pub created_at: DateTime,   // = chrono::NaiveDateTime
    pub updated_at: DateTime,   // = chrono::NaiveDateTime
    pub start_time: DateTime,   // = chrono::NaiveDateTime
    // ...
}
```

- `DateTime`은 SeaORM이 re-export한 `chrono::NaiveDateTime`
- DB에서 읽어온 `Model`의 날짜/시간 필드도 `NaiveDateTime`으로 접근
- `model.start_time.format("%Y-%m-%d").to_string()`: `NaiveDateTime` → `"2026-01-20"` 포맷팅
- `model.start_time.format("%H:%M").to_string()`: `NaiveDateTime` → `"10:00"` 포맷팅
- DTO 변환 시 `format()` 메서드로 API 응답에 맞는 문자열로 변환 (dto.rs의 `impl From<RetrospectModel>` 참조)
