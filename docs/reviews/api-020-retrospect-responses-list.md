# API-020: 회고 답변 카테고리별 조회 API 구현 리뷰

## 개요

| 항목 | 내용 |
|------|------|
| **API** | GET /api/v1/retrospects/{retrospectId}/responses |
| **브랜치** | feature/api-020-retrospect-responses-list |
| **베이스 브랜치** | feature/api-013-retrospect-delete |
| **명세서** | docs/api-specs/020-retrospect-responses-list.md |

## 구현 내용

### 엔드포인트
- **Method**: GET
- **Path**: `/api/v1/retrospects/{retrospectId}/responses`
- **인증**: Bearer Token 필수
- **Path 파라미터**: `retrospectId` (필수, 1 이상의 양수)
- **쿼리 파라미터**:
  - `category` (필수): ALL, QUESTION_1~QUESTION_5
  - `cursor` (선택): 마지막 조회된 답변 ID
  - `size` (선택, 기본값: 10, 범위: 1~100)

### 카테고리 필터 (ResponseCategory)

| 값 | 설명 | 질문 인덱스 |
|----|------|------------|
| `ALL` | 전체 답변 조회 | 전체 |
| `QUESTION_1` | 질문 1에 대한 답변만 | 0 |
| `QUESTION_2` | 질문 2에 대한 답변만 | 1 |
| `QUESTION_3` | 질문 3에 대한 답변만 | 2 |
| `QUESTION_4` | 질문 4에 대한 답변만 | 3 |
| `QUESTION_5` | 질문 5에 대한 답변만 | 4 |

### 응답 구조
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "답변 리스트 조회를 성공했습니다.",
  "result": {
    "responses": [
      {
        "responseId": 501,
        "userName": "제이슨",
        "content": "이번 스프린트에서 테스트 코드를 꼼꼼히 짠 것이 좋았습니다.",
        "likeCount": 12,
        "commentCount": 3
      }
    ],
    "hasNext": true,
    "nextCursor": 455
  }
}
```

## 변경 파일

### 신규 파일

| 파일 | 설명 |
|------|------|
| `tests/retrospect_responses_list_test.rs` | 통합 테스트 (19개) |

### 수정 파일

| 파일 | 변경 내용 |
|------|----------|
| `src/utils/error.rs` | `RetroCategoryInvalid` 에러 변형 추가 (RETRO4004, 400) |
| `src/domain/retrospect/dto.rs` | ResponseCategory, ResponsesQueryParams, ResponseListItem, ResponsesListResponse, SuccessResponsesListResponse DTO 추가 + 단위 테스트 18개 |
| `src/domain/retrospect/service.rs` | `list_responses()` 메서드 추가 |
| `src/domain/retrospect/handler.rs` | `list_responses` 핸들러 추가 + Swagger 문서화 |
| `src/main.rs` | 라우트 등록 + OpenAPI 스키마/경로 추가 |

## 비즈니스 로직

### 서비스 흐름 (`RetrospectService::list_responses`)
1. 회고 조회 및 팀 멤버십 확인 (`find_retrospect_for_member`)
2. 해당 회고의 전체 response 조회 (response_id 오름차순)
3. `member_response`를 통해 질문 순서 결정 (첫 번째 참여자의 응답 기준)
4. 카테고리에 따른 대상 응답 필터링 (질문 텍스트 매칭)
5. 빈 답변(공백만 있는 content) 제외
6. 커서 기반 페이지네이션 (response_id 내림차순, size+1 조회로 hasNext 판단)
7. `member_response` → `member` 조인으로 작성자 닉네임 조회
8. `response_like`, `response_comment` COUNT 집계
9. DTO 변환 및 nextCursor 계산

### 커서 기반 페이지네이션
- response_id 내림차순으로 정렬 (최신 답변부터)
- `cursor` 값보다 작은 response_id를 조회 (`WHERE response_id < cursor`)
- `size + 1`개를 조회하여 다음 페이지 존재 여부 확인
- 마지막 페이지: `hasNext: false`, `nextCursor: null`

### 에러 처리

| 상황 | HTTP | 코드 | 메시지 |
|------|------|------|--------|
| 인증 실패 | 401 | AUTH4001 | 인증 정보가 유효하지 않습니다. |
| retrospectId 유효성 | 400 | COMMON400 | retrospectId는 1 이상의 양수여야 합니다. |
| category 누락/유효하지 않음 | 400 | RETRO4004 | 유효하지 않은 카테고리 값입니다. |
| cursor 유효성 | 400 | COMMON400 | cursor는 1 이상의 양수여야 합니다. |
| size 범위 초과 | 400 | COMMON400 | size는 1~100 범위의 정수여야 합니다. |
| 회고 미존재 / 접근 불가 | 404 | RETRO4041 | 존재하지 않는 회고이거나 접근 권한이 없습니다. |
| 서버 오류 | 500 | COMMON500 | 서버 에러, 관리자에게 문의 바랍니다. |

## 테스트 커버리지

### 단위 테스트 (18개) - dto.rs

| 테스트 | 검증 내용 |
|--------|----------|
| `should_deserialize_all_response_category` | ALL 역직렬화 + question_index None |
| `should_deserialize_question_1_category` | QUESTION_1 역직렬화 + index 0 |
| `should_deserialize_question_2_category` | QUESTION_2 역직렬화 + index 1 |
| `should_deserialize_question_3_category` | QUESTION_3 역직렬화 + index 2 |
| `should_deserialize_question_4_category` | QUESTION_4 역직렬화 + index 3 |
| `should_deserialize_question_5_category` | QUESTION_5 역직렬화 + index 4 |
| `should_fail_deserialize_invalid_response_category` | 잘못된 값 역직렬화 실패 |
| `should_fail_deserialize_question_6_category` | QUESTION_6 역직렬화 실패 |
| `should_display_response_category_correctly` | Display 트레이트 출력 검증 |
| `should_serialize_response_list_item_in_camel_case` | ResponseListItem camelCase 직렬화 |
| `should_serialize_response_list_item_with_zero_counts` | 0 카운트 직렬화 |
| `should_serialize_responses_list_response_in_camel_case` | 페이지네이션 포함 직렬화 |
| `should_serialize_empty_responses_list_response` | 빈 응답 직렬화 |
| `should_serialize_last_page_responses` | 마지막 페이지 직렬화 |
| `should_serialize_success_responses_list_response_in_camel_case` | 전체 응답 래핑 직렬화 |
| `should_deserialize_responses_query_params_with_all_fields` | 전체 필드 역직렬화 |
| `should_deserialize_responses_query_params_with_category_only` | 카테고리만 역직렬화 |
| `should_deserialize_responses_query_params_without_optional_fields` | 선택 필드 누락 역직렬화 |

### 통합 테스트 (19개) - retrospect_responses_list_test.rs

| 테스트 | 검증 내용 |
|--------|----------|
| `api020_should_return_401_when_authorization_header_missing` | 인증 헤더 누락 → 401 |
| `api020_should_return_401_when_authorization_header_format_invalid` | 잘못된 인증 형식 → 401 |
| `api020_should_return_400_when_retrospect_id_is_zero` | retrospectId=0 → 400 |
| `api020_should_return_400_when_retrospect_id_is_negative` | retrospectId=-1 → 400 |
| `api020_should_return_400_when_category_is_missing` | category 누락 → 400 RETRO4004 |
| `api020_should_return_400_when_category_is_invalid` | 잘못된 category → 400 RETRO4004 |
| `api020_should_return_400_when_cursor_is_zero` | cursor=0 → 400 |
| `api020_should_return_400_when_size_is_out_of_range` | size=101 → 400 |
| `api020_should_return_400_when_size_is_zero` | size=0 → 400 |
| `api020_should_return_404_when_retrospect_not_found` | 존재하지 않는 회고 → 404 |
| `api020_should_return_403_when_access_denied` | 접근 권한 없음 → 403 |
| `api020_should_return_200_with_all_responses` | category=ALL 성공 응답 |
| `api020_should_return_correct_response_fields` | 응답 필드 구조 검증 |
| `api020_should_return_pagination_fields` | hasNext/nextCursor 검증 |
| `api020_should_return_filtered_responses_for_question_1` | QUESTION_1 필터링 |
| `api020_should_return_null_cursor_on_last_page` | 마지막 페이지 검증 |
| `api020_should_return_empty_responses_for_unused_category` | 빈 결과 처리 |
| `api020_should_use_camel_case_field_names_in_response` | camelCase 필드명 검증 |
| `api020_should_return_responses_sorted_by_response_id_descending` | 내림차순 정렬 검증 |

## 코드 리뷰 체크리스트

- [x] TDD 원칙을 따라 테스트 코드가 먼저 작성되었는가?
- [x] 모든 테스트가 통과하는가? (159개 전체 통과)
- [x] API 문서가 `docs/reviews/` 디렉토리에 작성되었는가?
- [x] 공통 유틸리티를 재사용했는가? (BaseResponse, AppError, AuthUser, find_retrospect_for_member)
- [x] 에러 처리가 적절하게 되어 있는가? (Result + ? 연산자)
- [x] 코드가 Rust 컨벤션을 따르는가? (camelCase DTO, snake_case 함수)
- [x] 불필요한 의존성이 추가되지 않았는가?
- [x] `cargo clippy -- -D warnings` 통과
- [x] `cargo fmt` 적용 완료
- [x] Swagger/OpenAPI 문서화 완료

## 품질 검증 결과
```text
cargo test     → 159 passed, 0 failed (unit 140 + integration 19)
cargo clippy   → 0 errors, 0 warnings
cargo fmt      → clean
```
