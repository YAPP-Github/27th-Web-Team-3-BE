# 동작 흐름: 팀 회고 목록 조회 (API-010)

## 전체 흐름 요약

```
클라이언트 요청
  -> [Axum] AuthUser extractor (JWT 인증)
  -> [Handler] list_team_retrospects (입력 검증)
  -> [Service] list_team_retrospects (비즈니스 로직)
    -> DB: 팀 존재 확인
    -> DB: 팀 멤버십 확인
    -> DB: 회고 목록 조회 (최신순)
    -> DTO 변환 (From trait)
  -> [Handler] BaseResponse 래핑 후 JSON 응답
```

## 단계별 상세

### 1단계: JWT 인증 (AuthUser Extractor)

**파일**: `src/utils/auth.rs:23-55`

Axum의 `FromRequestParts` trait을 구현한 `AuthUser`가 요청에서 자동으로 JWT를 추출 및 검증합니다.

```rust
pub struct AuthUser(pub Claims);

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        // Authorization 헤더에서 "Bearer {token}" 추출
        // decode_access_token으로 JWT 검증
    }
}
```

- `Authorization` 헤더가 없으면 `AppError::Unauthorized` 반환
- `Bearer ` 접두사가 없으면 `AppError::Unauthorized` 반환
- 토큰이 유효하지 않으면 `AppError::Unauthorized` 반환

### 2단계: 핸들러 - 입력 검증 및 서비스 호출

**파일**: `src/domain/retrospect/handler.rs:88-110`

```rust
pub async fn list_team_retrospects(
    user: AuthUser,                    // 1단계에서 자동 추출
    State(state): State<AppState>,     // 앱 상태 (DB 커넥션 등)
    Path(team_id): Path<i64>,          // URL 경로에서 teamId 추출
) -> Result<Json<BaseResponse<Vec<TeamRetrospectListItem>>>, AppError> {
    // teamId 검증: 1 이상의 양수
    if team_id < 1 {
        return Err(AppError::BadRequest("팀 ID는 1 이상이어야 합니다.".to_string()));
    }

    // JWT에서 사용자 ID 추출
    let user_id = user.user_id()?;

    // 서비스 호출
    let result = RetrospectService::list_team_retrospects(state, user_id, team_id).await?;

    Ok(Json(BaseResponse::success_with_message(result, "팀 내 전체 회고 목록 조회를 성공했습니다.")))
}
```

핸들러의 역할:
- `Path(team_id)`: URL의 `{teamId}`를 `i64`로 파싱
- `team_id < 1` 검증으로 유효하지 않은 ID 조기 차단
- `user.user_id()?`: Claims의 `sub` 필드를 `i64`로 파싱
- 비즈니스 로직은 Service에 위임

### 3단계: 서비스 - 팀 존재 여부 확인

**파일**: `src/domain/retrospect/service.rs:278-288`

```rust
let team_exists = team::Entity::find_by_id(team_id)
    .one(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

if team_exists.is_none() {
    return Err(AppError::TeamNotFound("존재하지 않는 팀입니다.".to_string()));
}
```

- SeaORM의 `find_by_id`로 PK 기반 단건 조회
- `.one()`으로 `Option<Model>` 반환
- `None`이면 404 에러 (`TEAM4041`)

### 4단계: 서비스 - 팀 멤버십 확인

**파일**: `src/domain/retrospect/service.rs:291-302`

```rust
let is_member = member_team::Entity::find()
    .filter(member_team::Column::MemberId.eq(user_id))
    .filter(member_team::Column::TeamId.eq(team_id))
    .one(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

if is_member.is_none() {
    return Err(AppError::TeamAccessDenied("해당 팀에 접근 권한이 없습니다.".to_string()));
}
```

- `member_team` 테이블에서 `(user_id, team_id)` 조합으로 멤버 존재 확인
- `.filter()` 체이닝으로 WHERE 조건 추가
- 멤버가 아니면 403 에러 (`TEAM4031`)

### 5단계: 서비스 - 회고 목록 조회

**파일**: `src/domain/retrospect/service.rs:304-311`

```rust
let retrospects = retrospect::Entity::find()
    .filter(retrospect::Column::TeamId.eq(team_id))
    .order_by_desc(retrospect::Column::StartTime)
    .order_by_desc(retrospect::Column::RetrospectId)
    .all(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```

- `retrospect::Column::TeamId.eq(team_id)`: 해당 팀의 회고만 필터링
- `.order_by_desc(StartTime)`: 최신순 정렬 (1차 정렬 기준)
- `.order_by_desc(RetrospectId)`: 동일 시간일 경우 ID 역순으로 안정 정렬 (2차 정렬 기준)
- `.all()`: 전체 목록을 `Vec<Model>`로 반환

### 6단계: 서비스 - DTO 변환

**파일**: `src/domain/retrospect/service.rs:314-315`

```rust
let result: Vec<TeamRetrospectListItem> =
    retrospects.into_iter().map(|r| r.into()).collect();
```

- `into_iter()`로 소유권 이전 이터레이터 생성
- `.map(|r| r.into())`: `From<RetrospectModel>` trait 구현을 통해 엔티티 -> DTO 변환
- `.collect()`: 이터레이터를 `Vec`으로 수집

### 7단계: 응답 래핑

**파일**: `src/domain/retrospect/handler.rs:106-109`

```rust
Ok(Json(BaseResponse::success_with_message(
    result,
    "팀 내 전체 회고 목록 조회를 성공했습니다.",
)))
```

- `BaseResponse::success_with_message`로 표준 응답 형식 래핑
- `Json()`으로 직렬화하여 HTTP 응답 생성

## 에러 흐름

| 단계 | 조건 | 에러 코드 | HTTP 상태 |
|------|------|-----------|-----------|
| 1 (인증) | 토큰 없음/만료/잘못된 형식 | AUTH4001 | 401 |
| 2 (핸들러) | team_id < 1 | COMMON400 | 400 |
| 3 (서비스) | 팀이 DB에 없음 | TEAM4041 | 404 |
| 4 (서비스) | 사용자가 팀 멤버가 아님 | TEAM4031 | 403 |
| 3~5 (서비스) | DB 쿼리 실패 | COMMON500 | 500 |
