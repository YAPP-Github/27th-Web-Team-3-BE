# Spring 개발자를 위한 API-023 (회고 검색) 분석 가이드

이 문서는 Spring 개발자에게 익숙한 **검색(Search) 및 쿼리 파라미터 처리** 패턴을 Rust(Axum + SeaORM) 환경과 비교하여 설명합니다.

---

## 1. 아키텍처 매핑

| 역할 | Spring (Java/Kotlin) | Rust (Axum/SeaORM) | 파일 위치 |
|------|---------------------|-------------------|-----------|
| **웹 계층** | `@RestController` | `handler` 함수 | `handler.rs:457-476` |
| **비즈니스 계층** | `@Service` | `Service` impl | `service.rs:971-1040` |
| **데이터 계층** | `@Repository` / JPA | `Entity::find()` | SeaORM 체이닝 쿼리 |
| **데이터 전송** | DTO Class/Record | struct (Serialize) | `dto.rs:506-541` |
| **인증** | Spring Security Filter | `AuthUser` Extractor | `handler.rs:458` |

---

## 2. Query Parameter 처리 (Controller vs Handler)

**Spring (Java)**
```java
@GetMapping("/search")
public ResponseEntity<...> search(
    @RequestParam(required = false) String keyword
    // required = false로 해야 누락 시 커스텀 에러 반환 가능
) {
    if (keyword == null || keyword.trim().isEmpty()) {
        throw new SearchKeywordException("검색어를 입력해주세요.");
    }
    ...
}
```

**Rust (Axum)**
```rust
#[derive(Deserialize, IntoParams)]
pub struct SearchQueryParams {
    pub keyword: Option<String>,  // Option = required = false
}

pub async fn search_retrospects(
    user: AuthUser,
    State(state): State<AppState>,
    Query(params): Query<SearchQueryParams>,  // 구조체 단위 매핑
) -> Result<Json<BaseResponse<Vec<SearchRetrospectItem>>>, AppError> {
    let user_id = user.0.sub.parse()...?;
    let result = RetrospectService::search_retrospects(state, user_id, params).await?;
    Ok(Json(BaseResponse::success_with_message(result, "검색을 성공했습니다.")))
}
```

**핵심 차이점:**
- Spring은 `@RequestParam` 어노테이션으로 개별 파라미터를 받지만, Axum은 `Query<T>`를 통해 **구조체 단위**로 매핑
- `Option<String>` = Spring의 `required = false` (누락 시에도 핸들러 실행)
- 검증은 별도 함수(`validate_search_keyword`)에서 수행하여 `SEARCH4001` 커스텀 에러 반환

---

## 3. 데이터 접근 패턴 (Service Layer)

이 API는 JPA의 JOIN/Fetch Join 대신 **3개의 순차 쿼리 + HashMap 인메모리 매핑**을 사용합니다.

**Spring (JPA/QueryDSL)**
```java
@Transactional(readOnly = true)
public List<SearchRetrospectItem> search(Long userId, String keyword) {
    // 1. 소속 팀 조회
    List<Long> teamIds = memberTeamRepository.findTeamIdsByMemberId(userId);
    if (teamIds.isEmpty()) return List.of();

    // 2. 팀 정보 -> Map
    Map<Long, String> teamMap = teamRepository.findAllById(teamIds)
        .stream().collect(Collectors.toMap(Team::getId, Team::getName));

    // 3. LIKE 검색
    List<Retrospect> results = retrospectRepository
        .findByTeamIdInAndTitleContainingOrderByStartTimeDesc(teamIds, keyword);

    // 4. DTO 변환
    return results.stream().map(r -> new SearchRetrospectItem(
        r.getId(),
        r.getTitle(),                          // DB: title -> API: projectName
        teamMap.getOrDefault(r.getTeamId(), ""),
        r.getRetrospectMethod(),
        r.getStartTime().format(DateTimeFormatter.ISO_LOCAL_DATE),
        r.getStartTime().format(DateTimeFormatter.ofPattern("HH:mm"))
    )).collect(Collectors.toList());
}
```

**Rust (SeaORM)**
```rust
pub async fn search_retrospects(
    state: AppState,
    user_id: i64,
    params: SearchQueryParams,
) -> Result<Vec<SearchRetrospectItem>, AppError> {
    // 1. 키워드 검증 (trim, 빈값, 100자)
    let keyword = Self::validate_search_keyword(params.keyword.as_deref())?;

    // 2. 소속 팀 조회 (member_team 중간 테이블)
    let user_teams = member_team::Entity::find()
        .filter(member_team::Column::MemberId.eq(user_id))
        .all(&state.db).await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    if user_teams.is_empty() { return Ok(vec![]); }
    let team_ids: Vec<i64> = user_teams.iter().map(|mt| mt.team_id).collect();

    // 3. 팀 이름 HashMap (JPA: findAllById + Collectors.toMap)
    let teams = team::Entity::find()
        .filter(team::Column::TeamId.is_in(team_ids.clone()))
        .all(&state.db).await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    let team_map: HashMap<i64, String> =
        teams.iter().map(|t| (t.team_id, t.name.clone())).collect();

    // 4. LIKE 검색 + 정렬
    let retrospects = retrospect::Entity::find()
        .filter(retrospect::Column::TeamId.is_in(team_ids))
        .filter(retrospect::Column::Title.contains(&keyword))  // LIKE '%keyword%'
        .order_by_desc(retrospect::Column::StartTime)
        .order_by_desc(retrospect::Column::RetrospectId)       // 안정 정렬
        .all(&state.db).await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    // 5. DTO 변환 (Java Stream과 동일)
    let items: Vec<SearchRetrospectItem> = retrospects
        .iter()
        .map(|r| SearchRetrospectItem {
            retrospect_id: r.retrospect_id,
            project_name: r.title.clone(),
            team_name: team_map.get(&r.team_id).cloned().unwrap_or_default(),
            retrospect_method: r.retrospect_method.clone(),
            retrospect_date: r.start_time.format("%Y-%m-%d").to_string(),
            retrospect_time: r.start_time.format("%H:%M").to_string(),
        })
        .collect();

    Ok(items)
}
```

**Spring 관점 해석:**
1. **`member_team` 조회** = `memberTeamRepository.findTeamIdsByMemberId()` -- N:M 중간 테이블 접근
2. **HashMap 구축** = `Collectors.toMap()` -- ID to Name 매핑
3. **`.filter().filter()`** = QueryDSL의 `.where()` 체이닝 (AND 조건)
4. **`.contains(keyword)`** = JPA의 `Containing` / QueryDSL의 `.contains()` -- LIKE 검색
5. **`.iter().map().collect()`** = Java Stream의 `.stream().map().collect()`
6. **`unwrap_or_default()`** = Java의 `Map.getOrDefault(key, "")`

---

## 4. 검증 패턴 비교

**Spring:**
```java
// Controller에서 @Valid + DTO Bean Validation 또는 서비스에서 직접 검증
if (keyword == null || keyword.trim().isEmpty()) {
    throw new SearchKeywordException("검색어를 입력해주세요.");
}
if (keyword.trim().length() > 100) {
    throw new SearchKeywordException("검색어는 최대 100자까지 입력 가능합니다.");
}
keyword = keyword.trim();
```

**Rust:**
```rust
fn validate_search_keyword(keyword: Option<&str>) -> Result<String, AppError> {
    let trimmed = keyword.unwrap_or("").trim().to_string();
    if trimmed.is_empty() {
        return Err(AppError::SearchKeywordInvalid("검색어를 입력해주세요.".to_string()));
    }
    if trimmed.chars().count() > 100 {
        return Err(AppError::SearchKeywordInvalid("검색어는 최대 100자까지 입력 가능합니다.".to_string()));
    }
    Ok(trimmed)
}
```

**핵심 차이:**
- Spring은 `throw`로 예외를 던지지만, Rust는 `Err()`를 반환하고 `?`로 전파
- `chars().count()`는 `length()`와 달리 유니코드 문자 수를 카운팅 (Java의 `length()`는 UTF-16 코드 유닛 수)

---

## 5. 요약

| 관점 | Spring | Rust (이 프로젝트) |
|------|--------|-------------------|
| 파라미터 | `@RequestParam(required = false)` | `Option<String>` in DTO struct |
| 검색 쿼리 | JPA `Containing` / QueryDSL `.contains()` | SeaORM `.contains()` |
| 팀 이름 조회 | Fetch Join 또는 별도 조회 | 3개 순차 쿼리 + HashMap |
| 에러 | `throw SearchKeywordException` | `Err(AppError::SearchKeywordInvalid(...))` |
| DTO 매핑 | Stream API | `iter().map().collect()` |
| 날짜 포맷 | `DateTimeFormatter` | `chrono::format()` |

이 API의 비즈니스 로직은 Spring의 검색 서비스와 거의 동일합니다. 주요 차이는 명시적 에러 반환(`Result`), 수동 HashMap 매핑(JPA Lazy Loading 없음), 그리고 유니코드 안전 문자열 처리(`chars().count()`)입니다.
