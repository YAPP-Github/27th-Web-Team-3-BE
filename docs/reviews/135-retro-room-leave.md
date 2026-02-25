# Issue #135: 팀 탈퇴 API 구현 및 팀 삭제 API 제거

## 변경 사항

### 1. RoomRole 완전 제거
- `member_retro_room.rs`: `RoomRole` enum 및 `role` 필드 삭제
- `create_retro_room`: `role: Set(RoomRole::Owner)` 제거
- `join_retro_room`: `role: Set(RoomRole::Member)` 제거
- `list_retro_room_members`: role 기반 정렬 -> 본인(요청자) 맨 위 + `created_at` 오름차순
- `RetroRoomMemberItem` DTO에서 `role` 필드 삭제
- `update_retro_room_name`: Owner 권한 체크 제거 (멤버면 변경 가능)

### 2. 팀 삭제 API 제거
- `DELETE /api/v1/retro-rooms/{retro_room_id}` 라우트 제거
- `delete_retro_room` 핸들러/DTO 삭제
- 회고방 전체 삭제 로직은 `delete_retro_room_internal` 내부 함수로 유지

### 3. 팀 탈퇴 API 추가
- `POST /api/v1/retro-rooms/{retro_room_id}/leave`
- 진행 중인 회고 차단 (`RETRO4009`)
- 마지막 멤버 탈퇴 시 회고방 자동 삭제

### 4. 에러 코드 추가
- `RetroRoomLeaveBlocked` (RETRO4009, 400)

## 테스트

- `api_009_retro_room_delete_test.rs` -> Leave 응답 직렬화 테스트로 변환
- `api_030_retro_room_members_test.rs` -> role 필드 제거 반영

## API 스펙

### POST /api/v1/retro-rooms/{retro_room_id}/leave

**성공 응답 (200)**
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "회고방 탈퇴에 성공하였습니다.",
  "result": {
    "retroRoomId": 1,
    "leftAt": "2026-02-25T12:00:00"
  }
}
```

**에러 응답**

| HTTP | 코드 | 설명 |
|------|------|------|
| 400 | RETRO4009 | 진행 중인 회고가 있어 탈퇴 불가 |
| 401 | AUTH4001 | 인증 실패 |
| 403 | RETRO4031 | 해당 회고방의 멤버가 아님 |
| 404 | RETRO4041 | 존재하지 않는 회고방 |
