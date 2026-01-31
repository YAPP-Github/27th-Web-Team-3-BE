# API-016 회고 답변 임시 저장 API 리뷰

## 개요

| 항목 | 내용 |
|------|------|
| API | `PUT /api/v1/retrospects/{retrospectId}/drafts` |
| 기능 | 진행 중인 회고의 답변을 임시로 저장 |
| 브랜치 | `feature/api-016-retrospect-draft-save` |

## 구현 파일

| 파일 | 역할 |
|------|------|
| `src/domain/retrospect/dto.rs` | Request/Response DTO 정의 |
| `src/domain/retrospect/handler.rs` | HTTP 핸들러 |
| `src/domain/retrospect/service.rs` | 비즈니스 로직 + 검증 |
| `src/domain/retrospect/mod.rs` | 모듈 export |
| `src/utils/error.rs` | 에러 타입 추가 |
| `src/utils/response.rs` | `success_with_message` 추가 |
| `src/main.rs` | 라우트 등록 + Swagger 스키마 |

## 비즈니스 로직

### 요청 검증 (서비스 레이어)

| 검증 규칙 | 에러 코드 | HTTP |
|----------|-----------|------|
| drafts 배열 최소 1개 | COMMON400 | 400 |
| drafts 배열 최대 5개 | COMMON400 | 400 |
| 중복 questionNumber 불가 | COMMON400 | 400 |
| questionNumber 1~5 범위 | COMMON400 | 400 |
| content 최대 1,000자 | RETRO4003 | 400 |

### 접근 제어

| 검증 | 에러 코드 | HTTP |
|------|-----------|------|
| retrospectId 존재 여부 | RETRO4041 | 404 |
| member_retro 참석자 확인 | RETRO4031 | 403 |
| Bearer 토큰 인증 | AUTH4001 | 401 |

### 저장 로직

1. member_response → response 조인으로 사용자의 응답 목록 조회
2. questionNumber → response_id 오름차순 매핑 (인덱스 기반)
3. content가 null이면 빈 문자열로 저장 (기존 내용 삭제)
4. content가 있으면 해당 내용으로 덮어쓰기
5. updated_at 갱신

## 테스트 커버리지

### DTO 테스트 (6개)

- 정상 역직렬화 (camelCase → snake_case)
- null content 역직렬화
- 빈 문자열 content 역직렬화
- content 필드 누락 시 None 처리
- 응답 직렬화 (snake_case → camelCase)
- Swagger 응답 직렬화

### 검증 로직 테스트 (14개)

- 단일/다중/전체 5개 드래프트 정상 통과
- null/빈 문자열/혼합 content 정상 통과
- 1,000자 경계값 정상 통과
- 빈 배열 → 실패
- 6개 초과 → 실패
- 중복 질문 번호 → 실패
- 범위 벗어난 질문 번호 (0, 6, -1) → 실패
- 1,001자 초과 → 실패

## 코드 리뷰 체크리스트

- [x] TDD 원칙을 따라 테스트 코드가 먼저 작성되었는가?
- [x] 모든 테스트가 통과하는가? (22/22)
- [x] API 문서가 `docs/reviews/` 디렉토리에 작성되었는가?
- [x] 공통 유틸리티를 재사용했는가? (BaseResponse, AppError, AuthUser)
- [x] 에러 처리가 적절하게 되어 있는가? (unwrap 없음, Result + ? 사용)
- [x] 코드가 Rust 컨벤션을 따르는가? (cargo fmt, clippy 통과)
- [x] 불필요한 의존성이 추가되지 않았는가?
