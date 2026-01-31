# 학습 가이드: API-010 팀 회고 목록 조회

이 문서는 **JVM 기반 Spring만 익숙한 개발자**가 API-010을 “끝까지 이해”할 수 있도록 작성한 학습 가이드입니다. Rust 문법, Axum 프레임워크, SeaORM(ORM) 사용 흐름을 **Spring 개념에 매핑**해서 설명합니다.

---

## 1) 한눈에 보는 기술 매핑 (Spring ↔ Rust/Axum/SeaORM)

| Spring (JVM) | 이 프로젝트 (Rust) | 의미/역할 |
|---|---|---|
| Controller | `handler.rs`의 핸들러 함수 | HTTP 요청 진입점 |
| Service | `service.rs`의 서비스 함수 | 비즈니스 로직 |
| Repository (JPA/QueryDSL) | SeaORM `Entity::find()` | DB 조회 |
| DTO / VO | `dto.rs`의 struct | 요청/응답 데이터 구조 |
| Spring Security Filter | `AuthUser` Extractor | JWT 인증/인가 |
| ResponseEntity | `Json<BaseResponse<T>>` | 표준 응답 래핑 |
| ExceptionHandler | `AppError` + `IntoResponse` | 에러 매핑/응답 |

---

## 2) API-010 기능 요약

- **엔드포인트**: `GET /api/v1/teams/{teamId}/retrospects`
- **기능**: 특정 팀의 회고 목록 전체를 최신순으로 조회
- **인증**: JWT Bearer 토큰 필수
- **정렬**: `start_time DESC`, 동일 시각이면 `retrospect_id DESC`

---

## 3) 요청 흐름 (Spring 기준으로 이해하기)

```
클라이언트 요청
  -> AuthUser Extractor (Spring Security Filter 역할)
  -> Handler (Controller 역할)
  -> Service (Service 역할)
    -> SeaORM Entity 조회 (Repository 역할)
  -> BaseResponse 래핑 (ResponseEntity 역할)
```

### 단계별 실제 코드 위치

1. **JWT 인증**: `codes/server/src/utils/auth.rs`
2. **핸들러(Controller)**: `codes/server/src/domain/retrospect/handler.rs`
3. **서비스(Service)**: `codes/server/src/domain/retrospect/service.rs`
4. **ORM/Entity**: `codes/server/src/domain/retrospect/entity/retrospect.rs`
5. **DTO**: `codes/server/src/domain/retrospect/dto.rs`
6. **공통 응답 구조**: `codes/server/src/utils/response.rs`
7. **에러 처리**: `codes/server/src/utils/error.rs`

---

## 4) 코드 흐름 상세 (API-010)

### 4-1. 인증 단계 (Spring Security Filter 역할)

**파일**: `src/utils/auth.rs`

```rust
pub struct AuthUser(pub Claims);

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    async fn from_request_parts(...) -> Result<Self, Self::Rejection> {
        // Authorization: Bearer <token> 추출
        // decode_access_token(...)으로 검증
    }
}
```

- `AuthUser`가 **커스텀 Extractor**로 등록되어 있어, 핸들러에 `user: AuthUser`만 적으면 자동 인증 수행.
- 실패 시 `AppError::Unauthorized` 반환 → `401` 응답으로 변환.

**Spring 대응**: `OncePerRequestFilter`에서 JWT 파싱/검증 후 SecurityContext에 넣는 것과 유사.

---

### 4-2. Handler (Controller 역할)

**파일**: `src/domain/retrospect/handler.rs`

```rust
pub async fn list_team_retrospects(
    user: AuthUser,
    State(state): State<AppState>,
    Path(team_id): Path<i64>,
) -> Result<Json<BaseResponse<Vec<TeamRetrospectListItem>>>, AppError> {
    if team_id < 1 {
        return Err(AppError::BadRequest("팀 ID는 1 이상이어야 합니다.".to_string()));
    }

    let user_id = user.user_id()?;
    let result = RetrospectService::list_team_retrospects(state, user_id, team_id).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "팀 내 전체 회고 목록 조회를 성공했습니다.",
    )))
}
```

핵심 포인트:
- `Path(team_id): Path<i64>` → URL `{teamId}`를 자동 파싱 (Spring의 `@PathVariable`)
- `State(state): State<AppState>` → 전역 상태 주입 (Spring의 `@Autowired` 느낌)
- `user.user_id()?` → JWT Claims에서 사용자 ID 추출
- `Result<_, AppError>` → 에러가 발생하면 자동으로 HTTP 에러 응답

---

### 4-3. Service (비즈니스 로직)

**파일**: `src/domain/retrospect/service.rs`

```rust
pub async fn list_team_retrospects(
    state: AppState,
    user_id: i64,
    team_id: i64,
) -> Result<Vec<TeamRetrospectListItem>, AppError> {
    // 1) 팀 존재 여부
    let team_exists = team::Entity::find_by_id(team_id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    if team_exists.is_none() {
        return Err(AppError::TeamNotFound("존재하지 않는 팀입니다.".to_string()));
    }

    // 2) 팀 멤버십 여부
    let is_member = member_team::Entity::find()
        .filter(member_team::Column::MemberId.eq(user_id))
        .filter(member_team::Column::TeamId.eq(team_id))
        .one(&state.db)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    if is_member.is_none() {
        return Err(AppError::TeamAccessDenied("해당 팀에 접근 권한이 없습니다.".to_string()));
    }

    // 3) 회고 목록 조회
    let retrospects = retrospect::Entity::find()
        .filter(retrospect::Column::TeamId.eq(team_id))
        .order_by_desc(retrospect::Column::StartTime)
        .order_by_desc(retrospect::Column::RetrospectId)
        .all(&state.db)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    // 4) DTO 변환
    let result = retrospects.into_iter().map(|r| r.into()).collect();

    Ok(result)
}
```

Spring 대응 요약:
- `Entity::find_by_id` ≈ `repository.findById(...)`
- `Entity::find().filter(...)` ≈ `repository.findByMemberIdAndTeamId(...)`
- `.all()` ≈ `findAll()`

---

## 5) SeaORM 기본 개념 (JPA와 비교)

### 5-1. 핵심 구조

- **Entity**: 테이블에 대응하는 타입 (JPA의 `@Entity`와 유사)
- **Model**: 조회 결과 레코드
- **ActiveModel**: INSERT/UPDATE용 변경 가능한 모델

**파일**: `src/domain/retrospect/entity/retrospect.rs`

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "retrospects")]
pub struct Model {
    pub retrospect_id: i64,
    pub title: String,
    pub start_time: DateTime,
    pub team_id: i64,
    ...
}
```

### 5-2. 쿼리 작성 패턴

```rust
retrospect::Entity::find()
    .filter(retrospect::Column::TeamId.eq(team_id))
    .order_by_desc(retrospect::Column::StartTime)
    .order_by_desc(retrospect::Column::RetrospectId)
    .all(&state.db)
    .await
```

**실제 SQL로 보면**

```sql
SELECT *
FROM retrospects
WHERE team_id = ?
ORDER BY start_time DESC, retrospect_id DESC;
```

---

## 6) Rust 문법 핵심 포인트 (이 API에 필요한 것만)

### 6-1. `async/await`

- Rust의 비동기 함수는 `async fn`으로 정의
- DB 호출은 `.await`로 기다림

### 6-2. `Result<T, E>`와 `?` 연산자

```rust
let team_exists = team::Entity::find_by_id(team_id)
    .one(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```

- `?`는 에러가 있으면 즉시 반환 (Spring에서 `throw`와 유사)
- `map_err`로 에러 타입 변환

### 6-3. `From`/`Into` (DTO 변환)

```rust
impl From<RetrospectModel> for TeamRetrospectListItem {
    fn from(model: RetrospectModel) -> Self { ... }
}
```

- `From` 구현 → `.into()` 호출 가능
- 엔티티 → DTO 변환 로직 캡슐화

### 6-4. 소유권과 `into_iter()`

```rust
retrospects.into_iter().map(|r| r.into()).collect();
```

- `into_iter()`는 **소유권을 옮기면서 순회** (Java Stream과 유사)
- `collect()`로 `Vec` 생성

---

## 7) DTO/응답 구조 이해

**파일**: `src/domain/retrospect/dto.rs`

```rust
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TeamRetrospectListItem {
    pub retrospect_id: i64,
    pub project_name: String,
    pub retrospect_method: RetrospectMethod,
    pub retrospect_date: String,
    pub retrospect_time: String,
}
```

- `serde(rename_all = "camelCase")` → JSON에서는 `retrospectId`로 변환됨
- `RetrospectMethod`는 Enum이며 DB에는 문자열 Enum으로 저장됨

**응답 래핑** (`BaseResponse`):

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "팀 내 전체 회고 목록 조회를 성공했습니다.",
  "result": [ ... ]
}
```

---

## 8) 에러 처리 흐름 (Spring ExceptionHandler 역할)

**파일**: `src/utils/error.rs`

- `AppError`는 에러 종류별로 **HTTP Status + 에러 코드**를 매핑
- `IntoResponse` 구현 덕분에 `Result<_, AppError>`만 반환하면 자동으로 JSON 에러 응답 생성

예시:
- `AppError::TeamNotFound` → HTTP 404, 코드 `TEAM4041`
- `AppError::TeamAccessDenied` → HTTP 403, 코드 `TEAM4031`

---

## 9) 이해 체크리스트

다음을 설명할 수 있으면 API-010을 이해한 것입니다.

- JWT 인증은 어디에서 수행되며 실패 시 어떤 응답이 나오는가?
- `{teamId}`는 어떤 방식으로 i64로 파싱되는가?
- 팀 존재 여부 검증은 왜 먼저 하는가?
- 팀 멤버가 아닐 때 왜 403이 반환되는가?
- SeaORM의 `.filter().order_by_desc().all()`은 어떤 SQL로 변환되는가?
- `RetrospectModel → TeamRetrospectListItem` 변환은 어디에서 일어나는가?
- `BaseResponse`가 왜 필요한가?

---

## 10) Spring 개발자를 위한 빠른 요약

- **Controller = Handler**, **Service = Service**, **Repository = SeaORM Entity**
- **AuthUser Extractor = Security Filter + Argument Resolver**
- `Result<T, AppError>`는 Spring의 `throw`와 유사하게 흐름을 끊고 에러 응답을 만든다
- DTO 변환은 `From` trait로 구현 (Java의 Mapper 클래스 역할)

---

필요하면 다음 문서들도 함께 보세요:
- `flow.md`: 흐름 요약
- `key-concepts.md`: Rust/Axum/SeaORM 핵심 개념
- `keywords.md`: 용어 정리
