# Spring 개발자를 위한 Rust API 구현 가이드 (API-018)

이 문서는 **API-018 (회고 참고자료 목록 조회)**의 Rust 구현을 Spring(Java/Kotlin) 개발자의 관점에서 이해하기 쉽게 설명합니다.
API-018은 비교적 단순한 **조회(Read) 로직**으로 구성되어 있어, Rust의 데이터 조회 및 매핑 패턴을 익히기에 적합합니다.

## 1. 아키텍처 매핑

Spring의 계층형 아키텍처와 거의 동일한 구조를 가집니다.

| 역할 | Rust (현재 프로젝트) | Spring (JVM) | 비고 |
|------|-------------------|--------------|------|
| **Web Layer** | `handler.rs` (Axum) | `Controller` | HTTP 요청 처리 및 파라미터 검증 |
| **Service Layer** | `service.rs` | `Service` | 비즈니스 로직 및 권한 검사 |
| **Persistence** | `SeaORM` (Entity) | `JPA Repository` | DB 쿼리 실행 |
| **DTO** | `dto.rs` (ReferenceItem) | `DTO` | 응답 데이터 구조 |

---

## 2. 코드 상세 분석

### 2.1 Handler Layer (`handler.rs`)

**Rust 코드:**
```rust
// GET /api/v1/retrospects/{retrospectId}/references
pub async fn list_references(
    user: AuthUser,                    // 1. 인증 정보 자동 주입 (Principal)
    State(state): State<AppState>,     // 2. DI (Service/Repository 의존성)
    Path(retrospect_id): Path<i64>,    // 3. @PathVariable
) -> Result<Json<BaseResponse<Vec<ReferenceItem>>>, AppError> { // 4. 반환 타입
    
    // 유효성 검증
    if retrospect_id < 1 {
        return Err(AppError::BadRequest("...".to_string()));
    }

    let user_id = user.user_id()?;

    // 서비스 호출
    let result = RetrospectService::list_references(state, user_id, retrospect_id).await?;

    // 응답 래핑 (ResponseEntity.ok(...))
    Ok(Json(BaseResponse::success_with_message(result, "...")))
}
```

**Spring 관점 해석:**

1.  **`user: AuthUser`**: Spring Security의 `SecurityContextHolder`에서 가져오는 `Authentication` 객체와 유사합니다. 요청이 핸들러에 도달하기 전, 미들웨어(Interceptor/Filter) 레벨에서 토큰을 검증하고 추출된 정보를 주입해줍니다.
2.  **`State(state)`**: Spring Bean을 주입받는 것과 같습니다. DB 커넥션 풀(`DatabaseConnection`)을 포함하고 있어, 이를 통해 쿼리를 실행할 수 있습니다.
3.  **`Path`**: `@PathVariable`과 100% 동일한 역할을 합니다.
4.  **`Result`**: 예외를 던지는 대신, 성공(`Ok`)과 실패(`Err`)를 값으로 반환합니다. 이 패턴은 Go, Swift, Kotlin(Result 타입) 등 최신 언어들의 트렌드입니다.

---

### 2.2 Service Layer (`service.rs`)

**Rust 코드:**
```rust
pub async fn list_references(
    state: AppState,
    user_id: i64,
    retrospect_id: i64,
) -> Result<Vec<ReferenceItem>, AppError> { // List<ReferenceItem> 반환
    
    // 1. 권한 검사 (팀 멤버 여부 등)
    // find_retrospect_for_member 내부에서 조회 실패 시 Error를 반환하여 조기 리턴
    let _retrospect_model =
        Self::find_retrospect_for_member(&state, user_id, retrospect_id).await?;

    // 2. 목록 조회 (JPA: findAllByRetrospectIdOrderByReferenceIdAsc)
    let references = retro_reference::Entity::find()
        .filter(retro_reference::Column::RetrospectId.eq(retrospect_id)) // where
        .order_by_asc(retro_reference::Column::RetroReferenceId)       // order by
        .all(&state.db)                                                // fetch
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    // 3. DTO 변환 (Stream API와 유사)
    let result: Vec<ReferenceItem> = references
        .into_iter()
        .map(|r| ReferenceItem {
            reference_id: r.retro_reference_id,
            url_name: r.title,
            url: r.url,
        })
        .collect();

    Ok(result)
}
```

**Spring 관점 해석:**

1.  **`Entity::find()` 체이닝**: Spring Data JPA의 Query Creation과 유사하지만, 명시적으로 쿼리를 조립합니다. QueryDSL과 더 비슷하다고 볼 수 있습니다.
    *   `.filter(...)` = `where`
    *   `.order_by_asc(...)` = `orderBy ... asc`
    *   `.all(&db)` = `findAll()` / `getResultList()`
2.  **`await`**: 비동기 처리를 위한 키워드입니다. Spring WebFlux의 `Mono/Flux`를 `block()` 없이 처리하는 것과 비슷하지만, 코드는 동기 방식처럼 순차적으로 작성됩니다.
3.  **DTO 매핑 (`map/collect`)**: Java Stream API와 완벽히 동일합니다.
    *   Java: `references.stream().map(r -> new ReferenceItem(...)).collect(Collectors.toList())`
    *   Rust: `references.into_iter().map(|r| ReferenceItem { ... }).collect()`

---

### 2.3 DB 엔티티와 DTO

**Entity (`retro_reference`)**:
```rust
// JPA Entity와 유사
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "retro_reference")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub retro_reference_id: i64,
    pub title: String,
    pub url: String,
    pub retrospect_id: i64,
}
```

**DTO (`ReferenceItem`)**:
```rust
// Lombok @Data, @Builder와 유사
#[derive(Debug, Serialize, ToSchema)] // Getter, toString, Swagger 스키마 자동생성
#[serde(rename_all = "camelCase")] // JSON 필드명을 camelCase로 자동 변환
pub struct ReferenceItem {
    /// 자료 고유 식별자
    pub reference_id: i64,
    /// 자료 별칭 (예: 깃허브 레포지토리)
    pub url_name: String,
    /// 참고자료 주소
    pub url: String,
}
```

*   `#[derive(Debug, Serialize, ToSchema)]`: 실제 코드에서는 `Serialize`만 포함되어 있고 `Deserialize`는 없습니다 (응답 전용 DTO이므로). `ToSchema`는 Swagger UI 자동 생성을 위한 utoipa 매크로입니다.
*   `#[serde(rename_all = "camelCase")]`: Jackson 라이브러리의 `@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)`와 반대되는 설정으로, Rust의 snake_case 필드명을 JSON 응답 시 자동으로 camelCase로 변환해줍니다. (예: `reference_id` -> `referenceId`)

---

## 3. 핵심 요약

이 API는 **Spring Boot + Spring Data JPA + Lombok** 스택을 사용하는 개발자가 아래와 같이 작성하는 코드와 논리적으로 완전히 동일합니다.

```java
@Transactional(readOnly = true)
public List<ReferenceItem> listReferences(Long userId, Long retrospectId) {
    // 1. 권한 검사
    retrospectRepository.findByIdAndMemberId(retrospectId, userId)
        .orElseThrow(() -> new AccessDeniedException("..."));

    // 2. 조회
    List<RetroReference> refs = referenceRepository.findAllByRetrospectIdOrderByIdAsc(retrospectId);

    // 3. 변환
    return refs.stream()
        .map(ref -> new ReferenceItem(ref.getId(), ref.getTitle(), ref.getUrl()))
        .collect(Collectors.toList());
}
```

Rust는 언어적 특성(메모리 안전성, 에러 처리 방식)만 다를 뿐, **웹 애플리케이션을 구성하는 논리와 패턴은 Spring과 매우 유사**합니다.

---

## 4. 스펙 vs 구현 차이 (Spring 관점)

Spring 프로젝트에서도 발생할 수 있는 **명세서와 구현의 차이**가 이 API에 존재합니다.

| 항목 | API 명세서 | 실제 구현 | Spring에서의 유사 패턴 |
|------|-----------|-----------|----------------------|
| 403 에러 | 별도 `TEAM4031` 에러 정의 | 404(`RETRO4041`)로 통합 | `@PreAuthorize` 실패 시 403 대신 404로 처리하여 리소스 존재 여부 노출 방지 |
| urlName | "자료 별칭" (최대 50자) | URL 원본 값 그대로 (최대 2,048자) | Entity의 `title` 필드에 별칭 대신 URL을 저장 |

Spring의 `@ControllerAdvice`와 마찬가지로, Rust에서도 에러 타입(`AppError`)이 `IntoResponse`를 구현하여 에러 응답 형식을 중앙에서 관리합니다. 이 API에서 403이 발생하지 않는 이유는 서비스 레이어의 `find_retrospect_for_member` 헬퍼가 접근 권한 실패를 의도적으로 `RetrospectNotFound`(404)로 매핑하기 때문입니다.
