# API-005: 레트로룸 참여 (Team Join) API 구현 리뷰

## 개요

| 항목 | 내용 |
|------|------|
| **API** | POST /api/v1/retro-rooms/join |
| **브랜치** | feat/team-generate |
| **베이스 브랜치** | dev |
| **명세서** | docs/api-specs/005-team-join.md |

## 구현 내용

### 엔드포인트
- **Method**: POST
- **Path**: `/api/v1/retro-rooms/join`
- **인증**: Bearer Token 필수

### 요청 구조
```json
{
  "inviteUrl": "https://service.com/invite/INV-A1B2-C3D4"
}
```

### 응답 구조
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공입니다.",
  "result": {
    "retroRoomId": 123,
    "title": "프로젝트 A 회고",
    "joinedAt": "2026-01-26T15:30:00"
  }
}
```

## 변경 파일

| 파일 | 변경 내용 |
|------|----------|
| `src/domain/retrospect/dto.rs` | JoinRetroRoomRequest, JoinRetroRoomResponse DTO 추가 |
| `src/domain/retrospect/service.rs` | `join_retro_room()`, `extract_invite_code()` 메서드 추가 |
| `src/domain/retrospect/handler.rs` | `join_retro_room` 핸들러 추가 + Swagger 문서화 |
| `src/utils/error.rs` | InvalidInviteLink, ExpiredInviteLink, AlreadyMember 에러 타입 추가 |
| `tests/api_005_retro_room_join_test.rs` | 단위 테스트 10개 |
| `src/main.rs` | 라우트 등록 + OpenAPI 스키마/경로 추가 |

## 비즈니스 로직

### 서비스 흐름 (`RetrospectService::join_retro_room`)
1. 초대 URL에서 초대 코드 추출 (`extract_invite_code`)
2. 초대 코드로 레트로룸 조회
3. 초대 링크 만료 체크 (7일)
4. 이미 참여 중인 멤버인지 확인
5. `member_retro_room` 테이블에 Member 권한으로 추가
6. 응답 반환 (retroRoomId, title, joinedAt)

### 초대 코드 추출 (`extract_invite_code`)
지원 형식:
- **Path segment**: `https://service.com/invite/INV-A1B2-C3D4`
- **Query parameter**: `https://service.com/join?code=INV-A1B2-C3D4`
- **다중 쿼리 파라미터**: `https://service.com/join?ref=abc&code=INV-A1B2-C3D4&foo=bar`

### 에러 처리

| 상황 | HTTP | 코드 | 메시지 |
|------|------|------|--------|
| 인증 실패 | 401 | COMMON401 | 인증 실패: ... |
| 유효하지 않은 URL 형식 | 400 | COMMON400 | 유효한 URL 형식이 아닙니다. |
| 유효하지 않은 초대 코드 | 400 | RETRO4002 | 유효하지 않은 초대 링크입니다. |
| 만료된 초대 링크 (7일) | 400 | RETRO4003 | 만료된 초대 링크입니다. |
| 존재하지 않는 룸 | 404 | RETRO4041 | 존재하지 않는 회고 룸입니다. |
| 이미 참여 중 | 409 | RETRO4092 | 이미 해당 회고 룸의 멤버입니다. |
| 서버 오류 | 500 | COMMON500 | 서버 에러, 관리자에게 문의 바랍니다. |

## 테스트 커버리지

### 단위 테스트 (10개) - `tests/api_005_retro_room_join_test.rs`

| 테스트 | 검증 내용 |
|--------|----------|
| `should_validate_join_request_with_valid_url` | 유효한 URL로 요청 시 검증 통과 |
| `should_fail_validation_with_invalid_url_format` | 잘못된 URL 형식 → 검증 실패 |
| `should_validate_join_request_with_query_param_url` | 쿼리 파라미터 형식 URL 검증 통과 |
| `should_serialize_join_response_in_camel_case` | 응답이 camelCase로 직렬화됨 |
| `should_extract_invite_code_from_path_segment` | Path segment에서 초대 코드 추출 |
| `should_extract_invite_code_from_query_parameter` | Query parameter에서 초대 코드 추출 |
| `should_extract_invite_code_from_query_with_multiple_params` | 다중 쿼리 파라미터에서 code 추출 |
| `should_return_error_for_invalid_url` | 유효하지 않은 URL → 에러 반환 |
| `should_return_error_for_empty_code` | 빈 code 파라미터 → 에러 반환 |
| `should_generate_valid_invite_code` | 초대 코드 생성 형식 검증 (INV-XXXX-XXXX) |

## 코드 리뷰 체크리스트

- [x] TDD 원칙을 따라 테스트 코드가 먼저 작성되었는가?
- [x] 모든 테스트가 통과하는가?
- [x] 공통 유틸리티를 재사용했는가? (BaseResponse, AppError, AuthUser)
- [x] 에러 처리가 적절하게 되어 있는가? (Result + ? 연산자)
- [x] 코드가 Rust 컨벤션을 따르는가? (camelCase DTO, snake_case 함수)
- [x] `cargo clippy -- -D warnings` 통과
- [x] `cargo fmt` 적용 완료
- [x] Swagger/OpenAPI 문서화 완료

## 품질 검증 결과
```text
cargo test     → 48 passed, 0 failed
cargo clippy   → 0 errors, 0 warnings
cargo fmt      → clean
```
