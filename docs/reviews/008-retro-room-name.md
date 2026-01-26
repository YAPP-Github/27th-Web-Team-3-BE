# API-008 RetroRoom Name Update Implementation Review

## 개요
- **API**: `PATCH /api/v1/retro-rooms/{retroRoomId}/name`
- **기능**: 레트로룸 이름 변경
- **담당자**: Claude Code
- **작성일**: 2026-01-26
- **상태**: 구현 완료

## API 스펙 요약

### Request
```json
PATCH /api/v1/retro-rooms/{retroRoomId}/name
Authorization: Bearer {accessToken}
Content-Type: application/json

{
  "name": "새로운 레트로룸 이름"
}
```

### Response (200 OK)
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공입니다.",
  "result": {
    "retroRoomId": 123,
    "retroRoomName": "새로운 레트로룸 이름",
    "updatedAt": "2026-01-26T15:30:00"
  }
}
```

## 구현 상세

### 1. DTO 설계 (`domain/retrospect/dto.rs`)

**Request**:
```rust
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRetroRoomNameRequest {
    #[validate(length(min = 1, max = 20, message = "레트로룸 이름은 1~20자여야 합니다."))]
    pub name: String,
}
```

**Response**:
```rust
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRetroRoomNameResponse {
    pub retro_room_id: i64,
    pub retro_room_name: String,
    pub updated_at: String,
}
```

### 2. 비즈니스 로직 (`domain/retrospect/service.rs`)

```rust
pub async fn update_retro_room_name(
    state: AppState,
    member_id: i64,
    retro_room_id: i64,
    req: UpdateRetroRoomNameRequest,
) -> Result<UpdateRetroRoomNameResponse, AppError> {
    // 1. 룸 존재 확인
    // 2. Owner 권한 확인 (RoomRole::Owner)
    // 3. 이름 중복 체크 (다른 룸과 중복 불가)
    // 4. 이름 및 updated_at 업데이트
}
```

**처리 순서**:
1. `retro_room_id`로 룸 존재 확인
2. `member_retro_room`에서 Owner 역할 확인
3. 동일 이름의 다른 룸이 있는지 확인
4. `retro_room.title` 및 `updated_at` 업데이트

### 3. 핸들러 (`domain/retrospect/handler.rs`)

```rust
#[utoipa::path(
    patch,
    path = "/api/v1/retro-rooms/{retro_room_id}/name",
    request_body = UpdateRetroRoomNameRequest,
    params(
        ("retro_room_id" = i64, Path, description = "레트로룸 ID")
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "이름 변경 성공", body = SuccessUpdateRetroRoomNameResponse),
        (status = 400, description = "이름 길이 초과", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "권한 없음", body = ErrorResponse),
        (status = 404, description = "룸 없음", body = ErrorResponse),
        (status = 409, description = "이름 중복", body = ErrorResponse)
    ),
    tag = "RetroRoom"
)]
pub async fn update_retro_room_name(...)
```

## 에러 처리

| 에러 코드 | HTTP 상태 | 설명 |
|-----------|-----------|------|
| `COMMON400` | 400 | 이름 길이 유효성 검사 실패 (1~20자) |
| `AUTH4001` | 401 | 인증 정보가 유효하지 않음 |
| `ROOM4031` | 403 | 이름 변경 권한 없음 (Owner가 아님) |
| `RETRO4041` | 404 | 존재하지 않는 레트로룸 |
| `RETRO4091` | 409 | 이미 사용 중인 이름 |
| `COMMON500` | 500 | 서버 내부 에러 |

## 추가된 에러 타입

### error.rs
```rust
/// ROOM4031: 권한 없음 - 이름 변경 (403)
NoRoomPermission(String),
```

## 코드 리뷰 체크리스트

- [x] Owner 권한 검증이 작동하는가?
- [x] 이름 길이 검증이 올바른가? (1~20자)
- [x] 이름 중복 검증이 작동하는가?
- [x] updated_at이 업데이트되는가?
- [x] Swagger 문서가 생성되는가?
- [x] 코드가 Rust 컨벤션을 따르는가?
