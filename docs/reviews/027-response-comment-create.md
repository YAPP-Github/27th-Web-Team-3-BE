# API-027 회고 답변 댓글 작성 Implementation Review

## 1. 개요
- **API 명**: `POST /api/v1/responses/{responseId}/comments`
- **구현 목적**: 동료의 회고 답변에 댓글(의견)을 남깁니다.
- **구현 일자**: 2026-01-27
- **브랜치**: feat/response-comments
- **API 스펙**: `docs/api-specs/027-response-comment-create.md`

## 2. 구현 상세

### 2.1 도메인 구조
`codes/server/src/domain/retrospect/`
- `dto.rs`: `CreateCommentRequest`, `CreateCommentResponse`, `SuccessCreateCommentResponse`
- `service.rs`: `create_comment` 메서드 - 댓글 작성 비즈니스 로직
- `handler.rs`: `create_comment` - HTTP 핸들러 + utoipa 문서화

`codes/server/src/utils/error.rs`
- `ResponseNotFound`: 존재하지 않는 회고 답변 (RES4041, 404 Not Found)
- `CommentTooLong`: 댓글 길이 초과 (RES4001, 400 Bad Request)

### 2.2 주요 로직

#### 2.2.1 검증 단계
1. **Path Parameter 검증**:
   - `responseId >= 1` 확인 (핸들러 레벨)
   - 0 이하인 경우 `COMMON400` 반환

2. **Request Body 검증**:
   - `content` 필드 필수 (validator 사용)
   - 빈 문자열 불허 (min = 1)
   - 누락/빈 값인 경우 `COMMON400` 반환

3. **댓글 길이 검증**:
   - `content.chars().count() > 200` 체크 (서비스 레벨)
   - 200자 초과 시 `RES4001` 반환
   - UTF-8 멀티바이트 문자 정확한 카운트

4. **인증 확인**:
   - `AuthUser` extractor를 통한 JWT 토큰 검증
   - 유효하지 않으면 `AUTH4001` 반환

5. **답변 존재 확인**:
   - `response::Entity::find_by_id()` 조회
   - 없으면 `RES4041` 반환

6. **팀 멤버십 확인**:
   - response → retrospect → team 경로로 팀 정보 조회
   - `member_team` 테이블에서 사용자의 팀 소속 여부 확인
   - 권한 없으면 `RETRO4031` 반환

#### 2.2.2 댓글 생성
7. **댓글 데이터 삽입**:
   - `response_comment` 테이블에 새 레코드 삽입
   - `content`: 요청에서 받은 댓글 내용
   - `member_id`: JWT에서 추출한 사용자 ID
   - `response_id`: Path Parameter
   - `created_at`, `updated_at`: 현재 UTC 시간

#### 2.2.3 응답 구성
8. **응답 반환**:
   - `commentId`: 생성된 댓글의 고유 ID
   - `responseId`: 부모 답변의 ID
   - `content`: 서버가 저장한 댓글 내용
   - `createdAt`: 작성 일시 (yyyy-MM-ddTHH:mm:ss 형식)

### 2.3 에러 코드
| Code | HTTP | Description | 발생 조건 |
|------|------|-------------|----------|
| AUTH4001 | 401 | 인증 정보가 유효하지 않음 | Authorization 헤더 없음, 토큰 만료 |
| COMMON400 | 400 | 잘못된 요청 | responseId가 0 이하, content 누락/빈 값 |
| RES4001 | 400 | 댓글 길이 초과 | content가 200자 초과 |
| RETRO4031 | 403 | 권한 없음 | 팀 멤버가 아닌 유저가 댓글 작성 시도 |
| RES4041 | 404 | 존재하지 않는 회고 답변 | responseId가 DB에 없음 |
| COMMON500 | 500 | 서버 내부 오류 | DB 에러 등 |

### 2.4 데이터베이스 변경
- **response_comment 테이블**: 새 레코드 삽입
  - `response_comment_id`: 자동 생성 (PK)
  - `content`: 댓글 내용
  - `created_at`: 생성 시간
  - `updated_at`: 수정 시간
  - `response_id`: FK to response
  - `member_id`: FK to member

## 3. 테스트 결과

### 3.1 통합 테스트 시나리오
| 테스트 케이스 | 예상 결과 | 상태 |
|--------------|----------|------|
| 인증 헤더 없음 | 401 AUTH4001 | ✅ |
| 잘못된 인증 헤더 형식 | 401 AUTH4001 | ✅ |
| responseId = 0 | 400 COMMON400 | ✅ |
| responseId < 0 | 400 COMMON400 | ✅ |
| content 필드 누락 | 400 COMMON400 | ✅ |
| content가 빈 문자열 | 400 COMMON400 | ✅ |
| content가 201자 | 400 RES4001 | ✅ |
| content가 정확히 200자 | 200 성공 | ✅ |
| 존재하지 않는 답변 | 404 RES4041 | ✅ |
| 팀 멤버가 아님 | 403 RETRO4031 | ✅ |
| 유효한 요청 | 200 + 댓글 정보 | ✅ |
| 잘못된 JSON 바디 | 400 COMMON400 | ✅ |
| 빈 요청 바디 | 400 COMMON400 | ✅ |

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
| `src/utils/error.rs` | 수정 | CommentTooLong 에러 variant 추가 |
| `src/domain/retrospect/dto.rs` | 수정 | CreateCommentRequest, CreateCommentResponse 추가 |
| `src/domain/retrospect/service.rs` | 수정 | create_comment 메서드 추가 |
| `src/domain/retrospect/handler.rs` | 수정 | create_comment 핸들러 추가 + utoipa 문서화 |
| `src/main.rs` | 수정 | 라우트 등록 및 Swagger 스키마 추가 |
| `tests/response_comment_test.rs` | 신규 | API-026, 027 통합 테스트 |

## 6. 설계 결정 및 Trade-offs

### 6.1 댓글 길이 검증 위치
- **방식**: 서비스 레벨에서 `chars().count()` 사용
- **이유**: UTF-8 멀티바이트 문자(한글 등) 정확한 글자 수 카운트
- **대안**: validator의 length 사용 시 바이트 기준 검증 (부정확)

### 6.2 에러 코드 분리
- **COMMON400**: 필수 필드 누락, responseId 유효성
- **RES4001**: 댓글 길이 초과 (비즈니스 규칙)
- **이유**: 클라이언트가 에러 유형에 따라 다른 UI 처리 가능

### 6.3 팀 멤버십 확인 경로
- **방식**: response → retrospect → team → member_team
- **재사용**: `find_response_for_member` 헬퍼 (API-026과 공유)
- **이유**: 코드 중복 제거, 일관된 접근 제어

### 6.4 시간 형식
- **방식**: `yyyy-MM-ddTHH:mm:ss` (ISO 8601 without timezone)
- **이유**: API 스펙 준수, 클라이언트 파싱 용이
- **주의**: 서버는 UTC 저장, 클라이언트가 로컬 시간 변환

## 7. 향후 개선 사항

### 7.1 댓글 수정/삭제 API (우선순위: 높음)
- PATCH/DELETE `/api/v1/comments/{commentId}` 추가
- 작성자 본인만 수정/삭제 가능

### 7.2 알림 연동 (우선순위: 중간)
- 댓글 작성 시 답변 작성자에게 알림 전송
- 웹훅 또는 이벤트 기반 처리

### 7.3 멘션 기능 (우선순위: 낮음)
- `@닉네임` 형식으로 특정 사용자 언급
- 언급된 사용자에게 알림 전송

### 7.4 댓글 대댓글 (우선순위: 낮음)
- 중첩 댓글 구조 지원
- `parent_comment_id` 필드 추가

## 8. 관련 문서
- **API 스펙**: `docs/api-specs/027-response-comment-create.md`
- **아키텍처**: `docs/ai-conventions/architecture.md`
- **코딩 규칙**: `docs/ai-conventions/claude.md`
- **관련 API**: `docs/reviews/026-response-comments-list.md`
