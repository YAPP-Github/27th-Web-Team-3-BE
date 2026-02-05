# [API-030] 회고방 멤버 목록 조회 API 리뷰

## 개요

회고방에 참여한 모든 멤버 목록을 조회하는 API입니다.

## 구현 내용

### 엔드포인트

```
GET /api/v1/retro-rooms/{retro_room_id}/members
```

### 구현 파일

| 파일 | 역할 |
|------|------|
| `src/domain/retrospect/handler.rs` | API 핸들러 (`list_retro_room_members`) |
| `src/domain/retrospect/service.rs` | 비즈니스 로직 (`list_retro_room_members`) |
| `src/domain/retrospect/dto.rs` | DTO 정의 (`RetroRoomMemberItem`) |
| `src/main.rs` | 라우트 등록 |

### 핵심 로직

1. **인증 확인**: JWT 토큰에서 사용자 ID 추출
2. **회고방 존재 확인**: `retro_room_id`로 회고방 조회
3. **권한 확인**: 요청자가 해당 회고방의 멤버인지 확인
4. **멤버 목록 조회**:
   - OWNER가 먼저, 그 다음 MEMBER 순으로 정렬
   - 동일 역할 내에서는 가입일시 기준 오름차순 정렬

### 응답 필드

| Field | Type | Description |
|-------|------|-------------|
| memberId | long | 멤버 고유 식별자 |
| nickname | string | 멤버 닉네임 |
| role | string | 역할 (OWNER/MEMBER) |
| joinedAt | string | 가입 일시 |

## 체크리스트

- [x] TDD 원칙 준수 (테스트 먼저 작성)
- [x] 모든 테스트 통과
- [x] API 스펙 문서 작성 (`docs/api-specs/030-retro-room-members-list.md`)
- [x] 공통 유틸리티 재사용 (`BaseResponse`, `AppError`)
- [x] 적절한 에러 처리 (404, 403, 401)
- [x] Rust 컨벤션 준수 (`cargo fmt`, `cargo clippy`)
- [x] camelCase 직렬화 (`#[serde(rename_all = "camelCase")]`)

## 에러 처리

| 상황 | 에러 코드 | HTTP 상태 |
|------|----------|----------|
| 회고방 없음 | RETRO4041 | 404 |
| 권한 없음 | RETRO4031 | 403 |
| 인증 실패 | AUTH4001 | 401 |

## 관련 변경사항

- **API-011 수정**: `RetrospectListItem`에 `participantCount` 필드 추가
  - 회고 목록 조회 시 각 회고의 참여자 수를 표시
  - 기존 테스트 업데이트 완료
