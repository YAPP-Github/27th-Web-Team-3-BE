# API-007: 레트로룸 순서 변경 API 구현 리뷰

## 개요

| 항목 | 내용 |
|------|------|
| **API** | PATCH /api/v1/retro-rooms/order |
| **브랜치** | feat/team-generate |
| **베이스 브랜치** | dev |
| **명세서** | docs/api-specs/007-team-order-update.md |

## 구현 내용

### 엔드포인트
- **Method**: PATCH
- **Path**: `/api/v1/retro-rooms/order`
- **인증**: Bearer Token 필수

### 요청 구조
```json
{
  "retroRoomOrders": [
    { "retroRoomId": 456, "orderIndex": 1 },
    { "retroRoomId": 789, "orderIndex": 2 }
  ]
}
```

### 응답 구조
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공입니다.",
  "result": null
}
```

## 변경 파일

| 파일 | 변경 내용 |
|------|----------|
| `src/domain/retrospect/dto.rs` | RetroRoomOrderItem, UpdateRetroRoomOrderRequest, SuccessEmptyResponse DTO 추가 |
| `src/domain/retrospect/service.rs` | `update_retro_room_order()` 메서드 추가 |
| `src/domain/retrospect/handler.rs` | `update_retro_room_order` 핸들러 추가 + Swagger 문서화 |
| `src/utils/error.rs` | InvalidOrderData, NoPermission 에러 타입 추가 |
| `tests/api_007_retro_room_order_test.rs` | 단위 테스트 8개 |
| `src/main.rs` | 라우트 등록 |

## 비즈니스 로직

### 서비스 흐름 (`RetrospectService::update_retro_room_order`)
1. `orderIndex` 중복 체크 (HashSet 사용)
2. 각 룸에 대해:
   - 사용자가 해당 룸의 멤버인지 확인
   - `member_retro_room.order_index` 업데이트

### 유효성 검증
- `retroRoomOrders` 배열은 최소 1개 이상
- `orderIndex`는 1 이상의 값
- `orderIndex` 중복 불가

### 에러 처리

| 상황 | HTTP | 코드 | 메시지 |
|------|------|------|--------|
| 인증 실패 | 401 | COMMON401 | 인증 실패: ... |
| 빈 배열 전송 | 400 | COMMON400 | 최소 1개 이상의 순서 정보가 필요합니다. |
| orderIndex가 0 이하 | 400 | COMMON400 | orderIndex는 1 이상이어야 합니다. |
| orderIndex 중복 | 400 | TEAM4004 | 잘못된 순서 데이터입니다. |
| 멤버가 아닌 룸 | 403 | TEAM4031 | 순서를 변경할 권한이 없습니다. |
| 서버 오류 | 500 | COMMON500 | 서버 에러, 관리자에게 문의 바랍니다. |

## 테스트 커버리지

### 단위 테스트 (8개) - `tests/api_007_retro_room_order_test.rs`

| 테스트 | 검증 내용 |
|--------|----------|
| `should_validate_order_request_success` | 유효한 순서 데이터로 검증 통과 |
| `should_fail_validation_when_order_list_is_empty` | 빈 배열 → 검증 실패 |
| `should_fail_validation_when_order_index_is_zero` | orderIndex=0 → 검증 실패 |
| `should_fail_validation_when_order_index_is_negative` | orderIndex=-1 → 검증 실패 |
| `should_validate_order_index_with_large_value` | 큰 orderIndex 값 허용 검증 |
| `should_deserialize_order_request_from_camel_case` | camelCase JSON 역직렬화 검증 |
| `should_deserialize_order_request_with_multiple_items` | 다중 아이템 역직렬화 검증 |
| `should_serialize_order_item_in_camel_case` | camelCase 직렬화 검증 |

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
