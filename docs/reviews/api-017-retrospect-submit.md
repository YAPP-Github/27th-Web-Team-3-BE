# API-017 회고 최종 제출 Implementation Review

## 1. 개요
- **API 명**: `POST /api/v1/retrospects/{retrospectId}/submit`
- **구현 목적**: 작성한 모든 답변(총 5개)을 최종 제출하고 회고 상태를 SUBMITTED로 변경한다.
- **API 스펙**: `docs/api-specs/017-retrospect-submit.md`

## 2. 구현 상세

### 2.1 도메인 구조
`codes/server/src/domain/retrospect/`
- `dto.rs`: `SubmitRetrospectRequest`, `SubmitAnswerItem`, `SubmitRetrospectResponse`, `SuccessSubmitRetrospectResponse`
- `service.rs`: 답변 검증, 회고 존재 확인, 참석자 확인, 상태 확인, 트랜잭션 내 답변 업데이트 및 상태 변경
- `handler.rs`: HTTP 핸들러 (`submit_retrospect`) + utoipa 문서화
- `entity/member_retro.rs`: `RetrospectStatus` enum (DRAFT, SUBMITTED, ANALYZED) 추가, `status`/`submitted_at` 필드 추가

### 2.2 주요 로직
1. **입력 검증**:
   - Path parameter `retrospectId` >= 1 확인
   - DTO 유효성 검증 (`validator`): answers 배열 정확히 5개
2. **비즈니스 검증** (`validate_answers`):
   - questionNumber 1~5 모두 존재하는지 확인 (RETRO4002)
   - 각 답변 공백만 입력 여부 확인 (RETRO4007)
   - 각 답변 1,000자 초과 여부 확인 (RETRO4003)
3. **회고 존재 확인**: retrospect 테이블 조회 (RETRO4041)
4. **참석자 확인**: member_retro 테이블에서 user-retrospect 매핑 확인 (RETRO4041)
5. **중복 제출 방지**: member_retro.status가 SUBMITTED 또는 ANALYZED면 거부 (RETRO4033)
6. **트랜잭션 내 업데이트**:
   - response 테이블의 각 질문별 content 업데이트
   - member_retro의 status를 SUBMITTED, submitted_at을 현재 한국시간으로 업데이트
7. **응답 반환**: retrospectId, submittedAt (YYYY-MM-DD), status: "SUBMITTED"

### 2.3 에러 코드
| Code | HTTP | Description |
|------|------|-------------|
| RETRO4002 | 400 | 답변 누락 (5개 미만 또는 questionNumber 누락) |
| RETRO4003 | 400 | 답변 1,000자 초과 |
| RETRO4007 | 400 | 공백만 입력 |
| AUTH4001 | 401 | 인증 실패 |
| RETRO4033 | 403 | 이미 제출 완료된 회고 |
| RETRO4041 | 404 | 존재하지 않는 회고 |
| COMMON500 | 500 | 서버 내부 오류 |

### 2.4 엔티티 변경
- `member_retro` 엔티티에 `RetrospectStatus` enum과 `status`, `submitted_at` 필드 추가
- `RetrospectStatus`: DRAFT, SUBMITTED, ANALYZED

## 3. 테스트 결과

### 3.1 유닛 테스트 (16개 통과)
**DTO 검증 테스트** (`dto::tests`):
- `should_pass_validation_with_exactly_5_answers`: 정확히 5개 답변 -> 통과
- `should_fail_validation_when_answers_less_than_5`: 4개 답변 -> 실패
- `should_fail_validation_when_answers_more_than_5`: 6개 답변 -> 실패
- `should_fail_validation_when_answers_empty`: 빈 배열 -> 실패

**서비스 검증 테스트** (`service::tests`):
- `should_pass_valid_answers`: 유효한 5개 답변 -> 통과
- `should_fail_when_answers_count_is_not_5`: 4개 답변 -> RETRO4002
- `should_fail_when_question_number_missing`: questionNumber 3 대신 6 -> RETRO4002
- `should_fail_when_duplicate_question_numbers`: 중복 questionNumber -> RETRO4002
- `should_fail_when_content_is_whitespace_only`: 공백만 입력 -> RETRO4007
- `should_fail_when_content_is_empty`: 빈 문자열 -> RETRO4007
- `should_fail_when_content_exceeds_1000_chars`: 1,001자 -> RETRO4003
- `should_pass_when_content_is_exactly_1000_chars`: 정확히 1,000자 -> 통과
- `should_pass_when_content_has_leading_trailing_whitespace`: 앞뒤 공백 있는 유효한 답변 -> 통과
- `should_fail_when_answers_is_empty`: 빈 배열 -> RETRO4002

### 3.2 통합 테스트 (14개 통과)
`tests/retrospect_submit_test.rs`:
- **인증**: 401 (헤더 없음, 잘못된 형식)
- **Path Parameter**: 400 (retrospectId 0, 음수)
- **비즈니스 에러**: 404 (존재하지 않는 회고), 403 (이미 제출 완료)
- **답변 검증**: 400 (답변 부족, 빈 배열, 공백만, 1,000자 초과, 잘못된 JSON, 빈 바디)
- **성공**: 200 (유효한 요청, 최대 길이 답변)

## 4. 코드 리뷰 체크리스트
- [x] TDD 원칙을 따라 테스트 코드가 먼저 작성되었는가?
- [x] 모든 테스트가 통과하는가? (30개: 유닛 16 + 통합 14)
- [x] API 문서가 `docs/reviews/` 디렉토리에 작성되었는가?
- [x] 공통 유틸리티를 재사용했는가? (AppError, BaseResponse, AuthUser)
- [x] 에러 처리가 적절하게 되어 있는가? (Result + ? 패턴)
- [x] 코드가 Rust 컨벤션을 따르는가? (camelCase DTO, snake_case 함수)
- [x] 불필요한 의존성이 추가되지 않았는가?

## 5. 변경 파일 목록
| 파일 | 변경 유형 | 설명 |
|------|----------|------|
| `src/utils/error.rs` | 수정 | 5개 에러 variant 추가 |
| `src/utils/response.rs` | 수정 | `success_with_message` 메서드 추가 |
| `src/domain/retrospect/dto.rs` | 신규 | Request/Response DTO |
| `src/domain/retrospect/service.rs` | 신규 | 비즈니스 로직 및 유닛 테스트 |
| `src/domain/retrospect/handler.rs` | 신규 | HTTP 핸들러 |
| `src/domain/retrospect/mod.rs` | 수정 | 모듈 등록 |
| `src/domain/member/entity/member_retro.rs` | 수정 | RetrospectStatus enum, status/submitted_at 필드 |
| `src/main.rs` | 수정 | 라우트 등록 및 Swagger 스키마 |
| `Cargo.toml` | 수정 | tower util 기능, http-body-util 의존성 추가 |
| `tests/retrospect_submit_test.rs` | 신규 | 통합 테스트 14개 |
