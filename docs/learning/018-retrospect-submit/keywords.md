# 학습 키워드: 회고 최종 제출 (API-017)

## Rust / 프레임워크 키워드

### 1. `RetrospectStatus` enum

**소스**: `codes/server/src/domain/member/entity/member_retro.rs:11~21`

```rust
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, ToSchema)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "RetrospectStatus")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RetrospectStatus {
    #[sea_orm(string_value = "DRAFT")]
    Draft,
    #[sea_orm(string_value = "SUBMITTED")]
    Submitted,
    #[sea_orm(string_value = "ANALYZED")]
    Analyzed,
}
```

- `DeriveActiveEnum`: SeaORM에서 DB enum 타입과 Rust enum을 매핑
- `sea_orm(string_value = "...")`: DB에 저장되는 실제 문자열 값 지정
- `serde(rename_all = "SCREAMING_SNAKE_CASE")`: JSON 직렬화 시 대문자 스네이크 케이스 사용 (예: `SUBMITTED`)
- 상태 머신의 각 상태를 타입 안전하게 표현

---

### 2. `AppError::RetroAlreadySubmitted`

**소스**: `codes/server/src/utils/error.rs:88, 154, 195, 236`

```rust
RetroAlreadySubmitted(String),  // error.rs:88 - variant 정의
```

| 속성 | 값 |
|------|-----|
| 에러 코드 | `RETRO4033` (error.rs:195) |
| HTTP 상태 | `403 FORBIDDEN` (error.rs:236) |
| 메시지 | `"이미 제출이 완료된 회고입니다."` |

- 이미 `SUBMITTED` 또는 `ANALYZED` 상태인 회고에 재제출 시도 시 발생
- `403 Forbidden`을 사용하는 이유: 리소스는 존재하지만 해당 작업이 금지된 상태

---

### 3. `AppError::RetroAnswersMissing`

**소스**: `codes/server/src/utils/error.rs:79, 151, 192, 233`

```rust
RetroAnswersMissing(String),  // error.rs:79
```

| 속성 | 값 |
|------|-----|
| 에러 코드 | `RETRO4002` (error.rs:192) |
| HTTP 상태 | `400 BAD_REQUEST` (error.rs:233) |
| 메시지 | `"모든 질문에 대한 답변이 필요합니다."` |

발생 조건:
- `answers` 배열의 길이가 5가 아닌 경우 (service.rs:1545)
- `questionNumber` 1~5가 모두 존재하지 않는 경우 (service.rs:1554)

---

### 4. `AppError::RetroAnswerWhitespaceOnly`

**소스**: `codes/server/src/utils/error.rs:85, 153, 194, 235`

```rust
RetroAnswerWhitespaceOnly(String),  // error.rs:85
```

| 속성 | 값 |
|------|-----|
| 에러 코드 | `RETRO4007` (error.rs:194) |
| HTTP 상태 | `400 BAD_REQUEST` (error.rs:235) |
| 메시지 | `"답변 내용은 공백만으로 구성될 수 없습니다."` |

---

### 5. `AppError::RetroAnswerTooLong`

**소스**: `codes/server/src/utils/error.rs:82, 152, 193, 234`

```rust
RetroAnswerTooLong(String),  // error.rs:82
```

| 속성 | 값 |
|------|-----|
| 에러 코드 | `RETRO4003` (error.rs:193) |
| HTTP 상태 | `400 BAD_REQUEST` (error.rs:234) |
| 메시지 | `"답변은 1,000자를 초과할 수 없습니다."` |

---

### 6. `trim()`

**소스**: `service.rs:1563`

```rust
if answer.content.trim().is_empty() { ... }
```

- `str::trim()`: 문자열 양쪽 끝의 Unicode 공백 문자를 제거한 슬라이스(`&str`)를 반환
- Unicode White_Space 속성에 해당하는 문자들: 공백(` `), 탭(`\t`), 개행(`\n`), 캐리지 리턴(`\r`) 등
- 원본 문자열을 변경하지 않음 (불변 참조 반환)
- 이 API에서는 검증용으로만 사용되며, 실제 저장 시에는 trim하지 않고 원본 그대로 저장

---

### 7. `chars().count()`

**소스**: `service.rs:1570`

```rust
if answer.content.chars().count() > 1000 { ... }
```

- `chars()`: 문자열을 유니코드 스칼라 값(char) 단위 이터레이터로 변환
- `count()`: 이터레이터의 원소 수를 반환
- `len()`과의 차이: `len()`은 바이트 수, `chars().count()`는 문자 수
- 예: `"한글".len()` = 6 (UTF-8 기준 한글 1자 = 3바이트), `"한글".chars().count()` = 2

---

### 8. `ActiveModel` update 패턴

**소스**: `service.rs:652~658`

```rust
let mut active: response::ActiveModel = response_model.clone().into();
active.content = Set(answer.content.clone());
active.updated_at = Set(now);
active.update(&txn).await...;
```

- `Model` -> `ActiveModel` 변환: `.into()`를 통해 기존 DB 레코드를 수정 가능한 형태로 변환
- `Set(value)`: SeaORM의 `ActiveValue`로, 해당 필드를 지정한 값으로 업데이트하겠다는 의미
- `update(&txn)`: 트랜잭션 컨텍스트 내에서 UPDATE 쿼리 실행
- `clone()` 사용 이유: 원본 모델의 소유권을 유지하면서 ActiveModel을 생성

---

### 9. `lock_exclusive()`

**소스**: `service.rs:602`

```rust
.lock_exclusive()
```

- SeaORM의 쿼리 빌더 메서드로, SQL의 `SELECT ... FOR UPDATE`를 생성
- 해당 행에 배타적 잠금(exclusive lock)을 설정
- 트랜잭션이 커밋되거나 롤백될 때까지 다른 트랜잭션이 해당 행에 접근 불가
- 동시 제출 경쟁 조건(race condition) 방지에 사용

---

### 10. `HashSet` 비교를 통한 완전성 검증

**소스**: `service.rs:1552~1558`

```rust
let question_numbers: HashSet<i32> = answers.iter().map(|a| a.question_number).collect();
let expected: HashSet<i32> = (1..=5).collect();
if question_numbers != expected { ... }
```

- `HashSet`의 `!=` 비교: 두 집합의 원소가 정확히 동일한지 확인
- `(1..=5)`: 1부터 5까지의 닫힌 범위(inclusive range)
- `.collect()`: 이터레이터를 `HashSet`으로 수집
- 이 패턴으로 중복 번호나 범위 밖 번호를 동시에 감지 가능

---

## DTO 키워드

### 11. `SubmitRetrospectRequest`

**소스**: `dto.rs:139~144`

```rust
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubmitRetrospectRequest {
    pub answers: Vec<SubmitAnswerItem>,
}
```

- `Deserialize`: JSON -> Rust 구조체 역직렬화
- `ToSchema`: utoipa에서 OpenAPI 스키마 자동 생성
- `rename_all = "camelCase"`: JSON 필드명을 camelCase로 매핑

---

### 12. `SubmitAnswerItem`

**소스**: `dto.rs:147~154`

```rust
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubmitAnswerItem {
    pub question_number: i32,  // JSON: "questionNumber"
    pub content: String,       // JSON: "content"
}
```

- `question_number`는 `i32` 타입 (1~5 범위, 서비스에서 검증)
- `content`는 `String` 타입 (필수값, `Option`이 아님 -- 임시 저장의 `DraftItem`과 다름)

---

### 13. `SubmitRetrospectResponse`

**소스**: `dto.rs:157~166`

```rust
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubmitRetrospectResponse {
    pub retrospect_id: i64,
    pub submitted_at: String,
    pub status: RetrospectStatus,
}
```

- `submitted_at`: KST 기준 `YYYY-MM-DD` 형식 문자열
- `status`: 항상 `RetrospectStatus::Submitted` -> JSON에서 `"SUBMITTED"`로 직렬화

---

## 데이터베이스 키워드

### 14. `member_retro` 테이블

**소스**: `codes/server/src/domain/member/entity/member_retro.rs:23~33`

```rust
#[sea_orm(table_name = "member_retro")]
pub struct Model {
    pub member_retro_id: i64,
    pub personal_insight: Option<String>,
    pub member_id: i64,
    pub retrospect_id: i64,
    pub status: RetrospectStatus,
    pub submitted_at: Option<DateTime>,
}
```

- 멤버와 회고의 참여 관계를 관리하는 테이블
- `status`: 해당 멤버의 회고 참여 상태 (DRAFT/SUBMITTED/ANALYZED)
- `submitted_at`: 최종 제출 시각 (UTC)

### 15. 트랜잭션 내 관련 테이블

| 테이블 | 역할 | 수행 작업 |
|--------|------|-----------|
| `retrospect` | 회고 기본 정보 | 존재 여부 확인 (트랜잭션 외부) |
| `member_retro` | 멤버-회고 참여 관계 | 행 잠금 조회 + 상태 업데이트 |
| `member_response` | 멤버-응답 매핑 | response_id 조회 |
| `response` | 실제 답변 데이터 | 5건 내용 업데이트 |
