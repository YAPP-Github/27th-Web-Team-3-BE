# 핵심 개념: 팀 회고 목록 조회 (API-010)

## Axum Path Extractor

Axum의 `Path` extractor는 URL 경로의 동적 세그먼트를 추출합니다.

**파일**: `src/domain/retrospect/handler.rs:91`

```rust
pub async fn list_team_retrospects(
    user: AuthUser,
    State(state): State<AppState>,
    Path(team_id): Path<i64>,         // {teamId}를 i64로 자동 파싱
) -> Result<Json<BaseResponse<Vec<TeamRetrospectListItem>>>, AppError> {
```

- 라우터에서 `/api/v1/teams/{teamId}/retrospects`로 등록된 경로의 `{teamId}` 부분을 추출
- `Path<i64>`는 경로 값을 `i64`로 역직렬화 시도
- 파싱 실패 시 자동으로 400 에러 반환 (Axum 내장)
- 구조 분해 패턴 `Path(team_id)`로 내부 값을 바로 바인딩

## Axum FromRequestParts (커스텀 Extractor)

핸들러 매개변수에 커스텀 타입을 넣으면 Axum이 자동으로 요청에서 데이터를 추출합니다.

**파일**: `src/utils/auth.rs:9-10, 22-55`

```rust
pub struct AuthUser(pub Claims);  // 뉴타입 패턴

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Authorization 헤더 추출 -> Bearer 확인 -> JWT 디코딩
    }
}
```

- `FromRequestParts` trait을 구현하면 핸들러 인자로 사용 가능
- `type Rejection = AppError`: 추출 실패 시 반환할 에러 타입
- 핸들러에서 `user: AuthUser`만 선언하면 인증 로직이 자동 실행
- 여러 extractor를 순서대로 나열하면 Axum이 차례로 실행

## SeaORM QueryOrder - 다중 정렬

SeaORM에서 `order_by_desc`/`order_by_asc`를 체이닝하여 다중 정렬 조건을 적용합니다.

**파일**: `src/domain/retrospect/service.rs:305-308`

```rust
let retrospects = retrospect::Entity::find()
    .filter(retrospect::Column::TeamId.eq(team_id))
    .order_by_desc(retrospect::Column::StartTime)      // 1차: 시작 시간 내림차순
    .order_by_desc(retrospect::Column::RetrospectId)    // 2차: ID 내림차순 (안정 정렬)
    .all(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```

- `order_by_desc`를 연속 호출하면 SQL `ORDER BY start_time DESC, retrospect_id DESC` 생성
- 1차 정렬 키가 동일할 때 2차 정렬 키로 순서 보장 (안정 정렬)
- `QueryOrder` trait이 `find()` 반환 타입에 구현되어 있어 체이닝 가능

## From trait을 활용한 엔티티-DTO 변환

Rust의 `From` trait을 구현하여 엔티티 모델을 응답 DTO로 변환합니다.

**파일**: `src/domain/retrospect/dto.rs:198-208`

```rust
impl From<RetrospectModel> for TeamRetrospectListItem {
    fn from(model: RetrospectModel) -> Self {
        Self {
            retrospect_id: model.retrospect_id,
            project_name: model.title,             // 필드명 매핑 (title -> project_name)
            retrospect_method: model.retrospect_method,
            retrospect_date: model.start_time.format("%Y-%m-%d").to_string(),  // DateTime -> String
            retrospect_time: model.start_time.format("%H:%M").to_string(),     // DateTime -> String
        }
    }
}
```

- `From<RetrospectModel>`을 구현하면 `.into()` 호출로 변환 가능
- 엔티티의 `title`을 DTO의 `project_name`으로 매핑 (이름 변환)
- `start_time` (DateTime)을 `retrospect_date`와 `retrospect_time`으로 분리
- `chrono`의 `format()` 메서드로 날짜/시간 포맷팅

서비스에서 사용:
```rust
// service.rs:314-315
let result: Vec<TeamRetrospectListItem> =
    retrospects.into_iter().map(|r| r.into()).collect();
```

## serde rename_all = "camelCase"

Rust의 snake_case 필드명을 JSON의 camelCase로 자동 변환합니다.

**파일**: `src/domain/retrospect/dto.rs:183-196`

```rust
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]    // snake_case -> camelCase 자동 변환
pub struct TeamRetrospectListItem {
    pub retrospect_id: i64,           // JSON: "retrospectId"
    pub project_name: String,         // JSON: "projectName"
    pub retrospect_method: RetrospectMethod,  // JSON: "retrospectMethod"
    pub retrospect_date: String,      // JSON: "retrospectDate"
    pub retrospect_time: String,      // JSON: "retrospectTime"
}
```

- `#[serde(rename_all = "camelCase")]` 어트리뷰트로 전체 필드에 일괄 적용
- Serialize 시: `retrospect_id` -> `"retrospectId"`
- 프론트엔드(JavaScript)의 네이밍 규칙과 호환

## BaseResponse 래핑 패턴

모든 API 응답을 공통 구조체로 감싸서 일관된 응답 형식을 보장합니다.

**파일**: `src/utils/response.rs:17-43`

```rust
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseResponse<T: Serialize> {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: Option<T>,
}

impl<T: Serialize> BaseResponse<T> {
    pub fn success_with_message(result: T, message: impl Into<String>) -> Self {
        Self {
            is_success: true,
            code: "COMMON200".to_string(),
            message: message.into(),
            result: Some(result),
        }
    }
}
```

핸들러에서 사용:
```rust
// handler.rs:106-109
Ok(Json(BaseResponse::success_with_message(
    result,
    "팀 내 전체 회고 목록 조회를 성공했습니다.",
)))
```

- 제네릭 `T: Serialize`로 어떤 응답 데이터든 래핑 가능
- `impl Into<String>`으로 `&str`과 `String` 모두 수용
- 모든 API가 동일한 최상위 구조 (`isSuccess`, `code`, `message`, `result`)를 따름

## SeaORM 엔티티 조회 패턴

SeaORM에서 엔티티를 조회하는 기본 패턴입니다.

**파일**: `src/domain/retrospect/service.rs:279-282` (단건 조회)

```rust
// PK 기반 단건 조회
let team_exists = team::Entity::find_by_id(team_id)
    .one(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```

**파일**: `src/domain/retrospect/service.rs:291-296` (조건 조회)

```rust
// 필터 조건 기반 단건 조회
let is_member = member_team::Entity::find()
    .filter(member_team::Column::MemberId.eq(user_id))
    .filter(member_team::Column::TeamId.eq(team_id))
    .one(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```

- `Entity::find_by_id()`: PK로 조회 (WHERE id = ?)
- `Entity::find().filter()`: 임의 조건으로 조회 (WHERE column = ?)
- `.one()`: `Option<Model>` 반환 (0개 또는 1개)
- `.all()`: `Vec<Model>` 반환 (0개 이상)
- `.map_err(|e| AppError::InternalError(...))`: DB 에러를 앱 에러로 변환
- `?` 연산자로 에러 전파
