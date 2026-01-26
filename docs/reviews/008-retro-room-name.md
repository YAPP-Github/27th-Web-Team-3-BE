# API-008: 레트로룸 이름 변경 API 구현 리뷰

## 개요

| 항목 | 내용 |
|------|------|
| **API** | PATCH /api/v1/retro-rooms/{retroRoomId}/name |
| **브랜치** | feat/team-generate |
| **베이스 브랜치** | dev |
| **명세서** | docs/api-specs/008-team-name-update.md |

## 구현 내용

### 엔드포인트
- **Method**: PATCH
- **Path**: `/api/v1/retro-rooms/{retro_room_id}/name`
- **인증**: Bearer Token 필수
- **권한**: Owner만 가능

### 요청 구조
```json
{
  "name": "새로운 레트로룸 이름"
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
    "retroRoomName": "새로운 레트로룸 이름",
    "updatedAt": "2026-01-26T15:30:00"
  }
}
```

## 변경 파일

| 파일 | 변경 내용 |
|------|----------|
| `src/domain/retrospect/dto.rs` | UpdateRetroRoomNameRequest, UpdateRetroRoomNameResponse DTO 추가 |
| `src/domain/retrospect/service.rs` | `update_retro_room_name()` 메서드 추가 |
| `src/domain/retrospect/handler.rs` | `update_retro_room_name` 핸들러 추가 + Swagger 문서화 |
| `src/utils/error.rs` | NoRoomPermission 에러 타입 추가 |
| `tests/api_008_retro_room_name_test.rs` | 단위 테스트 10개 |
| `src/main.rs` | 라우트 등록 |

## 비즈니스 로직

### 서비스 흐름 (`RetrospectService::update_retro_room_name`)
1. `retro_room_id`로 룸 존재 확인
2. `member_retro_room`에서 Owner 역할 확인
3. 동일 이름의 다른 룸이 있는지 중복 체크
4. `retro_room.title` 및 `updated_at` 업데이트

### 유효성 검증
- `name`은 1자 이상 20자 이하
- 빈 문자열 불가

### 에러 처리

| 상황 | HTTP | 코드 | 메시지 |
|------|------|------|--------|
| 인증 실패 | 401 | COMMON401 | 인증 실패: ... |
| 이름이 빈 문자열 | 400 | COMMON400 | 레트로룸 이름은 1~20자여야 합니다. |
| 이름이 20자 초과 | 400 | COMMON400 | 레트로룸 이름은 1~20자여야 합니다. |
| Owner가 아님 | 403 | ROOM4031 | 레트로룸 이름을 변경할 권한이 없습니다. |
| 룸이 존재하지 않음 | 404 | RETRO4041 | 존재하지 않는 레트로룸입니다. |
| 이름 중복 | 409 | RETRO4091 | 이미 사용 중인 레트로룸 이름입니다. |
| 서버 오류 | 500 | COMMON500 | 서버 에러, 관리자에게 문의 바랍니다. |

## 테스트 커버리지

### 단위 테스트 (10개) - `tests/api_008_retro_room_name_test.rs`

| 테스트 | 검증 내용 |
|--------|----------|
| `should_validate_name_update_request_success` | 유효한 이름으로 검증 통과 |
| `should_fail_validation_when_name_is_empty` | 빈 문자열 → 검증 실패 |
| `should_fail_validation_when_name_exceeds_20_chars` | 21자 이상 → 검증 실패 |
| `should_allow_name_with_exactly_20_chars` | 정확히 20자 → 검증 통과 |
| `should_allow_name_with_exactly_1_char` | 정확히 1자 → 검증 통과 |
| `should_allow_name_with_korean_characters` | 한글 이름 허용 검증 |
| `should_allow_name_with_special_characters` | 특수문자 포함 이름 허용 검증 |
| `should_serialize_name_update_response_in_camel_case` | 응답이 camelCase로 직렬화됨 |
| `should_deserialize_name_request_from_camel_case` | 요청 역직렬화 검증 |

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
