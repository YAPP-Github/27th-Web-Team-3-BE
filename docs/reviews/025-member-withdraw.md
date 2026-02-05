# [API-028] 서비스 탈퇴 API 구현 리뷰

## 개요

| 항목 | 내용 |
|------|------|
| API | `DELETE /api/members/me` |
| 기능 | 현재 로그인한 사용자의 계정 삭제 및 서비스 탈퇴 |
| 구현 일자 | 2025-01-30 |

## 구현 파일

| 파일 | 역할 |
|------|------|
| `src/domain/member/handler.rs` | HTTP 핸들러 |
| `src/domain/member/service.rs` | 비즈니스 로직 |
| `src/domain/member/dto.rs` | Request/Response DTO |
| `src/domain/member/mod.rs` | 모듈 등록 |
| `src/utils/error.rs` | MemberNotFound 에러 추가 |
| `src/main.rs` | 라우트 및 Swagger 등록 |
| `tests/member_withdraw_test.rs` | 통합 테스트 |

## API 스펙

### Request

- **Method**: `DELETE`
- **Path**: `/api/members/me`
- **Headers**: `Authorization: Bearer {accessToken}`
- **Body**: 없음

### Response

**성공 (200 OK)**
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "회원 탈퇴가 성공적으로 완료되었습니다.",
  "result": null
}
```

## 에러 코드

| Code | HTTP Status | Description |
|------|-------------|-------------|
| AUTH4001 | 401 | 인증 정보가 유효하지 않습니다 |
| MEMBER4042 | 404 | 존재하지 않는 사용자입니다 |
| COMMON500 | 500 | 서버 내부 오류입니다 |

## 삭제 순서

회원 탈퇴 시 다음 순서로 데이터가 삭제됩니다:

1. `refresh_token` - 리프레시 토큰
2. `member_response` - 회고 답변 관계
3. `member_retro` - 회고 참여 관계
4. `member_retro_room` - 회고방 멤버십
5. `member` - 회원 정보

## 테스트 케이스

| 테스트 | 설명 | 상태 |
|--------|------|------|
| `should_withdraw_successfully` | 정상 탈퇴 | ✅ |
| `should_return_401_when_token_missing` | 토큰 누락 시 401 | ✅ |
| `should_return_401_for_invalid_bearer_format` | 잘못된 Bearer 형식 | ✅ |
| `should_return_404_when_member_not_found` | 사용자 없음 404 | ✅ |

## 체크리스트

- [x] TDD 원칙 준수 (테스트 먼저 작성)
- [x] 모든 테스트 통과
- [x] API 문서 작성
- [x] 에러 처리 적절
- [x] Rust 컨벤션 준수 (cargo fmt, clippy)
- [x] Swagger 문서 추가

## 주의사항

- 탈퇴된 계정은 **복구할 수 없습니다**
- 탈퇴 시 모든 관련 데이터가 즉시 삭제됩니다
- 클라이언트에서 탈퇴 전 확인 다이얼로그 표시를 권장합니다
