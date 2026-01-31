# [API-019] 보관함 조회 - 학습 키워드

## 1. Axum Query Extractor

**관련 소스**: `handler.rs:371`

Axum 프레임워크에서 URL 쿼리스트링을 Rust 구조체로 자동 역직렬화하는 extractor이다.

```rust
// 함수 시그니처에서 선언하면 Axum이 자동으로 쿼리스트링을 파싱
Query(params): Query<StorageQueryParams>
```

### 내부 동작
1. HTTP 요청의 URI에서 `?` 이후 쿼리스트링을 추출한다.
2. `serde_urlencoded` 크레이트를 사용하여 대상 타입으로 역직렬화한다.
3. 실패 시 `QueryRejection` 에러를 반환한다 (HTTP 400).

### 학습 자료
- [Axum 공식 문서 - Query](https://docs.rs/axum/latest/axum/extract/struct.Query.html)
- `axum::extract::Query<T>` where `T: DeserializeOwned`

### 프로젝트 내 사용 예시
- `handler.rs:371` - `Query<StorageQueryParams>`: 보관함 기간 필터
- `handler.rs:460` - `Query<SearchQueryParams>`: 검색 키워드
- `handler.rs:573` - `Query<ResponsesQueryParams>`: 답변 조회 카테고리/페이지네이션

---

## 2. BTreeMap

**관련 소스**: `service.rs:748`

키가 정렬된 상태를 유지하는 Rust 표준 라이브러리의 맵 자료구조이다. B-Tree 자료구조를 기반으로 한다.

```rust
use std::collections::BTreeMap;

let mut year_groups: BTreeMap<i32, Vec<StorageRetrospectItem>> = BTreeMap::new();
```

### HashMap과의 차이

| 특성 | BTreeMap | HashMap |
|------|----------|---------|
| 내부 구조 | B-Tree | 해시 테이블 |
| 키 순서 | 정렬됨 (Ord trait 필요) | 순서 없음 (Hash trait 필요) |
| 삽입/조회 | O(log n) | O(1) 평균, O(n) 최악 |
| 이터레이션 순서 | 키 오름차순 | 임의 순서 |
| 메모리 | 더 적게 사용 (캐시 친화적) | 해시 버킷 오버헤드 |

### 이 API에서 선택한 이유
연도별 그룹화 후 연도 내림차순 정렬이 필요했다. BTreeMap은 키(연도)가 자동 정렬되므로 `.rev()`만 호출하면 내림차순 이터레이션이 가능하다. HashMap을 사용했다면 별도로 키를 수집하고 정렬해야 한다.

### 학습 자료
- [Rust 표준 라이브러리 - BTreeMap](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html)
- [Rust 컬렉션 가이드](https://doc.rust-lang.org/std/collections/index.html)

---

## 3. chrono Datelike

**관련 소스**: `service.rs:751-774`

`chrono` 크레이트의 trait으로, 날짜 관련 메서드(year, month, day 등)를 제공한다.

```rust
use chrono::{Datelike, NaiveDateTime, Utc, Duration};

// cutoff 계산
let cutoff = Utc::now().naive_utc() - Duration::days(90);

// NaiveDateTime에서 연도 추출 (Datelike trait)
let year: i32 = naive_datetime.year();

// 또는 format을 통한 추출 (이 API에서 사용한 방식)
let year_str = naive_datetime.format("%Y").to_string();
```

### Datelike trait이 제공하는 주요 메서드
- `year()` → `i32`: 연도
- `month()` → `u32`: 월 (1-12)
- `day()` → `u32`: 일 (1-31)
- `weekday()` → `Weekday`: 요일
- `ordinal()` → `u32`: 연중 일수 (1-366)

### NaiveDateTime vs DateTime<Utc>
- `NaiveDateTime`: 타임존 정보 없음. DB 저장/조회 시 주로 사용.
- `DateTime<Utc>`: UTC 타임존 포함. `Utc::now()`가 반환하는 타입.
- `.naive_utc()`: `DateTime<Utc>` → `NaiveDateTime` 변환 (타임존 정보 제거).

### 학습 자료
- [chrono 공식 문서](https://docs.rs/chrono/latest/chrono/)
- [chrono::Datelike trait](https://docs.rs/chrono/latest/chrono/trait.Datelike.html)

---

## 4. serde rename (enum variant 개별 적용)

**관련 소스**: `dto.rs:277-292`

serde의 `rename` 속성으로 Rust 식별자와 직렬화/역직렬화 시 이름을 다르게 지정한다.

```rust
#[derive(Deserialize)]
pub enum StorageRangeFilter {
    #[serde(rename = "ALL")]       // JSON/쿼리: "ALL"   → Rust: All
    All,
    #[serde(rename = "3_MONTHS")]  // JSON/쿼리: "3_MONTHS" → Rust: ThreeMonths
    ThreeMonths,
    #[serde(rename = "6_MONTHS")]
    SixMonths,
    #[serde(rename = "1_YEAR")]
    OneYear,
}
```

### 왜 개별 rename이 필요한가?
- Rust 식별자 규칙: 숫자로 시작할 수 없다 (`3Months`는 불가능).
- API 스펙에서는 `3_MONTHS`와 같은 형식이 요구된다.
- `rename_all = "SCREAMING_SNAKE_CASE"`를 사용하면 `ThreeMonths` → `THREE_MONTHS`가 되어 원하는 `3_MONTHS`와 달라진다.
- 따라서 각 variant에 `#[serde(rename = "...")]`를 개별 적용해야 한다.

### rename 관련 옵션 정리

| 속성 | 적용 대상 | 예시 |
|------|----------|------|
| `#[serde(rename = "name")]` | 필드/variant 개별 | `ThreeMonths` → `"3_MONTHS"` |
| `#[serde(rename_all = "camelCase")]` | 구조체/enum 전체 | `project_name` → `"projectName"` |
| `#[serde(rename_all = "SCREAMING_SNAKE_CASE")]` | 구조체/enum 전체 | `Kpt` → `"KPT"` |

### 학습 자료
- [serde 공식 문서 - Field attributes](https://serde.rs/field-attrs.html)
- [serde 공식 문서 - Enum representations](https://serde.rs/enum-representations.html)

---

## 5. Default trait (enum)

**관련 소스**: `dto.rs:277-282`

Rust의 `Default` trait은 타입의 기본값을 정의한다. enum에서는 `#[default]` 속성으로 기본 variant를 지정한다.

```rust
#[derive(Default)]
pub enum StorageRangeFilter {
    #[default]    // Rust 1.62+
    All,
    ThreeMonths,
    // ...
}

// 사용
let filter = StorageRangeFilter::default();  // → All
```

### `unwrap_or_default()`와의 조합

**소스**: `service.rs:693`

```rust
let range_filter = params.range.unwrap_or_default();
```

`Option<StorageRangeFilter>`에서:
- `Some(filter)` → `filter` 반환
- `None` → `StorageRangeFilter::default()` = `All` 반환

### Default trait의 derive 가능 조건
- **구조체**: 모든 필드가 `Default`를 구현해야 한다.
- **enum**: Rust 1.62+에서 `#[default]` 속성이 있는 variant가 정확히 하나 있어야 한다.

### 학습 자료
- [Rust 표준 라이브러리 - Default trait](https://doc.rust-lang.org/std/default/trait.Default.html)
- [Rust RFC 3107 - derive_default_enum](https://rust-lang.github.io/rfcs/3107-derive-default-enum.html)

---

## 6. Entry API (HashMap/BTreeMap)

**관련 소스**: `service.rs:744`, `service.rs:784`

컬렉션의 키에 대해 "없으면 삽입, 있으면 수정" 패턴을 효율적으로 처리하는 API이다.

```rust
// 참여자 수 집계 (HashMap)
*member_counts.entry(mr.retrospect_id).or_insert(0) += 1;

// 연도별 그룹화 (BTreeMap)
year_groups.entry(year).or_default().push(item);
```

### Entry API 체이닝 패턴

```rust
map.entry(key)           // Entry<'_, K, V> 반환
   .or_insert(default)   // 키 없으면 default 삽입, 가변 참조 반환
   .or_default()         // 키 없으면 V::default() 삽입 (V: Default 필요)
   .or_insert_with(|| f()) // 키 없으면 클로저 실행 결과 삽입
   .and_modify(|v| *v += 1) // 키 있으면 기존 값 수정
```

### Entry API 없이 같은 로직을 구현하면

```rust
// Entry API 사용 (한 줄)
*member_counts.entry(mr.retrospect_id).or_insert(0) += 1;

// Entry API 없이 (여러 줄)
if let Some(count) = member_counts.get_mut(&mr.retrospect_id) {
    *count += 1;
} else {
    member_counts.insert(mr.retrospect_id, 1);
}
```

Entry API는 키 조회를 한 번만 수행하므로 성능상으로도 유리하다 (키를 두 번 해싱하지 않음).

### 학습 자료
- [Rust 표준 라이브러리 - HashMap::entry](https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.entry)
- [Rust by Example - HashMap](https://doc.rust-lang.org/rust-by-example/std/hash.html)

---

## 7. DoubleEndedIterator (.rev())

**관련 소스**: `service.rs:790`

양 끝에서 순회할 수 있는 이터레이터 trait이다. `.rev()`를 호출하면 역순으로 순회하는 이터레이터를 반환한다.

```rust
year_groups
    .into_iter()  // BTreeMap의 이터레이터 (키 오름차순)
    .rev()         // DoubleEndedIterator::rev() → 키 내림차순
    .map(...)
    .collect()
```

### BTreeMap의 이터레이터가 DoubleEndedIterator인 이유
B-Tree 자료구조는 정렬된 상태를 유지하므로 가장 작은 키(왼쪽 끝)와 가장 큰 키(오른쪽 끝) 모두에서 효율적으로 접근할 수 있다. 따라서 `DoubleEndedIterator`를 구현할 수 있고, `.rev()`로 내림차순 순회가 가능하다.

### `.rev()` vs `.sort_by().reverse()`

```rust
// .rev() - 이터레이터 레벨 (추가 메모리 할당 없음)
btreemap.into_iter().rev().collect::<Vec<_>>();

// .sort_by() - 벡터 레벨 (이미 collect된 후 정렬)
let mut vec: Vec<_> = btreemap.into_iter().collect();
vec.sort_by(|a, b| b.0.cmp(&a.0));
```

`.rev()`는 새로운 컬렉션을 만들지 않고 이터레이터 어댑터로만 동작하므로 더 효율적이다.

### 학습 자료
- [Rust 표준 라이브러리 - DoubleEndedIterator](https://doc.rust-lang.org/std/iter/trait.DoubleEndedIterator.html)
- [Rust 표준 라이브러리 - Iterator::rev](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.rev)

---

## 8. IntoParams (utoipa)

**관련 소스**: `dto.rs:318`

utoipa 크레이트의 derive 매크로로, 구조체를 Swagger/OpenAPI 문서의 쿼리 파라미터로 자동 문서화한다.

```rust
#[derive(Debug, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct StorageQueryParams {
    /// 기간 필터 (기본값: ALL)
    pub range: Option<StorageRangeFilter>,
}
```

### 핸들러에서의 연동

```rust
// handler.rs:356
#[utoipa::path(
    get,
    path = "/api/v1/retrospects/storage",
    params(StorageQueryParams),    // ← IntoParams derive한 타입 직접 전달
    // ...
)]
```

`params(StorageQueryParams)`로 지정하면 Swagger UI에서 `range` 파라미터가 자동으로 쿼리 파라미터 섹션에 표시된다.

### IntoParams vs ToSchema
- `IntoParams`: 쿼리/패스 파라미터 문서화. `params(...)` 속성에서 사용.
- `ToSchema`: 요청/응답 바디 문서화. `request_body`, `responses`에서 사용.

### 학습 자료
- [utoipa 공식 문서](https://docs.rs/utoipa/latest/utoipa/)
- [utoipa IntoParams](https://docs.rs/utoipa/latest/utoipa/derive.IntoParams.html)

---

## 9. tracing 매크로와 구조화된 로깅

**관련 소스**: `service.rs:695-699`

```rust
info!(
    user_id = user_id,
    range = %range_filter,    // %: Display trait 사용
    "보관함 조회 요청"
);
```

### tracing 필드 포맷 지시자

| 지시자 | trait | 예시 | 출력 |
|--------|-------|------|------|
| `=` (기본) | Debug 또는 Value | `user_id = user_id` | `user_id=42` |
| `%` | Display | `range = %range_filter` | `range=3_MONTHS` |
| `?` | Debug | `filter = ?range_filter` | `filter=ThreeMonths` |

`%range_filter`는 `StorageRangeFilter`의 `Display` 구현(`dto.rs:294-303`)을 호출하여 사람이 읽기 쉬운 형태로 출력한다.

### 학습 자료
- [tracing 공식 문서](https://docs.rs/tracing/latest/tracing/)
- [tracing 매크로 사용법](https://docs.rs/tracing/latest/tracing/index.html#using-the-macros)
