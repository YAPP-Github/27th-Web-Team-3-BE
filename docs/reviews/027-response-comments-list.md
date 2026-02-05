# API-026 회고 답변 댓글 목록 조회 Implementation Review

## 1. 개요
- **API 명**: `GET /api/v1/responses/{responseId}/comments`
- **구현 목적**: 특정 회고 답변에 작성된 댓글 리스트를 커서 기반 페이지네이션으로 조회합니다.
- **구현 일자**: 2026-01-27
- **브랜치**: feat/response-comments
- **API 스펙**: `docs/api-specs/026-response-comments-list.md`

## 2. 구현 상세

### 2.1 도메인 구조
`codes/server/src/domain/retrospect/`
- `dto.rs`: `ListCommentsQuery`, `CommentItem`, `ListCommentsResponse`, `SuccessListCommentsResponse`
- `service.rs`: `list_comments` 메서드 - 댓글 목록 조회 비즈니스 로직
- `handler.rs`: `list_comments` - HTTP 핸들러 + utoipa 문서화

`codes/server/src/utils/error.rs`
- `ResponseNotFound`: 존재하지 않는 회고 답변 (RES4041, 404 Not Found)

### 2.2 주요 로직

#### 2.2.1 검증 단계
1. **Path Parameter 검증**:
   - `responseId >= 1` 확인 (핸들러 레벨)
   - 0 이하인 경우 `COMMON400` 반환

2. **Query Parameter 검증**:
   - `cursor`: 있으면 1 이상의 양수 (없으면 첫 페이지)
   - `size`: 1~100 범위 (기본값: 20)
   - 범위 벗어나면 `COMMON400` 반환

3. **인증 확인**:
   - `AuthUser` extractor를 통한 JWT 토큰 검증
   - 유효하지 않으면 `AUTH4001` 반환

4. **답변 존재 확인**:
   - `response::Entity::find_by_id()` 조회
   - 없으면 `RES4041` 반환

5. **회고방 멤버십 확인**:
   - response → retrospect → retro_room 경로로 회고방 정보 조회
   - `member_retro_room` 테이블에서 사용자의 회고방 소속 여부 확인
   - 권한 없으면 `RETRO4031` 반환

#### 2.2.2 데이터 조회
6. **댓글 목록 조회**:
   - `response_comment` 테이블에서 `response_id` 기준 필터링
   - 커서가 있으면 `response_comment_id < cursor` 조건 추가
   - `response_comment_id` 내림차순 정렬 (최신순)
   - `size + 1`개 조회하여 다음 페이지 존재 여부 판단

7. **작성자 정보 조회**:
   - `member` 테이블에서 `member_id` 기준 조회
   - `email`에서 `@` 앞부분을 `userName`으로 추출

#### 2.2.3 응답 구성
8. **페이지네이션 정보 계산**:
   - `hasNext`: 조회된 개수가 `size + 1`이면 `true`
   - `nextCursor`: 다음 페이지 있으면 마지막 댓글의 `commentId`, 없으면 `null`

9. **응답 반환**:
   - `comments`: 댓글 배열 (최대 `size`개)
   - `hasNext`: 다음 페이지 존재 여부
   - `nextCursor`: 다음 조회를 위한 커서

### 2.3 에러 코드
| Code | HTTP | Description | 발생 조건 |
|------|------|-------------|----------|
| AUTH4001 | 401 | 인증 정보가 유효하지 않음 | Authorization 헤더 없음, 토큰 만료 |
| COMMON400 | 400 | 잘못된 요청 | responseId/cursor가 0 이하, size가 1~100 범위 벗어남 |
| RETRO4031 | 403 | 접근 권한 없음 | 회고방 멤버가 아닌 유저가 댓글 조회 시도 |
| RES4041 | 404 | 존재하지 않는 회고 답변 | responseId가 DB에 없음 |
| COMMON500 | 500 | 서버 내부 오류 | DB 에러 등 |

### 2.4 커서 기반 페이지네이션 동작

```
첫 요청: GET /api/v1/responses/456/comments?size=2
응답:
{
  "comments": [
    { "commentId": 789, ... },
    { "commentId": 788, ... }
  ],
  "hasNext": true,
  "nextCursor": 787
}

다음 요청: GET /api/v1/responses/456/comments?cursor=787&size=2
응답:
{
  "comments": [
    { "commentId": 786, ... }
  ],
  "hasNext": false,
  "nextCursor": null
}
```

## 3. 테스트 결과

### 3.1 통합 테스트 시나리오
| 테스트 케이스 | 예상 결과 | 상태 |
|--------------|----------|------|
| 인증 헤더 없음 | 401 AUTH4001 | ✅ |
| 잘못된 인증 헤더 형식 | 401 AUTH4001 | ✅ |
| responseId = 0 | 400 COMMON400 | ✅ |
| responseId < 0 | 400 COMMON400 | ✅ |
| cursor = 0 | 400 COMMON400 | ✅ |
| size = 0 | 400 COMMON400 | ✅ |
| size = 101 | 400 COMMON400 | ✅ |
| 존재하지 않는 답변 | 404 RES4041 | ✅ |
| 회고방 멤버가 아님 | 403 RETRO4031 | ✅ |
| 댓글이 없는 답변 | 200 + 빈 배열 | ✅ |
| 유효한 요청 (첫 페이지) | 200 + 댓글 목록 | ✅ |
| 커서 기반 다음 페이지 | 200 + 다음 페이지 | ✅ |
| size 기본값 적용 | 200 (size=20 적용) | ✅ |

### 3.2 테스트 파일
- `codes/server/tests/response_comment_test.rs`

## 4. 코드 리뷰 체크리스트
- [x] TDD 원칙을 따라 테스트 코드가 먼저 작성되었는가?
- [x] 모든 테스트가 통과하는가?
- [x] API 문서가 `docs/reviews/` 디렉토리에 작성되었는가?
- [x] 공통 유틸리티를 재사용했는가? (AppError, BaseResponse, AuthUser)
- [x] 에러 처리가 적절하게 되어 있는가? (Result + ? 패턴)
- [x] 코드가 Rust 컨벤션을 따르는가? (camelCase DTO, snake_case 함수)
- [x] 불필요한 의존성이 추가되지 않았는가?
- [x] OpenAPI 문서화가 완료되었는가? (utoipa 적용)

## 5. 변경 파일 목록
| 파일 | 변경 유형 | 설명 |
|------|----------|------|
| `src/utils/error.rs` | 수정 | ResponseNotFound 에러 variant 추가 |
| `src/domain/retrospect/dto.rs` | 수정 | ListCommentsQuery, CommentItem, ListCommentsResponse 추가 |
| `src/domain/retrospect/service.rs` | 수정 | list_comments, find_response_for_member 메서드 추가 |
| `src/domain/retrospect/handler.rs` | 수정 | list_comments 핸들러 추가 + utoipa 문서화 |
| `src/main.rs` | 수정 | 라우트 등록 및 Swagger 스키마 추가 |
| `tests/response_comment_test.rs` | 신규 | API-026, 027 통합 테스트 |

## 6. 설계 결정 및 Trade-offs

### 6.1 커서 기반 페이지네이션
- **방식**: `commentId` 내림차순 기준 커서
- **장점**: 오프셋 기반보다 일관된 결과 보장 (새 댓글 추가 시에도)
- **장점**: 대용량 데이터에서도 성능 유지 (OFFSET 없음)
- **단점**: 특정 페이지로 직접 이동 불가

### 6.2 회고방 멤버십 확인 경로
- **방식**: response → retrospect → retro_room → member_retro_room
- **이유**: response 테이블에 직접 retro_room_id가 없어 조인 필요
- **고려사항**: 3단계 조회로 인한 성능 영향 (필요시 캐싱 검토)

### 6.3 사용자 이름 추출
- **방식**: `member.email.split('@')[0]`
- **이유**: member 테이블에 nickname 필드 없음
- **일관성**: 기존 API (API-014 등)와 동일한 방식 적용

### 6.4 size + 1 조회 전략
- **방식**: 요청 size보다 1개 더 조회
- **이유**: 추가 쿼리 없이 다음 페이지 존재 여부 판단
- **장점**: DB 왕복 1회로 페이지네이션 정보 완성

## 7. 향후 개선 사항

### 7.1 성능 최적화 (우선순위: 중간)
- response → retrospect → retro_room 조인 쿼리 최적화
- 자주 조회되는 답변에 대한 캐싱 도입

### 7.2 사용자 프로필 (우선순위: 낮음)
- member 테이블에 nickname 필드 추가
- 프로필 이미지 URL 추가

## 8. 관련 문서
- **API 스펙**: `docs/api-specs/026-response-comments-list.md`
- **아키텍처**: `docs/ai-conventions/architecture.md`
- **코딩 규칙**: `docs/ai-conventions/claude.md`
