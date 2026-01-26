# API-010: 레트로룸 내 회고 목록 조회 API 구현 리뷰

## 개요

| 항목 | 내용 |
|------|------|
| **API** | GET /api/v1/retro-rooms/{retroRoomId}/retrospects |
| **브랜치** | feat/team-generate |
| **베이스 브랜치** | dev |
| **명세서** | docs/api-specs/010-retro-room-retrospects.md |

## 구현 내용

### 엔드포인트
- **Method**: GET
- **Path**: `/api/v1/retro-rooms/{retro_room_id}/retrospects`
- **인증**: Bearer Token 필수
- **권한**: 해당 룸의 멤버만 가능

### 응답 구조
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공입니다.",
  "result": [
    {
      "retrospectId": 100,
      "projectName": "프로젝트 회고",
      "retrospectMethod": "KPT",
      "retrospectDate": "2026-01-20",
      "retrospectTime": "10:00"
    },
    {
      "retrospectId": 101,
      "projectName": "스프린트 회고",
      "retrospectMethod": "KPT",
      "retrospectDate": "2026-01-24",
      "retrospectTime": "16:00"
    }
  ]
}
```

## 변경 파일

| 파일 | 변경 내용 |
|------|----------|
| `src/domain/retrospect/dto.rs` | RetrospectListItem, SuccessRetrospectListResponse DTO 추가 + 단위 테스트 2개 |
| `src/domain/retrospect/service.rs` | `list_retrospects()` 메서드 추가 |
| `src/domain/retrospect/handler.rs` | `list_retrospects` 핸들러 추가 + Swagger 문서화 |
| `src/main.rs` | 라우트 등록 |

## 비즈니스 로직

### 서비스 흐름 (`RetrospectService::list_retrospects`)
1. `retro_room_id`로 룸 존재 확인
2. `member_retro_room`에서 멤버 여부 확인
3. `retrospects` 테이블에서 해당 룸의 회고 조회
4. `start_time` 기준 내림차순(최신순) 정렬
5. 날짜/시간 포맷팅하여 반환

### 응답 필드 매핑

| Response 필드 | Entity 필드 | 변환 |
|---------------|-------------|------|
| `retrospectId` | `retrospect_id` | 그대로 |
| `projectName` | `title` | 그대로 |
| `retrospectMethod` | `retro_category` | Enum → String (대문자) |
| `retrospectDate` | `start_time` | `%Y-%m-%d` 포맷 |
| `retrospectTime` | `start_time` | `%H:%M` 포맷 |

### retrospectMethod Enum

| 값 | 설명 |
|----|------|
| KPT | Keep-Problem-Try 방식 |
| FOUR_L | 4L (Liked, Learned, Lacked, Longed For) 방식 |
| FIVE_F | 5F (Facts, Feelings, Findings, Future, Feedback) 방식 |
| PMI | Plus-Minus-Interesting 방식 |
| FREE | 자유 형식 |

### 에러 처리

| 상황 | HTTP | 코드 | 메시지 |
|------|------|------|--------|
| 인증 실패 | 401 | COMMON401 | 인증 실패: ... |
| 멤버가 아님 | 403 | TEAM4031 | 해당 레트로룸에 접근 권한이 없습니다. |
| 룸이 존재하지 않음 | 404 | RETRO4041 | 존재하지 않는 레트로룸입니다. |
| 서버 오류 | 500 | COMMON500 | 서버 에러, 관리자에게 문의 바랍니다. |

## 테스트 커버리지

### 단위 테스트 - dto.rs (2개)

| 테스트 | 검증 내용 |
|--------|----------|
| `should_serialize_retrospect_list_item_in_camel_case` | RetrospectListItem이 camelCase로 직렬화됨 (retrospectId, projectName, retrospectMethod, retrospectDate, retrospectTime) |
| `should_serialize_empty_retrospect_list` | 빈 배열 응답 시 `"result":[]` 형태로 직렬화됨 |

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
cargo test     → 31 passed, 0 failed
cargo clippy   → 0 errors, 0 warnings
cargo fmt      → clean
```
