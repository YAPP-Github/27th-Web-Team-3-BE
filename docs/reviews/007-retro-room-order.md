# API-007 RetroRoom Order Update Implementation Review

## 개요
- **API**: `PATCH /api/v1/retro-rooms/order`
- **기능**: 레트로룸 순서 변경 (드래그 앤 드롭)
- **담당자**: Claude Code
- **작성일**: 2026-01-26
- **상태**: 구현 완료

## API 스펙 요약

### Request
```json
PATCH /api/v1/retro-rooms/order
Authorization: Bearer {accessToken}
Content-Type: application/json

{
  "retroRoomOrders": [
    { "retroRoomId": 456, "orderIndex": 1 },
    { "retroRoomId": 789, "orderIndex": 2 }
  ]
}
```

### Response (200 OK)
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공입니다.",
  "result": null
}
```

## 구현 상세

### 1. DTO 설계 (`domain/retrospect/dto.rs`)

```rust
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetroRoomOrderItem {
    pub retro_room_id: i64,
    #[validate(range(min = 1, message = "orderIndex는 1 이상이어야 합니다."))]
    pub order_index: i32,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRetroRoomOrderRequest {
    #[validate(length(min = 1, message = "최소 1개 이상의 순서 정보가 필요합니다."))]
    #[validate(nested)]
    pub retro_room_orders: Vec<RetroRoomOrderItem>,
}
```

### 2. 비즈니스 로직 (`domain/retrospect/service.rs`)

```rust
pub async fn update_retro_room_order(
    state: AppState,
    member_id: i64,
    req: UpdateRetroRoomOrderRequest,
) -> Result<(), AppError> {
    // 1. orderIndex 중복 체크 (HashSet 사용)
    // 2. 각 룸에 대해:
    //    - 사용자가 해당 룸의 멤버인지 확인
    //    - member_retro_room의 order_index 업데이트
}
```

**유효성 검증**:
- `orderIndex` 중복 불가 (HashSet으로 검증)
- 각 룸에 대해 멤버 권한 확인

### 3. 핸들러 (`domain/retrospect/handler.rs`)

```rust
#[utoipa::path(
    patch,
    path = "/api/v1/retro-rooms/order",
    request_body = UpdateRetroRoomOrderRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "순서 변경 성공", body = SuccessEmptyResponse),
        (status = 400, description = "잘못된 순서 데이터", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "권한 없음", body = ErrorResponse)
    ),
    tag = "RetroRoom"
)]
pub async fn update_retro_room_order(...)
```

## 에러 처리

| 에러 코드 | HTTP 상태 | 설명 |
|-----------|-----------|------|
| `TEAM4004` | 400 | 잘못된 순서 데이터 (중복된 orderIndex) |
| `AUTH4001` | 401 | 인증 정보가 유효하지 않음 |
| `TEAM4031` | 403 | 순서를 변경할 권한 없음 (멤버가 아님) |
| `COMMON500` | 500 | 서버 내부 에러 |

## 추가된 에러 타입

### error.rs
```rust
/// TEAM4004: 잘못된 순서 데이터 (400)
InvalidOrderData(String),

/// TEAM4031: 권한 없음 - 순서/삭제 (403)
NoPermission(String),
```

## 코드 리뷰 체크리스트

- [x] orderIndex 중복 검증이 작동하는가?
- [x] 멤버 권한 검증이 올바른가?
- [x] 부분 업데이트가 가능한가? (변경된 룸만 전송)
- [x] Swagger 문서가 생성되는가?
- [x] 코드가 Rust 컨벤션을 따르는가?
