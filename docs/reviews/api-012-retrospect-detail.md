# API-012: 회고 상세 정보 조회 API 구현 리뷰

## 개요

| 항목 | 내용 |
|------|------|
| **API** | GET /api/v1/retrospects/{retrospectId} |
| **브랜치** | feature/api-019-retrospect-storage |
| **명세서** | docs/api-specs/012-retrospect-detail.md |

## 구현 내용

### 엔드포인트
- **Method**: GET
- **Path**: `/api/v1/retrospects/{retrospectId}`
- **인증**: Bearer Token 필수
- **Path Parameter**: `retrospectId` (i64, 1 이상의 양수)

### 응답 구조
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "회고 상세 정보 조회를 성공했습니다.",
  "result": {
    "retroRoomId": 789,
    "title": "3차 스프린트 회고",
    "startTime": "2026-01-24",
    "retroCategory": "KPT",
    "members": [
      { "memberId": 1, "userName": "김민철" },
      { "memberId": 2, "userName": "카이" }
    ],
    "totalLikeCount": 156,
    "totalCommentCount": 42,
    "questions": [
      { "index": 1, "content": "계속 유지하고 싶은 좋은 점은 무엇인가요?" },
      { "index": 2, "content": "개선이 필요한 문제점은 무엇인가요?" },
      { "index": 3, "content": "다음에 시도해보고 싶은 것은 무엇인가요?" }
    ]
  }
}
```

## 변경 파일

### 수정 파일

| 파일 | 변경 내용 |
|------|----------|
| `src/domain/retrospect/dto.rs` | `RetrospectDetailResponse`, `RetrospectMemberItem`, `RetrospectQuestionItem`, `SuccessRetrospectDetailResponse` DTO 추가 + 단위 테스트 5개 |
| `src/domain/retrospect/service.rs` | `get_retrospect_detail()` 메서드 추가 |
| `src/domain/retrospect/handler.rs` | `get_retrospect_detail` 핸들러 추가 + Swagger/utoipa 문서화 |
| `src/utils/error.rs` | `RetroRoomAccessDenied` 에러 variant 추가 (RETRO4031) |
| `src/main.rs` | 라우트 등록 (`/api/v1/retrospects/:retrospect_id`) + OpenAPI 스키마/경로 추가 |

## 비즈니스 로직

### 서비스 흐름 (`RetrospectService::get_retrospect_detail`)

1. **Path Parameter 검증**: `retrospectId`가 1 이상인지 확인 (핸들러에서 수행, COMMON400)
2. **사용자 ID 추출**: AuthUser에서 JWT의 `sub` 클레임을 파싱하여 사용자 ID 획득
3. **회고 존재 여부 확인**: `retrospect` 테이블에서 해당 ID로 조회 (RETRO4041)
4. **접근 권한 확인**: `member_retro_room` 테이블에서 사용자가 해당 회고의 `retrospect_room`에 소속된 회고방 멤버인지 확인 (RETRO4031)
5. **참여 멤버 조회**: `member_retro` 테이블에서 해당 회고에 등록된 멤버 목록 조회 (등록일 기준 오름차순), `member` 테이블과 조인하여 닉네임 획득
6. **응답(response) 조회**: `response` 테이블에서 해당 회고의 전체 응답 조회 (response_id 오름차순)
7. **질문 리스트 추출**: 응답에서 중복 제거하여 질문 리스트 생성 (HashSet 사용, 최대 5개, index 1부터 순차 부여)
8. **전체 좋아요 수 조회**: `response_like` 테이블에서 해당 회고 응답들의 좋아요 총합 카운트
9. **전체 댓글 수 조회**: `response_comment` 테이블에서 해당 회고 응답들의 댓글 총합 카운트
10. **시작일 포맷**: UTC 저장된 `start_time`을 KST(+9시간) 변환 후 `YYYY-MM-DD` 형식으로 반환

### 에러 처리

| 상황 | HTTP | 코드 | 메시지 |
|------|------|------|--------|
| retrospectId가 0 이하 | 400 | COMMON400 | retrospectId는 1 이상의 양수여야 합니다. |
| 유효하지 않은 사용자 ID | 401 | COMMON401 | 유효하지 않은 사용자 ID입니다. |
| 인증 실패 | 401 | AUTH4001 | 인증 정보가 유효하지 않습니다. |
| 회고방 접근 권한 없음 | 403 | RETRO4031 | 해당 회고에 접근 권한이 없습니다. |
| 존재하지 않는 회고 | 404 | RETRO4041 | 존재하지 않는 회고입니다. |
| DB 오류 등 서버 에러 | 500 | COMMON500 | 서버 에러, 관리자에게 문의 바랍니다. |

## 테스트 커버리지

### 단위 테스트 (5개) - dto.rs

| 테스트 | 검증 내용 |
|--------|----------|
| `should_serialize_retrospect_detail_response_in_camel_case` | 전체 응답 camelCase 직렬화 검증 (retroRoomId, startTime, retroCategory, totalLikeCount, totalCommentCount, members, questions) |
| `should_serialize_retrospect_detail_with_empty_members_and_questions` | 빈 멤버/질문 리스트 + 통계 0 + FREE 카테고리 직렬화 검증 |
| `should_serialize_all_retro_categories_correctly` | 모든 RetroCategory enum 값 (KPT, FOUR_L, FIVE_F, PMI, FREE) 직렬화 검증 |
| `should_serialize_member_item_in_camel_case` | RetrospectMemberItem camelCase 직렬화 (memberId, userName) + snake_case 키 부재 확인 |
| `should_serialize_question_item_in_camel_case` | RetrospectQuestionItem camelCase 직렬화 (index, content) 검증 |

### 통합 테스트 (11개) - `tests/retrospect_detail_test.rs`

| 테스트 | 검증 내용 |
|--------|----------|
| `api012_should_return_401_when_authorization_header_missing` | 인증 헤더 없이 요청 시 401 반환 |
| `api012_should_return_401_when_authorization_header_format_invalid` | 잘못된 Authorization 형식 시 401 반환 |
| `api012_should_return_400_when_retrospect_id_is_zero` | retrospectId가 0일 때 400 반환 |
| `api012_should_return_400_when_retrospect_id_is_negative` | retrospectId가 음수일 때 400 반환 |
| `api012_should_return_404_when_retrospect_not_found` | 존재하지 않는 회고 요청 시 404 반환 |
| `api012_should_return_403_when_user_is_not_team_member` | 회고방 멤버가 아닌 사용자 요청 시 403 반환 |
| `api012_should_return_200_when_valid_request` | 유효한 요청 시 200 성공 응답 |
| `api012_should_return_correct_result_structure` | 성공 응답의 result 필드 구조 검증 |
| `api012_should_return_correct_members_fields` | 멤버 데이터 (memberId, userName) 검증 |
| `api012_should_return_correct_questions_fields` | 질문 데이터 (index, content) 검증 |
| `api012_should_use_camel_case_field_names_in_response` | 응답 전체 camelCase 필드명 검증 |

## 주요 구현 특징

### 접근 권한 검증 방식
- 회고(retrospect)가 속한 `retrospect_room_id`를 통해 `member_retro_room` 테이블에서 사용자의 회고방 소속 여부를 확인합니다.
- `member_retro`(회고 참석자)가 아닌 `member_retro_room`(회고방 멤버)으로 확인하므로, 아직 회고에 참석 등록하지 않은 팀원도 상세 정보를 조회할 수 있습니다.

### 질문 추출 로직
- `response` 테이블의 `question` 필드에서 중복을 제거(`HashSet`)하여 질문 리스트를 구성합니다.
- `enumerate()`를 사용해 1부터 시작하는 `index`를 순차적으로 부여합니다.

### 통계 조회 최적화
- `response_ids`가 비어있을 경우 DB 쿼리를 생략하고 0을 반환합니다.
- `response_like`와 `response_comment`를 각각 `count()` 쿼리로 조회합니다.

### 날짜 변환
- DB에 UTC로 저장된 `start_time`을 KST(+9시간)로 변환 후 `YYYY-MM-DD` 포맷으로 응답합니다.

## 코드 리뷰 체크리스트

- [x] TDD 원칙을 따라 테스트 코드가 먼저 작성되었는가?
- [x] 모든 테스트가 통과하는가?
- [x] API 문서가 `docs/reviews/` 디렉토리에 작성되었는가?
- [x] 공통 유틸리티를 재사용했는가? (BaseResponse, AppError, AuthUser)
- [x] 에러 처리가 적절하게 되어 있는가? (Result + ? 연산자)
- [x] 코드가 Rust 컨벤션을 따르는가? (camelCase DTO, snake_case 함수)
- [x] 불필요한 의존성이 추가되지 않았는가?
- [x] Swagger/OpenAPI 문서화 완료 (utoipa::path, ToSchema)
- [x] `#[serde(rename_all = "camelCase")]` 적용 확인
