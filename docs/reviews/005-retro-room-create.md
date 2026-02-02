# API-004: 레트로룸 생성 API 구현 리뷰

## 개요

| 항목 | 내용 |
|------|------|
| **API** | POST /api/v1/retro-rooms |
| **브랜치** | feat/team-generate |
| **베이스 브랜치** | dev |
| **명세서** | docs/api-specs/004-team-create.md |

## 구현 내용

### 엔드포인트
- **Method**: POST
- **Path**: `/api/v1/retro-rooms`
- **인증**: Bearer Token 필수

### 요청 구조
```json
{
  "title": "프로젝트 회고",
  "description": "스프린트 회고입니다"
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
    "title": "프로젝트 회고",
    "inviteCode": "INV-A1B2-C3D4"
  }
}
```

## 변경 파일

| 파일 | 변경 내용 |
|------|----------|
| `src/domain/retrospect/dto.rs` | RetroRoomCreateRequest, RetroRoomCreateResponse DTO 추가 |
| `src/domain/retrospect/service.rs` | `create_retro_room()` 메서드 추가 |
| `src/domain/retrospect/handler.rs` | `create_retro_room` 핸들러 추가 + Swagger 문서화 |
| `tests/api_004_retro_room_create_test.rs` | 단위 테스트 7개 |
| `src/main.rs` | 라우트 등록 |

## 비즈니스 로직

### 서비스 흐름 (`RetrospectService::create_retro_room`)
1. 요청 데이터 유효성 검증
2. 초대 코드 생성 (INV-XXXX-XXXX 형식)
3. `retro_room` 테이블에 새 룸 생성
4. `member_retro_room` 테이블에 생성자를 Owner로 등록
5. 생성 결과 반환

### 유효성 검증
- `title`은 1자 이상 20자 이하
- `description`은 최대 50자 (선택 사항)

### 에러 처리

| 상황 | HTTP | 코드 | 메시지 |
|------|------|------|--------|
| 인증 실패 | 401 | COMMON401 | 인증 실패: ... |
| 제목이 빈 문자열 | 400 | COMMON400 | 레트로룸 이름은 1~20자여야 합니다. |
| 제목이 20자 초과 | 400 | COMMON400 | 레트로룸 이름은 1~20자여야 합니다. |
| 설명이 50자 초과 | 400 | COMMON400 | 설명은 50자 이하여야 합니다. |
| 서버 오류 | 500 | COMMON500 | 서버 에러, 관리자에게 문의 바랍니다. |

## 테스트 커버리지

### 단위 테스트 (7개) - `tests/api_004_retro_room_create_test.rs`

| 테스트 | 검증 내용 |
|--------|----------|
| `should_validate_retro_room_create_request_success` | 유효한 요청으로 검증 통과 |
| `should_fail_validation_when_title_exceeds_20_chars` | 21자 이상 제목 → 검증 실패 |
| `should_fail_validation_when_description_exceeds_50_chars` | 51자 이상 설명 → 검증 실패 |
| `should_allow_empty_description` | 설명 없이 검증 통과 |
| `should_serialize_create_response_in_camel_case` | 응답이 camelCase로 직렬화됨 |
| `should_allow_title_with_exactly_20_chars` | 정확히 20자 제목 허용 |
| `should_allow_description_with_exactly_50_chars` | 정확히 50자 설명 허용 |

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
