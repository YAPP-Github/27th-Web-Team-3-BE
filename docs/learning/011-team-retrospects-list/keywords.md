# 학습 키워드: 팀 회고 목록 조회 (API-010)

## 1. Axum Path Extractor

URL 경로의 동적 세그먼트를 타입으로 추출하는 Axum 기능.

```rust
// handler.rs:91
Path(team_id): Path<i64>
```

- `/api/v1/teams/{teamId}/retrospects`에서 `{teamId}` 부분을 `i64`로 파싱
- 파싱 실패 시 Axum이 자동으로 400 에러 반환
- 구조 분해 패턴(`Path(team_id)`)으로 내부 값 바인딩

## 2. Axum State Extractor

애플리케이션 전역 상태(DB 커넥션 풀 등)를 핸들러에 주입하는 패턴.

```rust
// handler.rs:90
State(state): State<AppState>
```

- `AppState`에 DB 커넥션(`state.db`), 설정(`state.config`) 등이 포함
- Axum 라우터 생성 시 `.with_state(app_state)`로 등록

## 3. FromRequestParts trait (커스텀 Extractor)

Axum 핸들러의 매개변수로 사용할 수 있는 커스텀 추출기를 만드는 trait.

```rust
// auth.rs:22-55
#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;
    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> { ... }
}
```

- 핸들러에서 `user: AuthUser`만 선언하면 자동으로 JWT 인증 수행
- `type Rejection`으로 실패 시 에러 타입 지정

## 4. From/Into trait (타입 변환)

Rust 표준 라이브러리의 타입 변환 trait. `From`을 구현하면 `Into`가 자동으로 제공됩니다.

```rust
// dto.rs:198-208
impl From<RetrospectModel> for TeamRetrospectListItem {
    fn from(model: RetrospectModel) -> Self { ... }
}

// service.rs:315 (사용부)
retrospects.into_iter().map(|r| r.into()).collect();
```

- 엔티티 모델 -> 응답 DTO 변환에 활용
- `.into()` 호출로 `From` 구현이 자동 호출됨

## 5. SeaORM EntityTrait::find / find_by_id

SeaORM에서 DB 조회를 위한 기본 메서드.

```rust
// service.rs:279 (PK 조회)
team::Entity::find_by_id(team_id).one(&state.db).await

// service.rs:305 (전체 조회 + 필터)
retrospect::Entity::find()
    .filter(retrospect::Column::TeamId.eq(team_id))
    .all(&state.db).await
```

- `find_by_id()`: Primary Key 기반 단건 조회
- `find()`: 전체 조회 시작, `.filter()` 등으로 조건 추가
- `.one()`: `Option<Model>` 반환, `.all()`: `Vec<Model>` 반환

## 6. SeaORM QueryOrder

조회 결과의 정렬 순서를 지정하는 SeaORM trait.

```rust
// service.rs:307-308
.order_by_desc(retrospect::Column::StartTime)
.order_by_desc(retrospect::Column::RetrospectId)
```

- `order_by_desc` / `order_by_asc` 메서드로 정렬 조건 추가
- 체이닝으로 다중 정렬 지원 (SQL `ORDER BY col1 DESC, col2 DESC`)
- `QueryOrder` trait이 제공하는 메서드

## 7. SeaORM ColumnTrait::eq / filter

WHERE 조건을 구성하는 SeaORM 메서드.

```rust
// service.rs:292-293
.filter(member_team::Column::MemberId.eq(user_id))
.filter(member_team::Column::TeamId.eq(team_id))
```

- `Column::FieldName.eq(value)`: `WHERE field_name = value` 생성
- `.filter()` 체이닝으로 AND 조건 추가
- `.contains()`, `.gt()`, `.lt()` 등 다양한 비교 메서드 제공

## 8. serde rename_all = "camelCase"

Rust의 snake_case 필드를 JSON 직렬화 시 camelCase로 변환하는 serde 어트리뷰트.

```rust
// dto.rs:184
#[serde(rename_all = "camelCase")]
pub struct TeamRetrospectListItem {
    pub retrospect_id: i64,   // -> "retrospectId"
    pub project_name: String, // -> "projectName"
}
```

- Serialize/Deserialize 모두에 적용
- 프론트엔드 JavaScript의 camelCase 규칙과 호환

## 9. Result + ? 연산자 (에러 전파)

함수가 `Result<T, E>`를 반환할 때, `?` 연산자로 에러를 간결하게 전파하는 Rust 패턴.

```rust
// service.rs:279-282
let team_exists = team::Entity::find_by_id(team_id)
    .one(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;  // DB 에러 -> AppError 변환 후 전파
```

- `.map_err()`: 에러 타입 변환 (SeaORM DbErr -> AppError)
- `?`: `Err`이면 즉시 반환, `Ok`면 내부 값 추출
- 이 프로젝트에서는 `unwrap()` / `expect()` 사용을 금지하고 `?`를 사용

## 10. chrono DateTime format

날짜/시간 값을 문자열로 포맷팅하는 chrono 크레이트 메서드.

```rust
// dto.rs:204-205
retrospect_date: model.start_time.format("%Y-%m-%d").to_string(),  // "2026-01-20"
retrospect_time: model.start_time.format("%H:%M").to_string(),     // "10:00"
```

- `%Y`: 4자리 연도, `%m`: 2자리 월, `%d`: 2자리 일
- `%H`: 24시간 형식 시, `%M`: 2자리 분
- SeaORM 엔티티의 `DateTime` 타입을 API 응답 문자열로 변환

## 11. into_iter + map + collect (이터레이터 패턴)

Rust의 이터레이터 체이닝으로 컬렉션을 변환하는 함수형 패턴.

```rust
// service.rs:314-315
let result: Vec<TeamRetrospectListItem> =
    retrospects.into_iter().map(|r| r.into()).collect();
```

- `into_iter()`: 소유권을 이전하는 이터레이터 생성 (`iter()`와 달리 원본 소멸)
- `.map(|r| r.into())`: 각 요소를 `From` trait으로 변환
- `.collect()`: 이터레이터를 `Vec`으로 수집 (타입 추론으로 대상 타입 결정)

## 12. utoipa::path (Swagger 문서 생성)

API 핸들러에 Swagger/OpenAPI 문서를 자동 생성하는 매크로.

```rust
// handler.rs:69-87
#[utoipa::path(
    get,
    path = "/api/v1/teams/{teamId}/retrospects",
    params(("teamId" = i64, Path, description = "조회를 원하는 팀의 고유 ID")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "...", body = SuccessTeamRetrospectListResponse),
        (status = 400, description = "...", body = ErrorResponse),
    ),
    tag = "Retrospect"
)]
```

- `params`: Path/Query 파라미터 문서화
- `security`: 인증 요구사항 명시
- `responses`: 가능한 응답 상태 코드와 body 타입 지정
- `tag`: Swagger UI에서 API 그룹화
