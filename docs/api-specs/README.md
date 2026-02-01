# API 명세서 목록

> 회고록 작성 AI 서비스 백엔드 API 명세서

## 버전 정보

| 버전 | 날짜 | 설명 |
|------|------|------|
| 1.0.0 | 2025-01-25 | API 명세서 최초 작성 |

---

## API 목록

### 인증 (Auth) - 001~004

| API ID | Method | Endpoint | 설명 | 문서 |
|--------|--------|----------|------|------|
| API-001 | POST | `/api/v1/auth/social-login` | 소셜 로그인 (구글/카카오) | [001-auth-social-login.md](./001-auth-social-login.md) |
| API-002 | POST | `/api/v1/auth/signup` | 회원가입 (닉네임 등록) | [002-auth-signup.md](./002-auth-signup.md) |
| API-003 | POST | `/api/v1/auth/token/refresh` | 토큰 리프레시 | [003-auth-token-refresh.md](./003-auth-token-refresh.md) |
| API-029 | POST | `/api/v1/auth/logout` | 로그아웃 | [029-auth-logout.md](./029-auth-logout.md) |

### 회고방 (Retro Room) - 005~011

| API ID | Method | Endpoint | 설명 | 문서 |
|--------|--------|----------|------|------|
| API-005 | POST | `/api/v1/retro-rooms` | 회고방 생성 | [005-retro-room-create.md](./005-retro-room-create.md) |
| API-006 | POST | `/api/v1/retro-rooms/join` | 회고방 합류 (초대 링크) | [006-retro-room-join.md](./006-retro-room-join.md) |
| API-007 | GET | `/api/v1/retro-rooms` | 참여 회고방 목록 조회 | [007-retro-room-list.md](./007-retro-room-list.md) |
| API-008 | PATCH | `/api/v1/retro-rooms/order` | 회고방 순서 변경 | [008-retro-room-order-update.md](./008-retro-room-order-update.md) |
| API-009 | PATCH | `/api/v1/retro-rooms/{retroRoomId}/name` | 회고방 이름 변경 | [009-retro-room-name-update.md](./009-retro-room-name-update.md) |
| API-010 | DELETE | `/api/v1/retro-rooms/{retroRoomId}` | 회고방 삭제 | [010-retro-room-delete.md](./010-retro-room-delete.md) |
| API-011 | GET | `/api/v1/retro-rooms/{retroRoomId}/retrospects` | 회고방 내 회고 목록 조회 | [011-retro-room-retrospects-list.md](./011-retro-room-retrospects-list.md) |

### 회고 (Retrospect) - 012~024

| API ID | Method | Endpoint | 설명 | 문서 |
|--------|--------|----------|------|------|
| API-012 | POST | `/api/v1/retrospects` | 회고 생성 | [012-retrospect-create.md](./012-retrospect-create.md) |
| API-013 | GET | `/api/v1/retrospects/{retrospectId}` | 회고 상세 정보 조회 | [013-retrospect-detail.md](./013-retrospect-detail.md) |
| API-014 | DELETE | `/api/v1/retrospects/{retrospectId}` | 회고 삭제 | [014-retrospect-delete.md](./014-retrospect-delete.md) |
| API-015 | POST | `/api/v1/retrospects/{retrospectId}/participants` | 회고 참석 등록 | [015-retrospect-participant-create.md](./015-retrospect-participant-create.md) |
| API-016 | GET | `/api/v1/retrospects/{retrospectId}/participants` | 회고 참여자 및 질문 조회 | [016-retrospect-participants-list.md](./016-retrospect-participants-list.md) |
| API-017 | PUT | `/api/v1/retrospects/{retrospectId}/drafts` | 회고 답변 임시 저장 | [017-retrospect-draft-save.md](./017-retrospect-draft-save.md) |
| API-018 | POST | `/api/v1/retrospects/{retrospectId}/submit` | 회고 최종 제출 | [018-retrospect-submit.md](./018-retrospect-submit.md) |
| API-019 | GET | `/api/v1/retrospects/{retrospectId}/references` | 회고 참고자료 목록 조회 | [019-retrospect-references-list.md](./019-retrospect-references-list.md) |
| API-020 | GET | `/api/v1/retrospects/storage` | 보관함 회고 리스트 조회 | [020-retrospect-storage-list.md](./020-retrospect-storage-list.md) |
| API-021 | GET | `/api/v1/retrospects/{retrospectId}/responses` | 회고 답변 카테고리별 조회 | [021-retrospect-responses-list.md](./021-retrospect-responses-list.md) |
| API-022 | GET | `/api/v1/retrospects/{retrospectId}/export` | 회고 PDF 내보내기 | [022-retrospect-export.md](./022-retrospect-export.md) |
| API-023 | POST | `/api/v1/retrospects/{retrospectId}/analysis` | 회고 AI 분석 | [023-retrospect-analysis.md](./023-retrospect-analysis.md) |
| API-024 | GET | `/api/v1/retrospects/search` | 보관함 회고 검색 | [024-retrospect-search.md](./024-retrospect-search.md) |

### 회원 (Member) - 025

| API ID | Method | Endpoint | 설명 | 문서 |
|--------|--------|----------|------|------|
| API-025 | POST | `/api/v1/members/withdraw` | 서비스 탈퇴 | [025-member-withdraw.md](./025-member-withdraw.md) |

### 응답/댓글 (Response) - 026~028

| API ID | Method | Endpoint | 설명 | 문서 |
|--------|--------|----------|------|------|
| API-026 | POST | `/api/v1/responses/{responseId}/likes` | 답변 좋아요 토글 | [026-response-like-toggle.md](./026-response-like-toggle.md) |
| API-027 | GET | `/api/v1/responses/{responseId}/comments` | 답변 댓글 조회 | [027-response-comments-list.md](./027-response-comments-list.md) |
| API-028 | POST | `/api/v1/responses/{responseId}/comments` | 답변 댓글 작성 | [028-response-comment-create.md](./028-response-comment-create.md) |

### AI 어시스턴트 (AI Assistant) - 030

| API ID | Method | Endpoint | 설명 | 문서 |
|--------|--------|----------|------|------|
| API-030 | POST | `/api/v1/retrospects/{retrospectId}/questions/{questionId}/assistant` | 회고 질문별 AI 어시스턴트 | [030-retrospect-assistant.md](./030-retrospect-assistant.md) |

---

## 공통 응답 형식

### 성공 응답

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공 메시지",
  "result": { ... }
}
```

### 에러 응답

```json
{
  "isSuccess": false,
  "code": "ERROR_CODE",
  "message": "에러 메시지",
  "result": null
}
```

---

## 에러 코드 체계

| 접두사 | 도메인 | 예시 |
|--------|--------|------|
| AUTH | 인증 | AUTH4001, AUTH4002, AUTH4003 |
| RETRO | 회고/회고방 | RETRO4001, RETRO4031, RETRO4041, RETRO4043, RETRO4091 |
| DRAFT | 임시저장 | DRAFT4001, DRAFT4002 |
| RES | 응답/댓글 | RES4001, RES4041 |
| MEMBER | 회원 | MEMBER4001, MEMBER4041, MEMBER4042 |
| AI | AI 분석/어시스턴트 | AI4031, AI4032, AI5001 |
| SEARCH | 검색 | SEARCH4001 |
| COMMON | 공통 | COMMON400, COMMON500 |

### 에러 코드 번호 규칙

| 번호 패턴 | HTTP 상태 | 의미 |
|----------|-----------|------|
| X001~X009 | 400 | 유효성 검사 실패 |
| X031~X039 | 403 | 권한 없음 |
| X041~X049 | 404 | 리소스 없음 |
| X091~X099 | 409 | 충돌 (중복 등) |
| X5XX | 500 | 서버 내부 에러 |

---

## 인증 방식

대부분의 API는 `Authorization` 헤더를 통한 Bearer 토큰 인증이 필요합니다.

```
Authorization: Bearer {accessToken}
```

예외:
- API-001 (소셜 로그인): 소셜 서비스 토큰으로 인증

---

## 버전 관리 정책

각 API 문서는 개별 버전을 관리하며, 변경 시 다음 규칙을 따릅니다:

- **Major (X.0.0)**: Breaking Change (호환성 깨짐)
- **Minor (0.X.0)**: 새 기능 추가 (호환성 유지)
- **Patch (0.0.X)**: 버그 수정, 문서 개선
