# API-019: 보관함 조회 API 구현 리뷰

## 개요

| 항목 | 내용 |
|------|------|
| **API** | GET /api/v1/retrospects/storage |
| **브랜치** | feature/api-019-retrospect-storage |
| **베이스 브랜치** | feature/api-017-retrospect-submit |
| **명세서** | docs/api-specs/api-019-retrospect-storage-list.md |

## 구현 내용

### 엔드포인트
- **Method**: GET
- **Path**: `/api/v1/retrospects/storage`
- **인증**: Bearer Token 필수
- **쿼리 파라미터**: `range` (선택, 기본값: ALL)

### 기간 필터 (StorageRangeFilter)

| 값 | 설명 | 일수 |
|----|------|------|
| `ALL` | 전체 기간 (기본값) | 제한 없음 |
| `3_MONTHS` | 최근 3개월 | 90일 |
| `6_MONTHS` | 최근 6개월 | 180일 |
| `1_YEAR` | 최근 1년 | 365일 |

### 응답 구조
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "보관함 조회를 성공했습니다.",
  "result": {
    "years": [
      {
        "yearLabel": "2026년",
        "retrospects": [
          {
            "retrospectId": 124,
            "displayDate": "2026-01-24",
            "title": "프로젝트명",
            "retroCategory": "KPT",
            "memberCount": 5
          }
        ]
      }
    ]
  }
}
```

## 변경 파일

### 신규 파일

| 파일 | 설명 |
|------|------|
| `tests/retrospect_storage_test.rs` | 통합 테스트 (12개) |

### 수정 파일

| 파일 | 변경 내용 |
|------|----------|
| `src/domain/retrospect/dto.rs` | StorageRangeFilter, StorageQueryParams, StorageRetrospectItem, StorageYearGroup, StorageResponse, SuccessStorageResponse DTO 추가 + 단위 테스트 8개 |
| `src/domain/retrospect/service.rs` | `get_storage()` 메서드 추가 |
| `src/domain/retrospect/handler.rs` | `get_storage` 핸들러 추가 + Swagger 문서화 |
| `src/domain/retrospect/entity/retrospect.rs` | RetroCategory enum에 FourL, FiveF, Pmi, Free 추가 + ToSchema derive |
| `src/main.rs` | 라우트 등록 + OpenAPI 스키마/경로 추가 |
| `src/config/app_config.rs` | dead_code 경고 억제 (기존 이슈) |
| `src/domain/auth/handler.rs` | 미사용 import 제거 (기존 이슈) |

## 비즈니스 로직

### 서비스 흐름 (`RetrospectService::get_storage`)
1. 기간 필터 기본값 설정 (ALL)
2. `member_retro` 테이블에서 사용자의 제출/분석 완료 회고 조회
3. 기간 필터에 따른 날짜 범위 필터링 (`submitted_at` 기준)
4. 관련 `retrospect` 정보 조회
5. 각 회고의 참여자 수(`member_count`) 조회
6. 연도별 그룹핑 (BTreeMap 사용)
7. 연도 내림차순 + 그룹 내 날짜 내림차순 정렬

### 에러 처리

| 상황 | HTTP | 코드 | 메시지 |
|------|------|------|--------|
| 인증 실패 | 401 | AUTH4001 | 인증 정보가 유효하지 않습니다. |
| 유효하지 않은 필터 | 400 | COMMON400 | 유효하지 않은 기간 필터입니다. |
| 서버 오류 | 500 | COMMON500 | 서버 에러, 관리자에게 문의 바랍니다. |

## 테스트 커버리지

### 단위 테스트 (8개) - dto.rs

| 테스트 | 검증 내용 |
|--------|----------|
| `should_deserialize_all_range_filter` | ALL 역직렬화 + days() None |
| `should_deserialize_3_months_range_filter` | 3_MONTHS 역직렬화 + days() 90 |
| `should_deserialize_6_months_range_filter` | 6_MONTHS 역직렬화 + days() 180 |
| `should_deserialize_1_year_range_filter` | 1_YEAR 역직렬화 + days() 365 |
| `should_fail_deserialize_invalid_range_filter` | 잘못된 값 역직렬화 실패 |
| `should_default_to_all` | 기본값 ALL 검증 |
| `should_display_range_filter_correctly` | Display 트레이트 출력 검증 |
| `should_serialize_storage_response_in_camel_case` | camelCase 직렬화 검증 |
| `should_serialize_empty_storage_response` | 빈 응답 직렬화 검증 |

### 통합 테스트 (12개) - retrospect_storage_test.rs

| 테스트 | 검증 내용 |
|--------|----------|
| `api019_should_return_401_when_authorization_header_missing` | 인증 헤더 누락 → 401 |
| `api019_should_return_401_when_authorization_header_format_invalid` | 잘못된 인증 형식 → 401 |
| `api019_should_return_400_when_range_filter_is_invalid` | 유효하지 않은 range → 400 |
| `api019_should_return_200_with_default_range_all` | 기본값(ALL) 성공 응답 |
| `api019_should_return_correct_retrospect_item_fields` | 응답 필드 구조 검증 |
| `api019_should_sort_retrospects_by_date_descending_within_year` | 그룹 내 최신순 정렬 |
| `api019_should_return_filtered_results_with_3_months_range` | 3개월 필터 적용 |
| `api019_should_return_empty_years_when_no_retrospects_in_range` | 빈 결과 처리 |
| `api019_should_return_empty_years_with_1_year_range` | 1년 필터 적용 |
| `api019_should_return_all_results_with_explicit_all_range` | 명시적 ALL 전달 |
| `api019_should_support_multiple_retro_categories` | 다양한 카테고리 지원 |
| `api019_should_sort_year_groups_descending` | 연도 내림차순 정렬 |

## 코드 리뷰 체크리스트

- [x] TDD 원칙을 따라 테스트 코드가 먼저 작성되었는가?
- [x] 모든 테스트가 통과하는가? (51개 전체 통과)
- [x] API 문서가 `docs/reviews/` 디렉토리에 작성되었는가?
- [x] 공통 유틸리티를 재사용했는가? (BaseResponse, AppError, AuthUser)
- [x] 에러 처리가 적절하게 되어 있는가? (Result + ? 연산자)
- [x] 코드가 Rust 컨벤션을 따르는가? (camelCase DTO, snake_case 함수)
- [x] 불필요한 의존성이 추가되지 않았는가?
- [x] `cargo clippy -- -D warnings` 통과
- [x] `cargo fmt` 적용 완료
- [x] Swagger/OpenAPI 문서화 완료

## 품질 검증 결과
```text
cargo test     → 51 passed, 0 failed
cargo clippy   → 0 errors, 0 warnings
cargo fmt      → clean
```
