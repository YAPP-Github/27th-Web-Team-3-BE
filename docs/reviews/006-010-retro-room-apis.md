# API 006-010 RetroRoom APIs Implementation Review

## 개요
- **APIs**: 006 ~ 010 (레트로룸 관련 CRUD APIs)
- **담당자**: Claude Code
- **작성일**: 2026-01-26
- **상태**: 구현 완료

## 구현된 API 목록

| API | Method | Endpoint | 설명 |
|-----|--------|----------|------|
| 006 | GET | `/api/v1/retro-rooms` | 참여 중인 레트로룸 목록 조회 |
| 007 | PATCH | `/api/v1/retro-rooms/order` | 레트로룸 순서 변경 |
| 008 | PATCH | `/api/v1/retro-rooms/:id/name` | 레트로룸 이름 변경 |
| 009 | DELETE | `/api/v1/retro-rooms/:id` | 레트로룸 삭제 |
| 010 | GET | `/api/v1/retro-rooms/:id/retrospects` | 레트로룸 내 회고 목록 조회 |

---

## API-006: 레트로룸 목록 조회

### Request
```
GET /api/v1/retro-rooms
Authorization: Bearer {accessToken}
```

### Response
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
    }
  ]
}
```

### 구현 상세
- `member_retro_room` 테이블에서 사용자 참여 룸 조회
- `order_index` 기준 오름차순 정렬

---

## API-007: 레트로룸 순서 변경

### Request
```json
PATCH /api/v1/retro-rooms/order
{
  "retroRoomOrders": [
    { "retroRoomId": 456, "orderIndex": 1 },
    { "retroRoomId": 789, "orderIndex": 2 }
  ]
}
```

### Response
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공입니다.",
  "result": null
}
```

### 구현 상세
- orderIndex 중복 체크 (HashSet 사용)
- 각 룸에 대해 멤버 권한 확인 후 업데이트

---

## API-008: 레트로룸 이름 변경

### Request
```json
PATCH /api/v1/retro-rooms/{retroRoomId}/name
{
  "name": "새로운 이름"
}
```

### Response
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공입니다.",
  "result": {
    "retroRoomId": 123,
    "retroRoomName": "새로운 이름",
    "updatedAt": "2026-01-26T15:30:00"
  }
}
```

### 구현 상세
- 룸 존재 확인
- Owner 권한 확인 (RoomRole::Owner)
- 이름 중복 체크
- 이름 및 updated_at 업데이트

---

## API-009: 레트로룸 삭제

### Request
```
DELETE /api/v1/retro-rooms/{retroRoomId}
Authorization: Bearer {accessToken}
```

### Response
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

### 구현 상세
- 룸 존재 확인
- Owner 권한 확인
- 관련 member_retro_room 데이터 먼저 삭제
- retro_room 삭제

---

## API-010: 레트로룸 내 회고 목록 조회

### Request
```
GET /api/v1/retro-rooms/{retroRoomId}/retrospects
Authorization: Bearer {accessToken}
```

### Response
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공입니다.",
  "result": [
    {
      "retrospectId": 100,
      "projectName": "프로젝트 회고",
      "retrospectMethod": "KPT",
      "retrospectDate": "2026-01-20",
      "retrospectTime": "10:00"
    }
  ]
}
```

### 구현 상세
- 룸 존재 확인
- 멤버 권한 확인
- start_time 기준 최신순 정렬

---

## Entity 변경 사항

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

---

## 에러 처리

### 추가된 에러 코드

| 에러 코드 | HTTP 상태 | 설명 |
|-----------|-----------|------|
| `TEAM4004` | 400 | 잘못된 순서 데이터 |
| `TEAM4031` | 403 | 권한 없음 (순서/삭제) |
| `ROOM4031` | 403 | 권한 없음 (이름 변경) |

---

## DTO 목록

### Request DTOs
- `UpdateRetroRoomOrderRequest`
- `RetroRoomOrderItem`
- `UpdateRetroRoomNameRequest`

### Response DTOs
- `RetroRoomListItem`
- `UpdateRetroRoomNameResponse`
- `DeleteRetroRoomResponse`
- `RetrospectListItem`
- 각 Success Response DTOs (Swagger용)

---

## 품질 검사 결과

```bash
$ cargo fmt    # 통과
$ cargo clippy -- -D warnings  # 통과
$ cargo test   # 8 passed
```

---

## 코드 리뷰 체크리스트

- [x] API 명세에 맞게 구현되었는가?
- [x] 모든 에러 케이스가 처리되었는가?
- [x] 권한 검증이 올바르게 이루어지는가?
- [x] Swagger 문서가 생성되는가?
- [x] 코드가 Rust 컨벤션을 따르는가?
- [x] clippy 경고가 없는가?

---

## 변경 파일 목록

| 파일 | 변경 유형 | 설명 |
|------|-----------|------|
| `member_retro_room.rs` | 수정 | order_index 필드 추가 |
| `error.rs` | 수정 | 새 에러 타입 추가 |
| `dto.rs` | 수정 | API 006-010 DTOs 추가 |
| `service.rs` | 수정 | 5개 API 서비스 메서드 추가 |
| `handler.rs` | 수정 | 5개 API 핸들러 추가 |
| `main.rs` | 수정 | 라우트 및 OpenAPI 스키마 추가 |
