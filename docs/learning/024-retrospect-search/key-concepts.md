# 핵심 개념 (Key Concepts)

이 API를 구현하기 위해 알아야 할 주요 개념들입니다.

## 1. Optional 파라미터와 커스텀 에러 코드

`SearchQueryParams`의 `keyword`는 `Option<String>`으로 선언되어 있습니다.

**파일**: `dto.rs:506-513`

```rust
#[derive(Debug, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct SearchQueryParams {
    /// Option으로 선언하여 누락 시에도 핸들러가 실행되고
    /// 서비스 레이어에서 SEARCH4001 에러를 반환합니다.
    pub keyword: Option<String>,
}
```

**왜 `String`이 아닌 `Option<String>`인가?**
- `String`으로 선언하면 파라미터 누락 시 Axum이 자동으로 제네릭 400 응답을 반환
- `Option<String>`으로 선언하면 핸들러까지 도달한 뒤 서비스 레이어에서 `SEARCH4001` 에러 코드를 포함한 커스텀 응답을 반환할 수 있음
- 이 패턴은 API-020의 `ResponseCategory`를 `String`으로 받는 것과 동일한 설계 의도

## 2. 검색어 검증 (`validate_search_keyword`)

서비스 레이어에서 전용 검증 함수로 키워드를 검증합니다.

**파일**: `service.rs:952-969`

```rust
fn validate_search_keyword(keyword: Option<&str>) -> Result<String, AppError> {
    // 1. None이면 빈 문자열로 변환 후 trim
    let trimmed = keyword.unwrap_or("").trim().to_string();

    // 2. 빈 문자열 / 공백만 입력한 경우
    if trimmed.is_empty() {
        return Err(AppError::SearchKeywordInvalid(
            "검색어를 입력해주세요.".to_string(),
        ));
    }

    // 3. 100자 초과 (chars().count()로 유니코드 안전 카운팅)
    if trimmed.chars().count() > 100 {
        return Err(AppError::SearchKeywordInvalid(
            "검색어는 최대 100자까지 입력 가능합니다.".to_string(),
        ));
    }

    Ok(trimmed)
}
```

**핵심 포인트:**
- `unwrap_or("")`: `Option<&str>`이 `None`이면 빈 문자열로 대체
- `trim()`: 앞뒤 공백 제거 (공백만 입력한 경우도 잡아냄)
- `chars().count()`: 바이트 수(`len()`)가 아닌 유니코드 문자 수 카운팅 (한국어 1글자 = 3바이트이므로 중요)
- `SearchKeywordInvalid` -> 에러 코드 `SEARCH4001`, HTTP 400

## 3. 다중 쿼리 + HashMap 인메모리 조인

이 API의 데이터 접근은 **3개의 순차 쿼리**로 구성됩니다. JOIN이 아닌 HashMap 기반 인메모리 매핑을 사용합니다.

**파일**: `service.rs:986-1017`

### 쿼리 1: 사용자의 팀 목록 조회

```rust
// member_team 중간 테이블에서 유저가 속한 팀 ID 목록 조회
let user_teams = member_team::Entity::find()
    .filter(member_team::Column::MemberId.eq(user_id))
    .all(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

// 팀이 없으면 빈 결과 즉시 반환 (불필요한 쿼리 방지)
if user_teams.is_empty() {
    return Ok(vec![]);
}

let team_ids: Vec<i64> = user_teams.iter().map(|mt| mt.team_id).collect();
```

### 쿼리 2: 팀 이름 HashMap 구축

```rust
let teams = team::Entity::find()
    .filter(team::Column::TeamId.is_in(team_ids.clone()))
    .all(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

// HashMap<팀ID, 팀이름> 구축 -> 응답 매핑 시 O(1) 조회
let team_map: HashMap<i64, String> =
    teams.iter().map(|t| (t.team_id, t.name.clone())).collect();
```

### 쿼리 3: 회고 검색 (LIKE + IN)

```rust
let retrospects = retrospect::Entity::find()
    .filter(retrospect::Column::TeamId.is_in(team_ids))      // WHERE team_id IN (...)
    .filter(retrospect::Column::Title.contains(&keyword))     // AND title LIKE '%keyword%'
    .order_by_desc(retrospect::Column::StartTime)             // ORDER BY start_time DESC
    .order_by_desc(retrospect::Column::RetrospectId)          // , retrospect_id DESC (안정 정렬)
    .all(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```

**왜 JOIN 대신 다중 쿼리인가?**
- `member_team`은 유저-팀 N:M 관계의 중간 테이블로, retrospect와 직접 관계가 정의되어 있지 않음
- HashMap 조인은 코드가 명시적이고 디버깅이 용이
- 이 패턴은 프로젝트 내 API-012, API-019, API-020 등에서도 동일하게 사용

## 4. SeaORM LIKE 검색 (`contains`)

**파일**: `service.rs:1012`

```rust
.filter(retrospect::Column::Title.contains(&keyword))
```

- `contains()`: SQL `LIKE '%keyword%'`로 변환 (대소문자 구분은 DB 콜레이션 설정에 따름)
- `starts_with()`: `LIKE 'keyword%'`
- `ends_with()`: `LIKE '%keyword'`
- SeaORM이 자동으로 파라미터 바인딩하므로 SQL Injection 방지

**컬럼 매핑 주의**: API 스펙의 `projectName`은 DB 컬럼 `title`에 매핑됩니다.

## 5. 단일 `start_time` 컬럼에서 날짜/시간 분리

API 스펙의 응답에는 `retrospectDate`와 `retrospectTime`이 별도 필드이지만, DB에는 단일 `start_time` datetime 컬럼으로 저장됩니다.

**파일**: `service.rs:1027-1028`

```rust
retrospect_date: r.start_time.format("%Y-%m-%d").to_string(),  // "2026-01-28"
retrospect_time: r.start_time.format("%H:%M").to_string(),      // "14:30"
```

- `NaiveDateTime::format()`: chrono의 포맷 메서드로 원하는 형식의 문자열 생성
- `%Y-%m-%d`: 4자리 연도-월-일
- `%H:%M`: 24시간 형식 시:분
- `start_time`은 회고 생성 시 KST 기준으로 저장되므로 별도 타임존 변환 불필요

## 6. 안정 정렬 (Stable Sort)

**파일**: `service.rs:1013-1014`

```rust
.order_by_desc(retrospect::Column::StartTime)       // 1순위: 날짜+시간 내림차순
.order_by_desc(retrospect::Column::RetrospectId)     // 2순위: ID 내림차순 (동시간 안정 정렬)
```

- 동일한 `start_time`을 가진 회고가 여러 개 있을 때, `retrospect_id`로 추가 정렬하여 결과 순서를 일관되게 유지
- 이 패턴은 API-010에서도 동일하게 사용

## 7. DTO 매핑과 `unwrap_or_default`

**파일**: `service.rs:1020-1030`

```rust
let items: Vec<SearchRetrospectItem> = retrospects
    .iter()
    .map(|r| SearchRetrospectItem {
        retrospect_id: r.retrospect_id,
        project_name: r.title.clone(),                                    // DB: title -> API: projectName
        team_name: team_map.get(&r.team_id).cloned().unwrap_or_default(), // HashMap 조회
        retrospect_method: r.retrospect_method.clone(),
        retrospect_date: r.start_time.format("%Y-%m-%d").to_string(),
        retrospect_time: r.start_time.format("%H:%M").to_string(),
    })
    .collect();
```

- `team_map.get(&r.team_id)`: HashMap에서 팀 이름 조회 -> `Option<&String>` 반환
- `.cloned()`: `&String` -> `String` 변환
- `.unwrap_or_default()`: 팀 정보가 없으면 빈 문자열 반환 (데이터 정합성 보장)

## 8. 에러 처리 매핑

| 에러 | 코드 | HTTP | 트리거 조건 |
|------|------|------|-------------|
| `SearchKeywordInvalid` | SEARCH4001 | 400 | 키워드 누락, 공백만 입력, 100자 초과 |
| `Unauthorized` | AUTH4001 | 401 | JWT 토큰 유효하지 않음 |
| `InternalError` | COMMON500 | 500 | DB 연결 실패, 쿼리 오류 |

**에러 정의 위치**: `error.rs:111-112` (SearchKeywordInvalid), `error.rs:203` (SEARCH4001), `error.rs:244` (400)
