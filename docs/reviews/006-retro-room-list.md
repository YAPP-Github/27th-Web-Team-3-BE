# API-006 RetroRoom List Implementation Review

## 개요
- **API**: `GET /api/v1/retro-rooms`
- **기능**: 참여 중인 레트로룸 목록 조회
- **담당자**: Claude Code
- **작성일**: 2026-01-26
- **상태**: 구현 완료

## API 스펙 요약

### Request
```
GET /api/v1/retro-rooms
Authorization: Bearer {accessToken}
```

### Response (200 OK)
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공입니다.",
  "result": [
    {
      "retroRoomId": 789,
      "retroRoomName": "프로젝트 A",
      "orderIndex": 1
    },
    {
      "retroRoomId": 456,
      "retroRoomName": "프로젝트 B",
      "orderIndex": 2
    }
  ]
}
```

## 구현 상세

### 1. DTO 설계 (`domain/retrospect/dto.rs`)

```rust
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetroRoomListItem {
    pub retro_room_id: i64,
    pub retro_room_name: String,
    pub order_index: i32,
}
```

### 2. 비즈니스 로직 (`domain/retrospect/service.rs`)

```rust
pub async fn list_retro_rooms(
    state: AppState,
    member_id: i64,
) -> Result<Vec<RetroRoomListItem>, AppError> {
    // 1. member_retro_room에서 사용자가 참여 중인 룸 목록 조회
    // 2. order_index 기준 오름차순 정렬
    // 3. retro_room 테이블과 조인하여 룸 이름 조회
}
```

**처리 순서**:
1. `member_retro_room` 테이블에서 member_id로 필터링
2. `order_index` 기준 오름차순 정렬
3. 각 룸의 상세 정보 조회 후 반환

### 3. 핸들러 (`domain/retrospect/handler.rs`)

```rust
#[utoipa::path(
    get,
    path = "/api/v1/retro-rooms",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "레트로룸 목록 조회 성공", body = SuccessRetroRoomListResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse)
    ),
    tag = "RetroRoom"
)]
pub async fn list_retro_rooms(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<BaseResponse<Vec<RetroRoomListItem>>>, AppError>
```

## 에러 처리

| 에러 코드 | HTTP 상태 | 설명 |
|-----------|-----------|------|
| `AUTH4001` | 401 | 인증 정보가 유효하지 않음 |
| `COMMON500` | 500 | 서버 내부 에러 |

## Entity 변경

### member_retro_room.rs
```rust
pub struct Model {
    pub member_retrospect_room_id: i64,
    pub member_id: i64,
    pub retrospect_room_id: i64,
    pub role: RoomRole,
    pub order_index: i32,  // 추가됨
}
```

## 코드 리뷰 체크리스트

- [x] API 명세에 맞게 구현되었는가?
- [x] 정렬 순서가 올바른가? (order_index ASC)
- [x] 빈 배열 응답이 처리되는가?
- [x] Swagger 문서가 생성되는가?
- [x] 코드가 Rust 컨벤션을 따르는가?
