# 동작 흐름: 회고 참고자료 목록 조회 (API-018)

## 전체 흐름 요약

```
클라이언트 요청
  → 핸들러 (입력 검증 + 사용자 ID 추출)
    → 서비스 (회고 조회 + 멤버십 확인 + 참고자료 조회 + DTO 변환)
      → 응답 반환
```

## 단계별 상세 흐름

### 1단계: 핸들러 - 입력 검증 및 사용자 인증

**소스**: `codes/server/src/domain/retrospect/handler.rs:181~203`

```rust
pub async fn list_references(
    user: AuthUser,
    State(state): State<AppState>,
    Path(retrospect_id): Path<i64>,
) -> Result<Json<BaseResponse<Vec<ReferenceItem>>>, AppError> {
```

핸들러에서 수행하는 작업:

1. **retrospectId 경로 파라미터 검증** (라인 187~191)
   - `retrospect_id < 1`이면 `AppError::BadRequest("retrospectId는 1 이상의 양수여야 합니다.")` 반환
   - 에러 코드: `COMMON400` (400 Bad Request)
   - 0 이하의 값은 유효하지 않은 ID로 간주

2. **JWT에서 사용자 ID 추출** (라인 194)
   - `AuthUser` 미들웨어가 토큰을 파싱하고, `user.user_id()?`로 사용자 ID 획득
   - 토큰이 유효하지 않으면 `AppError::Unauthorized` 반환 (에러 코드: `AUTH4001`, 401)

3. **서비스 호출** (라인 197)
   - `RetrospectService::list_references(state, user_id, retrospect_id)` 호출

4. **성공 응답 래핑** (라인 199~202)
   - `BaseResponse::success_with_message`로 표준 응답 형식 적용

---

### 2단계: 서비스 - 회고 조회 및 팀 멤버십 확인

**소스**: `codes/server/src/domain/retrospect/service.rs:430~458`

```rust
pub async fn list_references(
    state: AppState,
    user_id: i64,
    retrospect_id: i64,
) -> Result<Vec<ReferenceItem>, AppError> {
```

#### 2-1. 회고 조회 + 멤버십 확인 (라인 436~437)

```rust
let _retrospect_model =
    Self::find_retrospect_for_member(&state, user_id, retrospect_id).await?;
```

**헬퍼 함수**: `find_retrospect_for_member` (라인 323~352)

이 헬퍼 함수는 두 가지를 동시에 검증합니다:

1. **회고 존재 여부 확인** (라인 328~336)
   - `retrospect::Entity::find_by_id(retrospect_id)`로 DB 조회
   - 존재하지 않으면 `AppError::RetrospectNotFound` 반환

2. **팀 멤버십 확인** (라인 338~349)
   - `member_team::Entity`에서 `user_id`와 `team_id` 조합으로 조회
   - 해당 팀의 멤버가 아니면 동일한 `AppError::RetrospectNotFound` 반환
   - 보안상 "존재하지 않음"과 "접근 권한 없음"을 구분하지 않고 404로 통합 처리

---

#### 2-2. 참고자료 목록 조회 (라인 440~445)

```rust
let references = retro_reference::Entity::find()
    .filter(retro_reference::Column::RetrospectId.eq(retrospect_id))
    .order_by_asc(retro_reference::Column::RetroReferenceId)
    .all(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```

- `retro_reference` 테이블에서 `retrospect_id`로 필터링
- `retro_reference_id` 오름차순 정렬 (등록 순서)
- DB 오류 발생 시 `AppError::InternalError`로 변환

---

#### 2-3. DTO 변환 (라인 448~455)

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

엔티티 모델의 필드를 DTO 필드로 매핑합니다:

| 엔티티 필드 (`retro_reference::Model`) | DTO 필드 (`ReferenceItem`) |
|----------------------------------------|----------------------------|
| `retro_reference_id` | `reference_id` |
| `title` | `url_name` |
| `url` | `url` |

참고: `title` 필드는 회고 생성 시 URL 값이 그대로 저장됩니다 (`service.rs:162`).

---

### 3단계: 응답 반환

최종 JSON 응답 예시:

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "참고자료 목록을 성공적으로 조회했습니다.",
  "result": [
    {
      "referenceId": 1,
      "urlName": "https://github.com/example/repo",
      "url": "https://github.com/example/repo"
    }
  ]
}
```

## 에러 발생 흐름

| 단계 | 조건 | 에러 | 에러 코드 | HTTP 코드 |
|------|------|------|-----------|-----------|
| 핸들러 | `retrospect_id < 1` | `AppError::BadRequest` | COMMON400 | 400 |
| 핸들러 | JWT 토큰 유효하지 않음 | `AppError::Unauthorized` | AUTH4001 | 401 |
| 서비스 | 회고가 DB에 없음 | `AppError::RetrospectNotFound` | RETRO4041 | 404 |
| 서비스 | 사용자가 팀 멤버 아님 | `AppError::RetrospectNotFound` | RETRO4041 | 404 |
| 서비스 | DB 쿼리 실패 | `AppError::InternalError` | COMMON500 | 500 |

**참고 -- 스펙과의 차이**: API 명세서(`docs/api-specs/018-retrospect-references-list.md`)에는 `403 Forbidden (TEAM4031)` 에러가 정의되어 있지만, 실제 구현에서는 팀 멤버가 아닌 경우에도 `404 (RETRO4041)`를 반환합니다. 이는 `find_retrospect_for_member` 헬퍼에서 보안을 위해 회고 존재 여부와 접근 권한 여부를 의도적으로 통합 처리하기 때문입니다.

## 시퀀스 다이어그램

```
Client          Handler              Service              DB
  |                |                    |                   |
  |-- GET req ---->|                    |                   |
  |                |-- validate id ---->|                   |
  |                |                    |-- find retro ---->|
  |                |                    |<-- retro model ---|
  |                |                    |-- check member -->|
  |                |                    |<-- membership ----|
  |                |                    |-- find refs ----->|
  |                |                    |<-- ref models ----|
  |                |                    |-- map to DTO      |
  |                |<-- Vec<RefItem> ---|                   |
  |<-- 200 JSON ---|                    |                   |
```
