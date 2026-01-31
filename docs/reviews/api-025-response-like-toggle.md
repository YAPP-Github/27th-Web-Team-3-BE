# API-025 회고 답변 좋아요 토글 구현 리뷰

## 개요

특정 회고 답변에 좋아요를 등록하거나 취소하는 토글 API입니다.

## 엔드포인트

```
POST /api/v1/responses/{responseId}/likes
```

## 구현 내용

### 파일 변경 사항

| 파일 | 변경 유형 | 설명 |
|------|----------|------|
| `src/utils/error.rs` | 수정 | `ResponseNotFound` (RES4041) 에러 타입 추가 |
| `src/domain/retrospect/dto.rs` | 수정 | `LikeToggleResponse`, `SuccessLikeToggleResponse` DTO 추가 |
| `src/domain/retrospect/service.rs` | 수정 | `toggle_like` 비즈니스 로직 추가 |
| `src/domain/retrospect/handler.rs` | 수정 | `toggle_like` 핸들러 + Swagger 문서 추가 |
| `src/main.rs` | 수정 | 라우트 등록 및 OpenAPI 스키마/경로 추가 |

### 비즈니스 로직 흐름

```
1. 답변(response) 존재 확인 → RES4041
2. 회고 정보 조회 → 팀 ID 확인
3. 팀 멤버십 확인 → RETRO4031
4. 기존 좋아요 확인
   - 좋아요 있으면 → 삭제 (취소)
   - 좋아요 없으면 → 추가 (등록)
5. 총 좋아요 개수 조회 후 응답 반환
```

### 응답 형식

**성공 (200)**:
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "좋아요 상태가 성공적으로 업데이트되었습니다.",
  "result": {
    "responseId": 456,
    "isLiked": true,
    "totalLikes": 13
  }
}
```

### 에러 코드

| 코드 | HTTP | 발생 조건 |
|------|------|----------|
| AUTH4001 | 401 | 인증 실패 |
| RETRO4031 | 403 | 팀 멤버가 아닌 유저 |
| RES4041 | 404 | 존재하지 않는 답변 |
| COMMON500 | 500 | 서버 내부 오류 |

## 코드 품질 검사

- [x] `cargo test` - 모든 테스트 통과
- [x] `cargo clippy -- -D warnings` - 경고 없음
- [x] `cargo fmt` - 포맷팅 적용

## 참고

- API 스펙: `docs/api-specs/025-response-like-toggle.md`
