# API-006: 레트로룸 목록 조회 API 구현 리뷰

## 개요

| 항목 | 내용 |
|------|------|
| **API** | GET /api/v1/retro-rooms |
| **브랜치** | feat/team-generate |
| **베이스 브랜치** | dev |
| **명세서** | docs/api-specs/006-team-list.md |

## 구현 내용

### 엔드포인트
- **Method**: GET
- **Path**: `/api/v1/retro-rooms`
- **인증**: Bearer Token 필수

### 응답 구조
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
    },
    {
      "retroRoomId": 456,
      "retroRoomName": "프로젝트 B",
      "orderIndex": 2
    }
  ]
}
```

## 변경 파일

| 파일 | 변경 내용 |
|------|----------|
| `src/domain/retrospect/dto.rs` | RetroRoomListItem, SuccessRetroRoomListResponse DTO 추가 |
| `src/domain/retrospect/service.rs` | `list_retro_rooms()` 메서드 추가 |
| `src/domain/retrospect/handler.rs` | `list_retro_rooms` 핸들러 추가 + Swagger 문서화 |
| `src/domain/member/entity/member_retro_room.rs` | `order_index: i32` 필드 추가 |
| `tests/api_006_retro_room_list_test.rs` | 단위 테스트 4개 |
| `src/main.rs` | 라우트 등록 |

## 비즈니스 로직

### 서비스 흐름 (`RetrospectService::list_retro_rooms`)
1. `member_retro_room` 테이블에서 member_id로 필터링
2. `order_index` 기준 오름차순 정렬
3. 각 룸의 `retro_room` 정보 조회
4. `RetroRoomListItem` 리스트로 변환하여 반환

### 에러 처리

| 상황 | HTTP | 코드 | 메시지 |
|------|------|------|--------|
| 인증 실패 | 401 | COMMON401 | 인증 실패: ... |
| 서버 오류 | 500 | COMMON500 | 서버 에러, 관리자에게 문의 바랍니다. |

## 테스트 커버리지

### 단위 테스트 (4개) - `tests/api_006_retro_room_list_test.rs`

| 테스트 | 검증 내용 |
|--------|----------|
| `should_serialize_list_item_in_camel_case` | RetroRoomListItem이 camelCase로 직렬화됨 |
| `should_serialize_empty_list_response` | 빈 배열 응답 시 `"result":[]` 형태로 직렬화됨 |
| `should_serialize_list_with_multiple_items` | 다중 아이템 목록 직렬화 검증 |
| `should_preserve_order_index_values` | order_index 값이 보존되는지 검증 |

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
