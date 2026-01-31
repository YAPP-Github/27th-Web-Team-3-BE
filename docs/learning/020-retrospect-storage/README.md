# [API-020] 보관함 조회 API 학습 노트

## 개요
- **엔드포인트**: `GET /api/v1/retrospects/storage`
- **역할**: 사용자가 제출 완료한 회고 목록을 연도별로 그룹화하여 조회하며, 기간 필터로 조회 범위를 제한할 수 있다
- **인증**: Bearer 토큰 필요

## 아키텍처 구조

```
Client  ──?range=3_MONTHS──►  Handler (get_storage)
                                  │
                                  ├─ AuthUser: JWT에서 user_id 추출
                                  ├─ Query(params): 쿼리 파라미터 파싱
                                  │
                                  ▼
                              Service (get_storage)
                                  │
                                  ├─ 1. range 필터 기본값 처리 (unwrap_or_default)
                                  ├─ 2. member_retro 테이블에서 SUBMITTED/ANALYZED 상태 조회
                                  ├─ 3. 기간 필터 적용 (submitted_at >= cutoff)
                                  ├─ 4. 회고 정보 조회 (retrospect 테이블)
                                  ├─ 5. 참여자 수 배치 조회 (HashMap 집계)
                                  ├─ 6. BTreeMap으로 연도별 그룹화
                                  └─ 7. 연도 내림차순 + 그룹 내 최신순 정렬
                                  │
                                  ▼
                          StorageResponse { years: Vec<StorageYearGroup> }
```

## 핵심 코드 분석

### 1. Handler 계층

**소스**: `codes/server/src/domain/retrospect/handler.rs:368-383`

```rust
pub async fn get_storage(
    user: AuthUser,
    State(state): State<AppState>,
    Query(params): Query<StorageQueryParams>,
) -> Result<Json<BaseResponse<StorageResponse>>, AppError> {
    let user_id = user.user_id()?;
    let result = RetrospectService::get_storage(state, user_id, params).await?;
    Ok(Json(BaseResponse::success_with_message(result, "보관함 조회를 성공했습니다.")))
}
```

- `Query(params)`: Axum의 Query extractor가 URL 쿼리스트링(`?range=3_MONTHS`)을 `StorageQueryParams` 구조체로 자동 역직렬화한다.
- 핸들러에 별도 검증 로직이 없다. `StorageRangeFilter` enum의 serde 역직렬화가 유효하지 않은 값을 자동 거부하기 때문이다.

### 2. DTO 설계

**소스**: `codes/server/src/domain/retrospect/dto.rs:277-367`

| 구조체 | 역할 | 라인 |
|--------|------|------|
| `StorageRangeFilter` | 기간 필터 enum (ALL, 3_MONTHS, 6_MONTHS, 1_YEAR) | dto.rs:277-315 |
| `StorageQueryParams` | 쿼리 파라미터 (`range: Option<StorageRangeFilter>`) | dto.rs:318-323 |
| `StorageRetrospectItem` | 개별 회고 아이템 (id, 날짜, 제목, 방식, 참여자 수) | dto.rs:326-339 |
| `StorageYearGroup` | 연도별 그룹 (연도 레이블 + 회고 리스트) | dto.rs:342-349 |
| `StorageResponse` | 최종 응답 (`years: Vec<StorageYearGroup>`) | dto.rs:352-357 |

### 3. StorageRangeFilter enum

**소스**: `codes/server/src/domain/retrospect/dto.rs:277-315`

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

impl StorageRangeFilter {
    pub fn days(&self) -> Option<i64> {
        match self {
            StorageRangeFilter::All => None,
            StorageRangeFilter::ThreeMonths => Some(90),
            StorageRangeFilter::SixMonths => Some(180),
            StorageRangeFilter::OneYear => Some(365),
        }
    }
}
```

- `#[default]` 속성으로 `All` variant를 기본값으로 지정한다. `Default` trait derive와 결합하여 `StorageRangeFilter::default()` 호출 시 `All`을 반환한다.
- `days()` 메서드는 `Option<i64>`를 반환한다. `All`은 `None`(전체 기간), 나머지는 `Some(일수)`로 필터 기준일을 계산한다.
- `#[serde(rename = "3_MONTHS")]`처럼 각 variant에 개별 rename을 적용한다. Rust 식별자는 숫자로 시작할 수 없으므로 `ThreeMonths`라는 이름을 쓰되, 직렬화/역직렬화 시에는 `"3_MONTHS"`로 매핑한다.

### 4. Service 비즈니스 로직

**소스**: `codes/server/src/domain/retrospect/service.rs:688-805`

**4-1. 기본값 처리 (line 693)**
```rust
let range_filter = params.range.unwrap_or_default();
```
`Option<StorageRangeFilter>`가 `None`이면 `StorageRangeFilter::default()` = `All`을 사용한다.

**4-2. 상태 필터 (line 702-707)**
```rust
let mut member_retro_query = member_retro::Entity::find()
    .filter(member_retro::Column::MemberId.eq(user_id))
    .filter(
        member_retro::Column::Status
            .is_in([RetrospectStatus::Submitted, RetrospectStatus::Analyzed]),
    );
```
제출 완료(`Submitted`) 또는 분석 완료(`Analyzed`) 상태인 회고만 조회한다. 보관함에는 완료된 회고만 표시해야 하기 때문이다.

**4-3. 기간 필터 적용 (line 710-714)**
```rust
if let Some(days) = range_filter.days() {
    let cutoff = Utc::now().naive_utc() - chrono::Duration::days(days);
    member_retro_query =
        member_retro_query.filter(member_retro::Column::SubmittedAt.gte(cutoff));
}
```
`days()`가 `Some`이면 현재 시각에서 해당 일수를 빼서 기준 시점(cutoff)을 계산하고, `submitted_at >= cutoff` 조건을 추가한다. `None`이면 필터를 추가하지 않아 전체 기간 조회가 된다.

**4-4. 참여자 수 배치 조회 (line 736-745)**
```rust
let mut member_counts: HashMap<i64, i64> = HashMap::new();
for mr in &all_member_retros_for_count {
    *member_counts.entry(mr.retrospect_id).or_insert(0) += 1;
}
```
N+1 쿼리를 피하기 위해 단일 배치 쿼리로 모든 회고의 참여자 수를 한 번에 조회하고 `HashMap`에 집계한다.

**4-5. 연도별 그룹화 (line 748-784)**
```rust
let mut year_groups: BTreeMap<i32, Vec<StorageRetrospectItem>> = BTreeMap::new();
// ...
year_groups.entry(year).or_default().push(item);
```
`BTreeMap`을 사용하여 key(연도)가 자동 정렬된 상태로 유지된다. `HashMap`과 달리 정렬 순서가 보장된다.

**4-6. 최종 정렬 (line 788-802)**
```rust
let mut years: Vec<StorageYearGroup> = year_groups
    .into_iter()
    .rev()          // BTreeMap은 오름차순이므로 rev()로 내림차순 변환
    .map(|(year, mut items)| {
        items.sort_by(|a, b| b.display_date.cmp(&a.display_date));  // 그룹 내 최신순
        StorageYearGroup {
            year_label: format!("{}년", year),
            retrospects: items,
        }
    })
    .collect();
```
`BTreeMap`의 `into_iter()`는 키 오름차순이므로 `.rev()`로 내림차순(최신 연도 우선)으로 뒤집는다. 각 그룹 내에서는 `display_date`를 내림차순으로 정렬한다.

## 사용된 Rust 패턴

### 1. Query Extractor 패턴
```rust
Query(params): Query<StorageQueryParams>
```
Axum의 `Query` extractor가 URL 쿼리스트링을 구조체로 자동 파싱한다. `serde::Deserialize`를 구현한 타입이면 사용 가능하다.

### 2. Default trait + `#[default]` 속성
```rust
#[derive(Default)]
pub enum StorageRangeFilter {
    #[default]
    All,
    // ...
}
```
Rust 1.62+에서 enum variant에 `#[default]` 속성을 사용할 수 있다. `unwrap_or_default()`와 조합하면 `Option` 처리가 간결해진다.

### 3. BTreeMap을 활용한 정렬된 그룹화
`HashMap` 대신 `BTreeMap`을 사용하여 삽입 시점부터 키가 정렬된 상태를 유지한다. 별도의 정렬 단계 없이 순서가 보장된다.

### 4. Entry API를 활용한 집계
```rust
*member_counts.entry(mr.retrospect_id).or_insert(0) += 1;
```
`entry()` API로 키가 없으면 초기값을 삽입하고, 있으면 기존 값에 접근한다. 조건 분기 없이 한 줄로 카운트 집계가 가능하다.

### 5. Option 기반 조건부 필터
```rust
if let Some(days) = range_filter.days() {
    // 필터 조건 추가
}
```
`days()`가 `None`이면 블록을 건너뛰어 조건 없는 전체 조회가 된다. 명시적인 분기 대신 `Option`의 패턴 매칭으로 깔끔하게 처리한다.

## 학습 포인트

### 새롭게 알게 된 점
1. **BTreeMap vs HashMap**: `BTreeMap`은 키 순서가 보장되므로 그룹화 + 정렬이 동시에 필요한 경우 유용하다. `HashMap`은 O(1) 조회가 장점이지만 순서가 없다.
2. **enum의 `#[default]` 속성**: Rust 1.62부터 enum variant에 직접 default를 지정할 수 있다. 이전에는 `impl Default`를 수동으로 작성해야 했다.
3. **serde rename으로 비표준 식별자 처리**: Rust에서 `3Months`라는 식별자는 불가능하지만, `#[serde(rename = "3_MONTHS")]`로 JSON 측 이름은 자유롭게 지정할 수 있다.
4. **Axum Query extractor**: Path와 달리 Query는 쿼리스트링 전체를 하나의 구조체로 역직렬화한다. `Option` 필드는 쿼리 파라미터 생략 시 `None`으로 처리된다.

### 설계 결정의 Trade-offs

| 결정 | 장점 | 단점 |
|------|------|------|
| BTreeMap으로 연도 그룹화 | 삽입 시 자동 정렬, 별도 sort 불필요 | HashMap 대비 삽입/조회 O(log n) |
| enum 기반 기간 필터 | 타입 안전성, 잘못된 값 컴파일 타임/역직렬화 단계에서 차단 | 새 필터 추가 시 코드 변경 필요 |
| 배치 쿼리로 참여자 수 집계 | N+1 쿼리 방지, DB 부하 감소 | 메모리에 전체 데이터 로드 |
| UTC 기반 cutoff 계산 | 서버 타임존 독립적 | 표시용 날짜(KST)와 필터 기준(UTC) 불일치 가능성 |

## 참고
- 핸들러: `codes/server/src/domain/retrospect/handler.rs` (line 349-383)
- 서비스: `codes/server/src/domain/retrospect/service.rs` (line 687-805)
- DTO: `codes/server/src/domain/retrospect/dto.rs` (line 272-367)
- API 스펙: `docs/api-specs/019-retrospect-storage-list.md`
