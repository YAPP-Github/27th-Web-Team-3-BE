# 핵심 개념: 회고 최종 제출 (API-017)

## 1. 상태 머신 (State Machine)

### RetrospectStatus enum

**소스**: `codes/server/src/domain/member/entity/member_retro.rs:11~21`

```rust
pub enum RetrospectStatus {
    Draft,      // 임시 저장 상태
    Submitted,  // 최종 제출 완료
    Analyzed,   // AI 분석 완료
}
```

### 상태 전이 규칙

```
DRAFT ──(최종 제출)──> SUBMITTED ──(AI 분석)──> ANALYZED
```

- `DRAFT` -> `SUBMITTED`: API-017 (회고 최종 제출)에서 수행
- `SUBMITTED` -> `ANALYZED`: API-022 (회고 분석)에서 수행
- 역방향 전이는 허용되지 않음

### 제출 시 상태 확인 로직

**소스**: `service.rs:612~619`

```rust
if member_retro_model.status == RetrospectStatus::Submitted
    || member_retro_model.status == RetrospectStatus::Analyzed
{
    return Err(AppError::RetroAlreadySubmitted(...));
}
```

- `SUBMITTED` 또는 `ANALYZED` 상태에서는 재제출 불가
- 오직 `DRAFT` 상태에서만 최종 제출이 허용됨
- 이를 통해 멱등성(idempotency)은 보장하지 않지만, 데이터 무결성을 보호함

---

## 2. 완전성 검증 (Completeness Validation)

### 정확히 5개 답변 필수

**소스**: `service.rs:1544~1558`

```rust
// 1. 배열 길이 검증
if answers.len() != 5 {
    return Err(AppError::RetroAnswersMissing(...));
}

// 2. HashSet으로 질문 번호 1~5 모두 존재 확인
let question_numbers: HashSet<i32> = answers.iter().map(|a| a.question_number).collect();
let expected: HashSet<i32> = (1..=5).collect();
if question_numbers != expected {
    return Err(AppError::RetroAnswersMissing(...));
}
```

왜 두 가지 검증이 모두 필요한가:
- `len() != 5`: 답변 개수가 맞는지 확인 (4개 이하 또는 6개 이상 차단)
- `HashSet` 비교: 중복 질문 번호가 있거나 범위 밖 번호(예: 0, 6)가 있는 경우 차단
- 예: `[1, 1, 2, 3, 4]`는 len이 5이지만, HashSet은 `{1,2,3,4}`로 expected와 불일치

### 임시 저장(API-016)과의 차이

| 항목 | 임시 저장 (API-016) | 최종 제출 (API-017) |
|------|---------------------|---------------------|
| 답변 개수 | 1~5개 (부분 저장 가능) | 정확히 5개 필수 |
| 공백 허용 | 허용 (빈 문자열, null 가능) | 불가 (trim 후 비어있으면 에러) |
| content 필수 | 선택 (`Option<String>`) | 필수 (`String`) |
| 상태 변경 | 변경 없음 | DRAFT -> SUBMITTED |

---

## 3. 공백 trim 검증

**소스**: `service.rs:1562~1567`

```rust
if answer.content.trim().is_empty() {
    return Err(AppError::RetroAnswerWhitespaceOnly(
        "답변 내용은 공백만으로 구성될 수 없습니다.".to_string(),
    ));
}
```

### trim()의 동작

Rust의 `str::trim()`은 문자열 양쪽 끝의 공백 문자(whitespace)를 제거합니다.

- `"  답변  ".trim()` -> `"답변"` (양쪽 공백 제거)
- `"   ".trim()` -> `""` (공백만 있으면 빈 문자열)
- `"\t\n ".trim()` -> `""` (탭, 개행 등도 공백으로 처리)

### 검증 의미

- 공백, 탭, 개행만으로 구성된 답변은 의미 있는 내용이 아니므로 거부
- 앞뒤 공백이 있더라도 내용이 포함되면 통과
- **주의**: trim은 검증용으로만 사용되며, 실제 저장 시에는 원본 content가 그대로 저장됨

### 글자 수 검증

**소스**: `service.rs:1569~1574`

```rust
if answer.content.chars().count() > 1000 {
    return Err(AppError::RetroAnswerTooLong(...));
}
```

- `chars().count()`는 유니코드 문자 단위로 계산 (바이트가 아님)
- 한글, 이모지 등 멀티바이트 문자도 1글자로 계산
- `len()`(바이트 기준)이 아닌 `chars().count()`(문자 기준)를 사용하는 것이 핵심

---

## 4. 트랜잭션 내 다중 테이블 업데이트

### 트랜잭션 범위

**소스**: `service.rs:591~673`

```
트랜잭션 시작 (service.rs:592)
    |
    ├── member_retro 조회 + 행 잠금 (service.rs:599~610)
    ├── 이미 제출 여부 확인 (service.rs:613~619)
    ├── member_response 조회 (service.rs:622~629)
    ├── response 조회 (service.rs:632~638)
    ├── response 5건 업데이트 (service.rs:648~659)
    ├── member_retro 상태 업데이트 (service.rs:662~668)
    |
트랜잭션 커밋 (service.rs:671)
```

### 원자성 보장

트랜잭션 내에서 수행되는 작업:
1. **5개 답변 내용 업데이트** (`response` 테이블): 각 질문에 대한 답변 내용과 `updated_at` 갱신
2. **참여 상태 변경** (`member_retro` 테이블): `status`를 `SUBMITTED`로, `submitted_at` 기록

이 두 작업은 반드시 함께 성공하거나 함께 실패해야 합니다:
- 답변만 저장되고 상태가 변경되지 않으면: 임시 저장 상태인데 최종 답변이 기록됨
- 상태만 변경되고 답변이 저장되지 않으면: 제출 완료인데 답변이 비어있음

### 행 잠금 (Exclusive Lock)

**소스**: `service.rs:602`

```rust
.lock_exclusive()   // SQL: SELECT ... FOR UPDATE
```

- `lock_exclusive()`는 해당 행에 배타적 잠금을 설정
- 같은 행에 대해 다른 트랜잭션이 동시에 읽기/쓰기를 할 수 없음
- 동일 사용자가 빠르게 두 번 제출 버튼을 누르는 경우에도 하나만 성공

---

## 5. UTC 저장 + KST 변환 응답 패턴

**소스**: `service.rs:647, 664, 676~678`

```rust
// DB 저장: UTC
let now = Utc::now().naive_utc();
member_retro_active.submitted_at = Set(Some(now));

// 응답 반환: KST (+9시간)
let kst_display = (now + chrono::Duration::hours(9))
    .format("%Y-%m-%d")
    .to_string();
```

### 이 패턴의 장점

- DB에는 항상 UTC로 저장하여 시간대 혼동 방지
- API 응답에서만 사용자 시간대(KST)로 변환
- 서버 시간대 설정에 의존하지 않는 일관된 동작

---

## 6. 질문-답변 매칭 전략

**소스**: `service.rs:648~651`

```rust
for answer in &req.answers {
    let idx = (answer.question_number - 1) as usize;
    let response_model = &responses[idx];
    // ...
}
```

### 매칭 방식

- `responses`는 `response_id` 오름차순으로 정렬된 5개 레코드
- `questionNumber`(1~5)를 0-based 인덱스(0~4)로 변환하여 배열 접근
- 이 방식은 `response_id` 오름차순 = 질문 순서라는 전제에 의존

### 전제 조건

- 회고 생성 시 1~5번 질문에 대한 response 레코드가 순서대로 생성되어야 함
- `responses.len() != 5` 검증(service.rs:640)이 이 전제를 보호

---

## Spring 프레임워크 비교

### Lock 관리 (동시성 제어)
- **Spring (JPA)**: `@Lock(LockModeType.PESSIMISTIC_WRITE)` 어노테이션이나 `entityManager.lock(...)`을 사용하여 비관적 락(`SELECT ... FOR UPDATE`) 적용.
- **Rust (SeaORM)**: 쿼리 빌더 체인에서 `.lock_exclusive()` 메서드를 호출하여 `FOR UPDATE` 구문 생성.

### Transaction 범위
- **Spring**: `@Transactional` 어노테이션이 붙은 메서드 전체가 트랜잭션 범위. RuntimeException 발생 시 자동 롤백.
- **Rust**: `db.begin()`으로 시작된 `txn` 객체의 생명주기가 트랜잭션 범위. `txn.commit()` 호출 전 에러가 반환되어 함수가 종료되면 `txn`이 drop되면서 자동 롤백 (RAII).

### Validation (검증)
- **Spring**: `@Valid`, `@NotNull`, `@Size` 등 어노테이션 기반 검증과 `Validator` 인터페이스 구현.
- **Rust**: `validator` 크레이트(`#[validate(...)]`)를 사용하거나, 서비스 로직 내에서 명시적인 검증 함수(`validate_answers`)를 구현하여 호출.

### 상태 패턴 (State Pattern)
- **Spring**: Enum에 메서드를 정의하여 상태 전이 로직을 포함하거나, 별도의 State Machine 라이브러리 사용.
- **Rust**: Enum을 사용하여 상태를 정의하고, `match` 문이나 `if` 조건문으로 상태 전이 로직을 명시적으로 구현. SeaORM의 `DeriveActiveEnum`으로 DB 값과 매핑.
