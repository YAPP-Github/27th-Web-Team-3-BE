# [API-013] 동작 흐름

## 전체 흐름도

```
Client
  │  DELETE /api/v1/retrospects/{retrospectId}
  │  Authorization: Bearer {token}
  ▼
Handler (delete_retrospect)                    ← handler.rs:624-668
  │  1. retrospectId >= 1 검증
  │  2. AuthUser에서 user_id 추출
  │  3. RetrospectService::delete_retrospect() 호출
  ▼
Service (delete_retrospect)                    ← service.rs:1134-1282
  │
  ├─ [사전 검증] find_retrospect_for_member()  ← service.rs:323-352
  │    ├─ 회고 존재 여부 확인 (find_by_id)
  │    └─ 팀 멤버십 확인 (member_team 조회)
  │         → 미존재/비멤버 모두 404 반환 (보안 통합)
  │    ※ 스펙에는 Owner/Creator 권한 검증이 정의되어 있으나
  │      현재 미구현 (DB 스키마 미비). 팀 멤버면 삭제 가능.
  │
  └─ [트랜잭션] db.begin() ~ txn.commit()      ← service.rs:1157-1269
       │
       ├─ Step 1: response ID 목록 조회         ← service.rs:1164-1171
       │    └─ SELECT response_id FROM response WHERE retrospect_id = ?
       │
       ├─ Step 2: (response_ids 비어있지 않은 경우)
       │    ├─ 댓글 삭제 (response_comment)      ← service.rs:1175-1179
       │    ├─ 좋아요 삭제 (response_like)       ← service.rs:1182-1186
       │    └─ 멤버 응답 매핑 삭제 (member_response) ← service.rs:1189-1193
       │
       ├─ Step 3: 응답 삭제 (response)           ← service.rs:1206-1210
       ├─ Step 4: 참고자료 삭제 (retro_reference) ← service.rs:1213-1217
       ├─ Step 5: 멤버-회고 매핑 삭제 (member_retro) ← service.rs:1220-1224
       ├─ Step 6: 회고 삭제 (retrospect)          ← service.rs:1227-1230
       │
       └─ Step 7: 회고방 조건부 삭제              ← service.rs:1233-1264
            ├─ 같은 room을 참조하는 다른 회고 수 조회
            ├─ 0개인 경우:
            │    ├─ 멤버-회고방 매핑 삭제 (member_retro_room)
            │    └─ 회고방 삭제 (retro_room)
            └─ 1개 이상인 경우:
                 └─ 삭제 건너뜀 (warn 로그)
  │
  ▼
BaseResponse<()> with message "회고가 성공적으로 삭제되었습니다."
```

## 단계별 상세 설명

### 1단계: Handler - 입력 검증 및 인증

**소스**: `handler.rs:624-668`

```rust
/// 회고 삭제 API (API-013)
///
/// 특정 회고 세션과 연관된 모든 데이터(답변, 댓글, 좋아요, AI 분석 결과)를 영구 삭제합니다.
/// 해당 팀의 멤버만 삭제가 가능합니다.
#[utoipa::path(
    delete,
    path = "/api/v1/retrospects/{retrospectId}",
    params(("retrospectId" = i64, Path, description = "삭제할 회고의 고유 식별자")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "회고가 성공적으로 삭제되었습니다.", body = SuccessDeleteRetrospectResponse),
        (status = 400, description = "잘못된 Path Parameter", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 404, description = "존재하지 않는 회고이거나 접근 권한 없음 (보안상 404로 통합)", body = ErrorResponse),
        (status = 500, description = "서버 내부 오류", body = ErrorResponse)
    ),
    tag = "Retrospect"
)]
pub async fn delete_retrospect(
    user: AuthUser,
    State(state): State<AppState>,
    Path(retrospect_id): Path<i64>,
) -> Result<Json<BaseResponse<()>>, AppError> {
    if retrospect_id < 1 {
        return Err(AppError::BadRequest(
            "retrospectId는 1 이상의 양수여야 합니다.".to_string(),
        ));
    }
    let user_id = user.user_id()?;
    RetrospectService::delete_retrospect(state, user_id, retrospect_id).await?;
    Ok(Json(BaseResponse::success_with_message(
        (),
        "회고가 성공적으로 삭제되었습니다.",
    )))
}
```

- `Path(retrospect_id)`: Axum이 URL 경로에서 `{retrospectId}`를 `i64`로 자동 파싱
- `BaseResponse<()>`: 삭제 API이므로 result 필드에 `null`이 들어감 (unit type `()` 직렬화 결과)
- Handler는 비즈니스 로직 없이 검증 + 서비스 위임 + 응답 래핑만 담당
- **스펙과의 차이**: API 스펙에는 403 에러가 정의되어 있으나, Swagger 응답에 403이 없음 (구현 현실 반영). doc 주석도 `"해당 팀의 멤버만 삭제가 가능합니다."`로 현재 동작을 정확히 기술함

### 2단계: Service - 멤버십 검증

**소스**: `service.rs:1150-1152` -> 헬퍼: `service.rs:323-352`

```rust
let retrospect_model =
    Self::find_retrospect_for_member(&state, user_id, retrospect_id).await?;
```

`find_retrospect_for_member`는 두 단계 검증을 수행:
1. `retrospect::Entity::find_by_id(retrospect_id)` -- 회고 존재 여부
2. `member_team::Entity::find().filter(member_id, team_id)` -- 팀 멤버십

두 검증 모두 실패 시 동일한 `RetrospectNotFound` 에러를 반환하여 보안을 강화한다 (존재 여부 노출 방지).

### 3단계: Service - 트랜잭션 시작

**소스**: `service.rs:1157-1161`

```rust
let txn = state
    .db
    .begin()
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```

`TransactionTrait::begin()`으로 트랜잭션을 시작한다. 이후 모든 삭제 쿼리는 `&txn`을 통해 실행된다.

### 4단계: Service - response ID 조회

**소스**: `service.rs:1164-1171`

```rust
let response_ids: Vec<i64> = response::Entity::find()
    .filter(response::Column::RetrospectId.eq(retrospect_id))
    .select_only()
    .column(response::Column::ResponseId)
    .into_tuple()
    .all(&txn)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```

- `select_only().column()`: 전체 모델이 아닌 `response_id` 컬럼만 조회 (메모리 절약)
- `into_tuple()`: 단일 컬럼을 `i64` 튜플로 변환
- 이 ID 목록을 `is_in()` 필터에 사용하여 하위 테이블을 일괄 삭제

### 5단계: Service - Cascade 삭제 (FK 의존 역순)

**소스**: `service.rs:1173-1230`

삭제 순서는 FK(Foreign Key) 의존 관계의 역순으로 수행해야 한다:

| 순서 | 테이블 | FK 참조 대상 | 라인 |
|:----:|--------|-------------|------|
| 1 | `response_comment` | response_id -> response | L1175-1179 |
| 2 | `response_like` | response_id -> response | L1182-1186 |
| 3 | `member_response` | response_id -> response | L1189-1193 |
| 4 | `response` | retrospect_id -> retrospect | L1206-1210 |
| 5 | `retro_reference` | retrospect_id -> retrospect | L1213-1217 |
| 6 | `member_retro` | retrospect_id -> retrospect | L1220-1224 |
| 7 | `retrospect` | retrospect_room_id -> retro_room | L1227-1230 |
| 8 | `member_retro_room` | retrospect_room_id -> retro_room | L1241-1245 |
| 9 | `retro_room` | (루트 테이블) | L1247-1251 |

### 6단계: Service - 회고방 조건부 삭제

**소스**: `service.rs:1233-1264`

```rust
let other_retro_count = retrospect::Entity::find()
    .filter(retrospect::Column::RetrospectRoomId.eq(retrospect_room_id))
    .count(&txn)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```

같은 `retrospect_room_id`를 참조하는 다른 회고가 있으면 회고방을 삭제하지 않는다. 이미 Step 7에서 현재 회고를 삭제했으므로, count 결과가 0이면 해당 회고방을 참조하는 회고가 더 이상 없음을 의미한다.

### 7단계: Service - 트랜잭션 커밋

**소스**: `service.rs:1267-1269`

```rust
txn.commit()
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```

모든 삭제가 성공하면 커밋한다. 중간에 에러가 발생하면 `txn`이 drop되면서 자동 rollback된다 (RAII 패턴).

## 에러 발생 시나리오

| 단계 | 에러 조건 | AppError | HTTP | 메시지 |
|------|----------|----------|------|--------|
| Handler | retrospectId < 1 | `BadRequest` | 400 | `"잘못된 요청입니다: retrospectId는 1 이상의 양수여야 합니다."` (스펙에는 prefix 없이 기술됨) |
| Handler | JWT 파싱 실패 | `Unauthorized` | 401 | (인증 미들웨어에서 처리) |
| Service | 회고 미존재 | `RetrospectNotFound` | 404 | `"존재하지 않는 회고이거나 접근 권한이 없습니다."` |
| Service | 팀 비멤버 | `RetrospectNotFound` | 404 | `"존재하지 않는 회고이거나 접근 권한이 없습니다."` |
| Service | DB 트랜잭션 실패 | `InternalError` | 500 | `"서버 에러, 관리자에게 문의 바랍니다."` (스펙에는 `"데이터 삭제 중 서버 에러가 발생했습니다."`) |

### 스펙 대비 미구현 에러

아래 에러는 API 스펙에 정의되어 있으나 현재 구현에서는 발생하지 않습니다.

| 스펙 에러 | AppError (정의됨) | HTTP | 미구현 사유 |
|----------|-------------------|------|------------|
| 삭제 권한 없음 (Owner/Creator가 아닌 경우) | `RetroDeleteAccessDenied` (`#[allow(dead_code)]`) | 403 | DB 스키마에 `created_by`, `member_team.role` 필드 미존재. 현재는 팀 멤버라면 누구나 삭제 가능 |

또한 스펙의 404 메시지는 `"존재하지 않는 회고입니다."`이지만, 구현에서는 보안 패턴에 따라 `"존재하지 않는 회고이거나 접근 권한이 없습니다."`로 통합되어 있습니다.
