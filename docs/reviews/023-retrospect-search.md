# API-023: 회고 검색 API 구현 리뷰

## 개요
사용자가 참여하는 모든 회고방의 회고를 프로젝트명/회고명 기준으로 검색하는 API입니다.

## 엔드포인트
- **Method**: `GET`
- **Path**: `/api/v1/retrospects/search`
- **인증**: Bearer Token 필수

## 요청
| 파라미터 | 타입 | 필수 | 설명 |
|----------|------|------|------|
| keyword | string | Y | 검색 키워드 (1~100자) |

## 응답
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "검색을 성공했습니다.",
  "result": [
    {
      "retrospectId": 42,
      "projectName": "스프린트 회고",
      "retroRoomName": "회고방A",
      "retrospectMethod": "KPT",
      "retrospectDate": "2026-01-24",
      "retrospectTime": "14:30"
    }
  ]
}
```

## 에러 코드
| 코드 | HTTP | 설명 |
|------|------|------|
| SEARCH4001 | 400 | 검색어 누락 또는 유효하지 않음 |
| AUTH4001 | 401 | 인증 실패 |
| COMMON500 | 500 | 서버 내부 오류 |

## 구현 세부사항

### 변경 파일
| 파일 | 변경 내용 |
|------|----------|
| `src/utils/error.rs` | `SearchKeywordInvalid` 에러 변형 추가 (SEARCH4001) |
| `src/domain/retrospect/dto.rs` | `SearchQueryParams`, `SearchRetrospectItem`, `SuccessSearchResponse` DTO 추가 |
| `src/domain/retrospect/service.rs` | `search_retrospects` 비즈니스 로직 구현 |
| `src/domain/retrospect/handler.rs` | `search_retrospects` 핸들러 추가 |
| `src/main.rs` | 라우트 등록 및 Swagger 스키마 추가 |

### 비즈니스 로직 흐름
1. 키워드 검증 (`validate_search_keyword`: 빈 값, 100자 초과, trim 처리)
2. 사용자가 속한 회고방 목록 조회 (`member_retro_room`)
3. 회고방 정보 조회 (회고방명 매핑)
4. 해당 회고방들의 회고 중 키워드 포함 회고 검색 (`LIKE '%keyword%'`)
5. 결과를 `start_time DESC`, `retrospect_id DESC` 정렬 (안정 정렬)
6. `start_time`은 생성 시 KST로 저장되므로 변환 없이 직접 포맷

### 정렬 기준
- 1차: `start_time` 내림차순 (최신순)
- 2차: `retrospect_id` 내림차순 (동일 시간대 안정 정렬)
- DB 레벨에서 `start_time DESC, retrospect_id DESC`로 처리

### 검증 규칙
- 키워드 필수 (빈 문자열/공백만 불가)
- 키워드 최대 100자 제한
- 앞뒤 공백 자동 제거 (trim)

## 테스트

### DTO 테스트 (dto.rs)
- `should_serialize_search_retrospect_item_in_camel_case` - camelCase 직렬화 검증
- `should_serialize_search_response_with_all_retrospect_methods` - 모든 회고 방식 직렬화
- `should_serialize_success_search_response_in_camel_case` - 성공 응답 직렬화
- `should_serialize_empty_search_response` - 빈 결과 직렬화
- `should_deserialize_search_query_params_with_keyword` - 키워드 파라미터 역직렬화
- `should_fail_deserialize_search_query_params_without_keyword` - 키워드 누락 시 역직렬화 실패

### 서비스 테스트 (service.rs)
- `should_fail_when_keyword_is_empty` - 빈 키워드 → `SearchKeywordInvalid` 에러
- `should_fail_when_keyword_exceeds_100_chars` - 100자 초과 → `SearchKeywordInvalid` 에러
- `should_pass_when_keyword_is_exactly_100_chars` - 100자 경계값 통과
- `should_fail_when_keyword_is_whitespace_only` - 공백만 → `SearchKeywordInvalid` 에러
- `should_trim_keyword_with_leading_trailing_whitespace` - 앞뒤 공백 trim 후 반환
- `should_pass_valid_keyword` - 정상 키워드 통과

## 리뷰 후 개선사항

### v2 개선 (리뷰 피드백 반영)
1. **keyword 필수 파라미터화**: `Option<String>` → `String`으로 변경하여 OpenAPI 스키마와 서비스 로직의 기대치를 일치시킴
2. **이중 KST 변환 버그 수정**: `start_time`은 생성 시 KST NaiveDateTime으로 저장되므로, 검색 및 상세 API에서 `+9시간` 변환을 제거 (이중 변환 방지)
3. **검색 정렬 안정화**: `start_time DESC`에 `retrospect_id DESC` 보조 정렬 추가로 동일 시간대 레코드의 순서 보장
4. **검색 테스트 보강**: 키워드 검증을 `validate_search_keyword` 함수로 추출하고, `AppError` 반환을 직접 검증하는 테스트로 교체

## 코드 리뷰 체크리스트
- [x] TDD 원칙을 따라 테스트 코드가 작성되었는가?
- [x] 모든 테스트가 통과하는가? (101 passed)
- [x] API 문서가 `docs/reviews/` 디렉토리에 작성되었는가?
- [x] 공통 유틸리티를 재사용했는가? (BaseResponse, AppError, AuthUser)
- [x] 에러 처리가 적절하게 되어 있는가? (SEARCH4001, AUTH4001, COMMON500)
- [x] 코드가 Rust 컨벤션을 따르는가? (cargo fmt, cargo clippy)
- [x] 불필요한 의존성이 추가되지 않았는가?
