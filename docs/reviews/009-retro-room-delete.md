# API-009 RetroRoom Delete Implementation Review

## 개요
- **API**: `DELETE /api/v1/retro-rooms/{retroRoomId}`
- **기능**: 레트로룸 삭제
- **담당자**: Claude Code
- **작성일**: 2026-01-26
- **상태**: 구현 완료

## API 스펙 요약

### Request
```
DELETE /api/v1/retro-rooms/{retroRoomId}
Authorization: Bearer {accessToken}
```

### Response (200 OK)
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공입니다.",
  "result": {
    "retroRoomId": 123,
    "deletedAt": "2026-01-26T22:45:05"
  }
}
```

## 구현 상세

### 1. DTO 설계 (`domain/retrospect/dto.rs`)

```rust
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeleteRetroRoomResponse {
    pub retro_room_id: i64,
    pub deleted_at: String,
}
```

### 2. 비즈니스 로직 (`domain/retrospect/service.rs`)

```rust
pub async fn delete_retro_room(
    state: AppState,
    member_id: i64,
    retro_room_id: i64,
) -> Result<DeleteRetroRoomResponse, AppError> {
    // 1. 룸 존재 확인
    // 2. Owner 권한 확인 (RoomRole::Owner)
    // 3. 관련 member_retro_room 데이터 먼저 삭제
    // 4. retro_room 삭제
}
```

**처리 순서**:
1. `retro_room_id`로 룸 존재 확인
2. `member_retro_room`에서 Owner 역할 확인
3. `member_retro_room` 테이블에서 해당 룸의 모든 멤버 관계 삭제
4. `retro_room` 테이블에서 룸 삭제

### 3. 핸들러 (`domain/retrospect/handler.rs`)

```rust
#[utoipa::path(
    delete,
    path = "/api/v1/retro-rooms/{retro_room_id}",
    params(
        ("retro_room_id" = i64, Path, description = "레트로룸 ID")
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "삭제 성공", body = SuccessDeleteRetroRoomResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "권한 없음", body = ErrorResponse),
        (status = 404, description = "룸 없음", body = ErrorResponse)
    ),
    tag = "RetroRoom"
)]
pub async fn delete_retro_room(...)
```

## 에러 처리

| 에러 코드 | HTTP 상태 | 설명 |
|-----------|-----------|------|
| `AUTH4001` | 401 | 인증 정보가 유효하지 않음 |
| `TEAM4031` | 403 | 삭제 권한 없음 (Owner가 아님) |
| `RETRO4041` | 404 | 존재하지 않는 레트로룸 |
| `COMMON500` | 500 | 서버 내부 에러 |

## 삭제 순서 (Cascade)

```
1. member_retro_room (해당 룸의 모든 멤버 관계)
2. retro_room (룸 자체)
```

> **주의**: 현재 구현에서는 `retrospects`, `responses`, `comments` 등의 연관 데이터는 별도 삭제 로직이 필요할 수 있습니다.

## 코드 리뷰 체크리스트

- [x] Owner 권한 검증이 작동하는가?
- [x] 관련 데이터가 올바른 순서로 삭제되는가?
- [x] 삭제 후 응답에 deletedAt이 포함되는가?
- [x] Swagger 문서가 생성되는가?
- [x] 코드가 Rust 컨벤션을 따르는가?
