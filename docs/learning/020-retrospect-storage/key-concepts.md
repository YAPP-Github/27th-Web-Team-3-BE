# [API-019] 보관함 조회 - 핵심 개념

## 1. Query Parameter Extractor

**소스**: `handler.rs:371`

```rust
Query(params): Query<StorageQueryParams>
```

Axum의 `Query` extractor는 URL 쿼리스트링을 지정한 구조체로 역직렬화한다. 내부적으로 `serde_urlencoded` 크레이트를 사용한다.

### 동작 방식

| URL 쿼리스트링 | 파싱 결과 |
|---------------|----------|
| `?range=ALL` | `StorageQueryParams { range: Some(StorageRangeFilter::All) }` |
| `?range=3_MONTHS` | `StorageQueryParams { range: Some(StorageRangeFilter::ThreeMonths) }` |
| (쿼리 파라미터 없음) | `StorageQueryParams { range: None }` |
| `?range=INVALID` | 역직렬화 실패 → 400 Bad Request |

### Path vs Query 비교

```rust
// Path: URL 경로의 일부 (/api/v1/teams/{teamId})
Path(team_id): Path<i64>

// Query: URL 쿼리스트링 (?range=ALL&page=1)
Query(params): Query<StorageQueryParams>
```

- `Path`는 단일 값 또는 튜플로 추출한다.
- `Query`는 전체 쿼리스트링을 하나의 구조체로 역직렬화한다.
- `Query`의 대상 구조체는 `Deserialize`를 derive해야 한다.

### StorageQueryParams 구조체

**소스**: `dto.rs:318-323`

```rust
#[derive(Debug, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct StorageQueryParams {
    pub range: Option<StorageRangeFilter>,
}
```

- `IntoParams`: utoipa 크레이트의 derive 매크로. Swagger 문서에서 쿼리 파라미터를 자동 문서화한다.
- `range`가 `Option`이므로 쿼리스트링에 `range`가 없어도 역직렬화가 성공한다 (`None`으로 처리).
- `#[serde(rename_all = "camelCase")]`는 이 구조체에서는 필드명이 `range` 하나뿐이라 실질적 효과는 없지만, 프로젝트 컨벤션에 따라 일관성을 유지한다.

---

## 2. Enum 기반 기간 필터와 days() 메서드

**소스**: `dto.rs:277-315`

```rust
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, ToSchema)]
pub enum StorageRangeFilter {
    #[serde(rename = "ALL")]
    #[default]
    All,
    #[serde(rename = "3_MONTHS")]
    ThreeMonths,
    #[serde(rename = "6_MONTHS")]
    SixMonths,
    #[serde(rename = "1_YEAR")]
    OneYear,
}
```

### `#[default]` 속성

Rust 1.62부터 enum variant에 `#[default]`를 적용할 수 있다. `Default` trait derive와 함께 사용하면 해당 variant가 기본값이 된다.

```rust
// 이전 방식 (수동 구현)
impl Default for StorageRangeFilter {
    fn default() -> Self {
        StorageRangeFilter::All
    }
}

// 현재 방식 (#[default] 속성)
#[derive(Default)]
pub enum StorageRangeFilter {
    #[default]
    All,
    // ...
}
```

### days() 메서드의 Option 반환 패턴

**소스**: `dto.rs:307-314`

```rust
pub fn days(&self) -> Option<i64> {
    match self {
        StorageRangeFilter::All => None,          // 전체 기간 → 필터 없음
        StorageRangeFilter::ThreeMonths => Some(90),
        StorageRangeFilter::SixMonths => Some(180),
        StorageRangeFilter::OneYear => Some(365),
    }
}
```

`Option<i64>`를 반환함으로써 "필터 없음"과 "특정 기간" 두 가지 상태를 타입 레벨에서 구분한다. 호출 측에서는 `if let Some(days)` 패턴으로 자연스럽게 분기한다.

```rust
// service.rs:710-714
if let Some(days) = range_filter.days() {
    let cutoff = Utc::now().naive_utc() - chrono::Duration::days(days);
    member_retro_query =
        member_retro_query.filter(member_retro::Column::SubmittedAt.gte(cutoff));
}
// All인 경우 이 블록을 건너뛰어 필터 없이 전체 조회
```

### serde rename으로 비표준 식별자 처리

Rust 식별자는 숫자로 시작할 수 없으므로 `3_MONTHS`를 그대로 variant 이름으로 쓸 수 없다. `#[serde(rename = "3_MONTHS")]`로 JSON/쿼리스트링 측 이름만 별도 지정한다.

```rust
#[serde(rename = "3_MONTHS")]
ThreeMonths,    // Rust 코드에서는 ThreeMonths
                // JSON/쿼리스트링에서는 "3_MONTHS"
```

---

## 3. 연도별 그룹화 (BTreeMap)

**소스**: `service.rs:748-784`

### BTreeMap vs HashMap

```rust
// 이 API에서 사용한 방식
let mut year_groups: BTreeMap<i32, Vec<StorageRetrospectItem>> = BTreeMap::new();
```

| 특성 | BTreeMap | HashMap |
|------|----------|---------|
| 키 순서 | 정렬 보장 (오름차순) | 순서 없음 |
| 삽입/조회 | O(log n) | O(1) 평균 |
| 이터레이션 | 정렬된 순서로 순회 | 임의 순서 |
| 적합한 용도 | 순서가 중요한 그룹화 | 빠른 lookup |

이 API에서 BTreeMap을 선택한 이유:
- 연도별로 내림차순 정렬이 필요하다.
- BTreeMap은 키가 자동 오름차순 정렬되므로 `.rev()`만 호출하면 내림차순이 된다.
- HashMap을 사용하면 별도의 키 수집 + 정렬 단계가 필요하다.

### Entry API

```rust
year_groups.entry(year).or_default().push(item);
```

`entry()` API의 동작:
1. `entry(year)`: 키에 대한 Entry(점유 상태) 반환
2. `.or_default()`: 키가 없으면 `Vec::default()` (빈 벡터) 삽입 후 가변 참조 반환
3. `.push(item)`: 해당 벡터에 아이템 추가

이 패턴은 "키가 없으면 생성, 있으면 추가" 로직을 한 줄로 표현한다.

### into_iter().rev()로 내림차순 변환

**소스**: `service.rs:788-798`

```rust
let years: Vec<StorageYearGroup> = year_groups
    .into_iter()   // BTreeMap의 소유권 이동 이터레이터 (키 오름차순)
    .rev()          // 이터레이터를 역순으로 (키 내림차순)
    .map(|(year, mut items)| {
        items.sort_by(|a, b| b.display_date.cmp(&a.display_date));
        StorageYearGroup {
            year_label: format!("{}년", year),
            retrospects: items,
        }
    })
    .collect();
```

- `into_iter()`: `BTreeMap`을 소비하며 `(key, value)` 튜플을 키 오름차순으로 생산한다.
- `.rev()`: `DoubleEndedIterator` trait 덕분에 끝에서부터 순회할 수 있다. BTreeMap의 이터레이터가 이를 구현하고 있다.
- `.map()` 내에서 각 그룹의 회고 리스트를 `display_date` 내림차순으로 정렬한다.

---

## 4. chrono::Datelike trait과 날짜 처리

**소스**: `service.rs:751-774`

이 API에서 날짜 처리는 `chrono` 크레이트의 `NaiveDateTime`을 기반으로 한다.

### UTC에서 KST 변환

```rust
let kst_offset = chrono::Duration::hours(9);

let display_date = submitted_dates
    .get(&retro.retrospect_id)
    .map(|dt| (*dt + kst_offset).format("%Y-%m-%d").to_string())
    .unwrap_or_else(|| {
        (retro.created_at + kst_offset).format("%Y-%m-%d").to_string()
    });
```

- DB에 저장된 `submitted_at`은 UTC 기준 `NaiveDateTime`이다.
- 표시용 날짜를 만들 때 `+ Duration::hours(9)`로 KST로 변환한다.
- `NaiveDateTime`은 타임존 정보가 없으므로 단순 시간 덧셈으로 변환한다.

### 연도 추출

```rust
let year = submitted_dates
    .get(&retro.retrospect_id)
    .map(|dt| (*dt + kst_offset).format("%Y").to_string())
    .unwrap_or_else(|| (retro.created_at + kst_offset).format("%Y").to_string())
    .parse::<i32>()
    .unwrap_or(0);
```

- `format("%Y")`로 연도 문자열을 추출한 뒤 `.parse::<i32>()`로 정수 변환한다.
- `chrono`의 `Datelike` trait이 제공하는 `.year()` 메서드를 직접 호출하는 대신 format + parse 방식을 사용했다. `Datelike::year()`를 사용하면 더 직접적이지만, 이미 KST 변환된 `NaiveDateTime`에서 `.year()`를 호출해도 동일한 결과다.

### cutoff 계산

**소스**: `service.rs:711`

```rust
let cutoff = Utc::now().naive_utc() - chrono::Duration::days(days);
```

- `Utc::now()`: 현재 UTC 시각 (`DateTime<Utc>`)
- `.naive_utc()`: 타임존 정보를 제거하여 `NaiveDateTime`으로 변환
- `- Duration::days(days)`: 지정된 일수만큼 과거 시점 계산
- 결과를 `submitted_at >= cutoff` 조건으로 사용하여 기간 필터 적용

---

## 5. Display trait 구현

**소스**: `dto.rs:294-303`

```rust
impl fmt::Display for StorageRangeFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageRangeFilter::All => write!(f, "ALL"),
            StorageRangeFilter::ThreeMonths => write!(f, "3_MONTHS"),
            StorageRangeFilter::SixMonths => write!(f, "6_MONTHS"),
            StorageRangeFilter::OneYear => write!(f, "1_YEAR"),
        }
    }
}
```

`Display` trait 구현의 용도:
- `tracing` 로깅에서 `%range_filter` 포맷 지시자로 사용 (`service.rs:697`)
- `.to_string()` 메서드 자동 제공 (`ToString` trait이 `Display` 구현체에 자동 적용)
- `format!()` 매크로에서 `{}` 플레이스홀더로 사용 가능

```rust
// service.rs:695-699
info!(
    user_id = user_id,
    range = %range_filter,    // Display trait 사용
    "보관함 조회 요청"
);
```

---

## 6. N+1 쿼리 방지 패턴

**소스**: `service.rs:736-745`

```rust
// 단일 배치 쿼리로 모든 회고의 참여자 데이터 조회
let all_member_retros_for_count = member_retro::Entity::find()
    .filter(member_retro::Column::RetrospectId.is_in(retrospect_ids.clone()))
    .all(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

// 어플리케이션 레벨에서 HashMap으로 집계
let mut member_counts: HashMap<i64, i64> = HashMap::new();
for mr in &all_member_retros_for_count {
    *member_counts.entry(mr.retrospect_id).or_insert(0) += 1;
}
```

### N+1 문제란?

N개의 회고가 있을 때, 각 회고의 참여자 수를 개별 쿼리로 조회하면 N+1번의 DB 쿼리가 발생한다.

```
// N+1 패턴 (비효율적)
Query 1: SELECT * FROM retrospect WHERE ...     // 회고 목록 (1회)
Query 2: SELECT COUNT(*) FROM member_retro WHERE retrospect_id = 1  // (N회)
Query 3: SELECT COUNT(*) FROM member_retro WHERE retrospect_id = 2
...
```

### 배치 쿼리 패턴 (이 API의 방식)

```
Query 1: SELECT * FROM member_retro WHERE member_id = ? AND status IN (...)
Query 2: SELECT * FROM retrospect WHERE retrospect_id IN (?, ?, ...)
Query 3: SELECT * FROM member_retro WHERE retrospect_id IN (?, ?, ...)
→ 어플리케이션 레벨에서 HashMap 집계
```

항상 3회의 쿼리로 고정된다. 회고 수(N)에 관계없이 DB 왕복 횟수가 일정하다.
