# 핵심 개념: 회고 참고자료 목록 조회 (API-018)

## 1. 헬퍼 함수 재사용 패턴: `find_retrospect_for_member`

### 개념

여러 API에서 반복적으로 필요한 "회고 조회 + 팀 멤버십 확인" 로직을 하나의 헬퍼 함수로 추출하여 재사용하는 패턴입니다.

**소스**: `codes/server/src/domain/retrospect/service.rs:323~352`

```rust
async fn find_retrospect_for_member(
    state: &AppState,
    user_id: i64,
    retrospect_id: i64,
) -> Result<retrospect::Model, AppError> {
    // 1. 회고 존재 확인
    // 2. 팀 멤버십 확인
    // 3. 두 실패를 동일한 404로 통합
}
```

### 재사용 현황

이 헬퍼 함수는 프로젝트 내 여러 API 서비스에서 호출됩니다:

| API | 서비스 함수 | 소스 위치 (service.rs) |
|-----|-------------|----------------------|
| API-014 참석자 등록 | `create_participant` | 라인 362 |
| **API-018 참고자료 조회** | `list_references` | **라인 437** |
| API-020 답변 조회 | `list_responses` | 라인 1056 |
| API-021 내보내기 | `export_retrospect` | 라인 1152 |
| API-022 분석 | `analyze_retrospective` | 라인 1863 |

### 학습 포인트

- **DRY 원칙**: "Don't Repeat Yourself" 원칙에 따라 공통 로직을 헬퍼로 추출
- **보안 통합 처리**: "존재하지 않음"과 "접근 권한 없음"을 동일한 404로 반환하여 비멤버에게 회고 존재 여부를 노출하지 않음
- **private 메서드**: `async fn`(pub 없음)으로 선언하여 서비스 내부에서만 사용 가능

---

## 2. 간단한 조회 API 구조

### 개념

API-018은 프로젝트 내에서 가장 단순한 형태의 조회 API입니다. 페이지네이션이 없고, 필터링 파라미터도 없으며, 단순히 특정 회고의 참고자료를 전부 반환합니다.

### 핸들러-서비스 분리 패턴

**핸들러** (`handler.rs:181~203`):
- 입력 검증 (Path Parameter 유효성)
- 사용자 인증 (JWT에서 user_id 추출)
- 서비스 호출 후 응답 래핑

**서비스** (`service.rs:430~458`):
- 비즈니스 로직 (회고 존재 + 멤버십 확인)
- 데이터 조회 (DB 쿼리)
- DTO 변환

이 분리를 통해 핸들러에는 HTTP 관련 처리만, 서비스에는 순수한 비즈니스 로직만 위치합니다.

### 비교: 복잡한 조회 API와의 차이

| 항목 | API-018 (참고자료 조회) | API-020 (답변 조회) |
|------|------------------------|---------------------|
| 페이지네이션 | 없음 | 커서 기반 |
| 필터링 | 없음 | 카테고리 필터 |
| 정렬 | referenceId 오름차순 고정 | responseId 내림차순 |
| 핸들러 검증 | Path Parameter만 | Path + Query Parameters |
| 응답 구조 | `Vec<ReferenceItem>` | `ResponsesListResponse` (hasNext, nextCursor 포함) |

---

## 3. SeaORM 엔티티 필터 + 정렬 패턴

### 개념

SeaORM을 사용하여 특정 조건으로 필터링하고 정렬하여 조회하는 패턴입니다.

**소스**: `codes/server/src/domain/retrospect/service.rs:440~445`

```rust
let references = retro_reference::Entity::find()
    .filter(retro_reference::Column::RetrospectId.eq(retrospect_id))
    .order_by_asc(retro_reference::Column::RetroReferenceId)
    .all(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```

### 구성 요소 설명

| 메서드 | 역할 | 비고 |
|--------|------|------|
| `Entity::find()` | SELECT 쿼리 시작 | `retro_reference` 테이블 대상 |
| `.filter(Column.eq(value))` | WHERE 조건 추가 | `retrospect_id = ?` |
| `.order_by_asc(Column)` | ORDER BY ASC | 등록 순서(오름차순) |
| `.all(&state.db)` | 전체 결과 반환 | `Vec<Model>` 타입 |
| `.map_err(...)` | 에러 타입 변환 | SeaORM DbErr -> AppError |

### 학습 포인트

- SeaORM의 `Entity::find()`는 빌더 패턴으로 쿼리를 조립
- `.all()`은 모든 결과를 반환하고, `.one()`은 단일 결과를 반환
- `.map_err()`로 라이브러리 에러를 애플리케이션 에러로 변환하는 것이 Rust 에러 처리의 핵심 패턴

---

## 4. 엔티티-DTO 변환 패턴 (수동 매핑)

### 개념

DB 엔티티 모델의 필드를 API 응답 DTO의 필드로 변환하는 방법입니다. API-018에서는 `into_iter().map().collect()` 패턴을 사용합니다.

**소스**: `codes/server/src/domain/retrospect/service.rs:448~455`

```rust
let result: Vec<ReferenceItem> = references
    .into_iter()
    .map(|r| ReferenceItem {
        reference_id: r.retro_reference_id,
        url_name: r.title,
        url: r.url,
    })
    .collect();
```

### 필드 매핑 관계

**엔티티** (`entity/retro_reference.rs:4~12`):
```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "retro_reference")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub retro_reference_id: i64,  // PK, 자동 증가
    pub title: String,            // URL 값이 그대로 복사됨 (현재 url과 동일)
    pub url: String,              // 원본 URL
    pub retrospect_id: i64,       // FK
}
```

**DTO** (`dto.rs:251~260`):
```rust
pub struct ReferenceItem {
    pub reference_id: i64,   // <- retro_reference_id
    pub url_name: String,    // <- title
    pub url: String,         // <- url (동일)
}
```

### 비교: From 트레잇 구현 방식

프로젝트 내 다른 DTO는 `From` 트레잇을 구현하여 변환합니다 (예: `dto.rs:198~208`의 `TeamRetrospectListItem`):

```rust
impl From<RetrospectModel> for TeamRetrospectListItem {
    fn from(model: RetrospectModel) -> Self { ... }
}
```

API-018에서는 필드 이름이 달라 직접 매핑하는 클로저를 사용한 점이 차이점입니다.

---

## 5. 참고자료 저장 시 title 생성 로직

### 개념

회고 생성(API-001) 시 참고자료 URL을 저장할 때, `title` 필드에 URL 값을 그대로 복사하여 저장합니다.

**소스**: `codes/server/src/domain/retrospect/service.rs:159~172`

```rust
for url in &req.reference_urls {
    let reference_model = retro_reference::ActiveModel {
        title: Set(url.clone()),    // URL을 title로 그대로 사용
        url: Set(url.clone()),      // 원본 URL
        retrospect_id: Set(retrospect_id),
        ..Default::default()
    };
    reference_model.insert(&txn).await...;
}
```

따라서 조회 시 `url_name`과 `url` 값은 현재 동일한 값을 가집니다. 향후 사용자가 직접 별칭을 입력하는 기능이 추가되면 `title` 필드에 별도 값이 저장될 수 있습니다.

---

## 6. 스펙 vs 구현 차이점 (Spec-Implementation Discrepancies)

### 개념

API 명세서(`docs/api-specs/018-retrospect-references-list.md`)와 실제 구현 사이에 의도적/비의도적 차이가 존재합니다. 이 차이를 인지하는 것은 프론트엔드 연동 및 향후 유지보수에 중요합니다.

### 차이점 상세

#### 6-1. 403 Forbidden 에러가 실제로는 반환되지 않음

**명세서**: `403 Forbidden (TEAM4031)` -- "해당 팀에 접근 권한이 없습니다."를 별도 에러로 정의

**구현**: `find_retrospect_for_member` 헬퍼(service.rs:323~352)에서 팀 멤버가 아닌 경우에도 `AppError::RetrospectNotFound`(RETRO4041, 404)를 반환

```rust
// service.rs:345~348 -- 멤버십 확인 실패 시에도 404 반환
if is_member.is_none() {
    return Err(AppError::RetrospectNotFound(
        "존재하지 않는 회고이거나 접근 권한이 없습니다.".to_string(),
    ));
}
```

**의도**: 비인가 사용자에게 회고의 존재 여부 자체를 노출하지 않기 위한 **보안 통합 처리** 패턴입니다. OWASP에서 권장하는 "Information Leakage 방지" 원칙에 해당합니다.

#### 6-2. urlName 필드값과 명세서 설명 불일치

**명세서**: `urlName`을 "자료 별칭 (예: 깃허브 레포지토리)"으로 정의하며 "최대 50자" 제약 조건을 명시

**구현**: 회고 생성 시 `title: Set(url.clone())`(service.rs:162)으로 URL 원본 값을 그대로 복사하여 저장. 별도의 50자 제약은 없고, URL 최대 길이(2,048자)가 실질적 제약

**결과**: 현재 `urlName`과 `url`은 항상 동일한 값을 반환합니다. 명세서의 "별칭" 개념은 아직 구현되지 않은 상태입니다.

#### 6-3. 404 에러 메시지 차이

**명세서**: "존재하지 않는 회고 세션입니다."

**구현**: "존재하지 않는 회고이거나 접근 권한이 없습니다." (service.rs:334, 347)

구현의 메시지가 더 포괄적이며, 403과 404를 통합한 의미를 내포합니다.

### 학습 포인트

- API 명세서는 기획 단계의 **이상적인 설계**를 반영하고, 구현은 **보안 및 현실적 제약**을 반영하므로 차이가 발생할 수 있음
- 프론트엔드 개발자는 명세서가 아닌 **실제 구현의 응답**을 기준으로 에러 핸들링을 구현해야 함
- 이러한 차이가 발견되면 명세서를 업데이트하거나, 학습 문서에 차이점을 명시하여 추후 혼란을 방지해야 함
