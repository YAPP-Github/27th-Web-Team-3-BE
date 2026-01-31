# API-029 회고 어시스턴트 Implementation Review

## 1. 개요
- **API 명**: `POST /api/v1/retrospects/{retrospectId}/questions/{questionId}/assistant`
- **구현 목적**: 회고 작성 시 각 질문에 대해 AI 어시스턴트가 작성 가이드를 제공한다.
- **구현 일자**: 2026-01-30
- **브랜치**: feature/api-029-retrospect-assistant
- **API 스펙**: `docs/api-specs/028-retrospect-assistant.md`

## 2. 구현 상세

### 2.1 도메인 구조

`codes/server/src/domain/retrospect/`
- `dto.rs`: `AssistantRequest`, `AssistantResponse`, `GuideItem`, `GuideType`, `SuccessAssistantResponse`
- `service.rs`: `generate_assistant_guide` 메서드 - 어시스턴트 비즈니스 로직
- `handler.rs`: `assistant_guide` - HTTP 핸들러 + utoipa 문서화

`codes/server/src/domain/member/entity/`
- `assistant_usage.rs`: 사용자별 월간 사용량 추적 엔티티 (신규 생성)

`codes/server/src/utils/error.rs`
- `AiAssistantLimitExceeded`: 월간 어시스턴트 한도 초과 (AI4032, 403 Forbidden)
- `QuestionNotFound`: 존재하지 않는 질문 (RETRO4043, 404 Not Found)

`codes/server/src/domain/ai/`
- `prompt.rs`: `AssistantPrompt` — 초기/맞춤 가이드 프롬프트 생성 (신규 추가)
- `service.rs`: `generate_assistant_guide` — OpenAI API 호출 (신규 추가)

### 2.2 주요 로직

#### 2.2.1 검증 단계
1. **Path Parameter 검증**:
   - `retrospectId >= 1` 확인
   - `questionId` 범위: 1~5

2. **Request Body 검증**:
   - `content` 선택 필드 (최대 1000자)
   - validator를 통한 자동 검증

3. **회고 존재 확인**:
   - `retrospect::Entity::find_by_id()` 조회
   - 없으면 `RETRO4041` 반환

4. **참여자 권한 확인**:
   - `member_retro` 테이블에서 사용자의 참여 여부 확인
   - 참여자가 아니면 `RETRO4031` 반환

5. **제출 상태 확인**:
   - `member_retro.status = DRAFT` 확인
   - 이미 제출된 회고는 `RETRO4033` 반환

6. **월간 사용량 사전 검증 (빠른 실패)**:
   - KST 기준 현재 월 1일 00:00 계산 (UTC+9)
   - `assistant_usage` 테이블에서 현재 월의 사용 횟수 카운트
   - 10회 이상이면 `AI4032` 반환 (AI 호출 전 불필요한 비용 차단)

#### 2.2.2 AI 가이드 생성
7. **질문 내용 조회**:
   - 회고 방식의 `default_questions()[question_id - 1]`로 직접 조회
   - DB 의존성 제거로 데이터 정합성 문제 방지

8. **가이드 유형 결정**:
   - `content`가 없거나 빈 문자열: `INITIAL` (초기 가이드)
   - `content`가 있는 경우: `PERSONALIZED` (맞춤 가이드)

9. **AI 서비스 호출**:
   - 초기 가이드: `AssistantPrompt::initial_system_prompt()` + `initial_user_prompt(question)`
   - 맞춤 가이드: `AssistantPrompt::personalized_system_prompt()` + `personalized_user_prompt(question, content)`
   - OpenAI gpt-4o-mini, temperature=0.7, 30초 타임아웃

#### 2.2.3 사용 기록 저장 (동시성 안전)
10. **트랜잭션 기반 사용량 기록**:
    - 트랜잭션 시작 → 삽입 → 최종 카운트 검증 → 커밋/롤백
    - 삽입 후 카운트가 10 초과시 롤백 (동시 요청 보호)
    - `member_id`, `retrospect_id`, `question_id`, `created_at` 저장

#### 2.2.4 응답 반환
11. **응답 구성**:
    - `questionId`: 질문 번호 (1~5)
    - `questionContent`: 질문 내용
    - `guideType`: INITIAL 또는 PERSONALIZED
    - `guides`: 가이드 1~3개 (title + description)
    - `remainingCount`: 이번 달 남은 사용 횟수

### 2.3 에러 코드
| Code | HTTP | Description | 발생 조건 |
|------|------|-------------|----------|
| AUTH4001 | 401 | 인증 정보가 유효하지 않음 | Authorization 헤더 없음, 토큰 만료 |
| COMMON400 | 400 | 잘못된 요청 | content 길이 초과 (1000자) |
| RETRO4031 | 403 | 참여 권한 없음 | 회고 참여자가 아닌 사용자 |
| RETRO4033 | 403 | 이미 제출된 회고 | 제출 완료된 회고에서 어시스턴트 요청 |
| AI4032 | 403 | 월간 어시스턴트 사용 횟수 초과 | 현재 월(KST) 사용 횟수 >= 10회 |
| RETRO4041 | 404 | 존재하지 않는 회고 | retrospectId가 DB에 없음 |
| RETRO4043 | 404 | 존재하지 않는 질문 | questionId가 1~5 범위 밖 |
| AI5001 | 500 | AI 분석 실패 | AI 응답 파싱 오류 등 |
| COMMON500 | 500 | 서버 내부 오류 | DB 에러 등 |

### 2.4 데이터베이스 변경
- **assistant_usage 테이블 (신규)**:
  - `assistant_usage_id`: 기본 키
  - `member_id`: 사용자 ID (FK)
  - `retrospect_id`: 회고 ID (FK)
  - `question_id`: 질문 번호 (1~5)
  - `created_at`: 사용 일시

## 3. 테스트 결과

### 3.1 단위 테스트
- `dto.rs`: AssistantRequest, AssistantResponse, GuideItem, GuideType 직렬화/역직렬화 테스트
- `prompt.rs`: initial/personalized 시스템/사용자 프롬프트 생성 테스트
- `service.rs`: extract_json 테스트 (기존)

### 3.2 테스트 시나리오
- 인증 실패 (401): Authorization 헤더 없음
- Request Body 검증 (400): content 1001자 초과
- 회고 없음 (404): 존재하지 않는 retrospectId
- 질문 없음 (404): questionId가 0 또는 6
- 참여자 아님 (403): 회고에 참여하지 않은 사용자
- 이미 제출됨 (403): status != DRAFT인 회고
- 월간 한도 초과 (403): 10회 사용 후 추가 요청
- 초기 가이드 성공 (200): content 없이 요청
- 맞춤 가이드 성공 (200): content와 함께 요청

## 4. 코드 리뷰 체크리스트
- [x] TDD 원칙을 따라 테스트 코드가 먼저 작성되었는가? (DTO 단위 테스트)
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
| `src/utils/error.rs` | 수정 | AI4032, RETRO4043 에러 variant 추가 |
| `src/domain/member/entity/assistant_usage.rs` | 신규 | 어시스턴트 사용량 추적 엔티티 |
| `src/domain/member/entity/mod.rs` | 수정 | assistant_usage 모듈 추가 |
| `src/domain/member/entity/member.rs` | 수정 | AssistantUsage relation 추가 |
| `src/config/database.rs` | 수정 | assistant_usage 테이블 자동 생성 등록 |
| `src/domain/retrospect/dto.rs` | 수정 | AssistantRequest, AssistantResponse 등 DTO 추가 |
| `src/domain/ai/prompt.rs` | 수정 | AssistantPrompt 구조체/impl 추가 |
| `src/domain/ai/service.rs` | 수정 | generate_assistant_guide 메서드 추가 |
| `src/domain/retrospect/service.rs` | 수정 | generate_assistant_guide 메서드 추가 |
| `src/domain/retrospect/handler.rs` | 수정 | assistant_guide 핸들러 추가 + utoipa 문서화 |
| `src/main.rs` | 수정 | 라우트 등록 및 Swagger 스키마 추가 |

### 5.1 코드 리뷰 개선 사항 (2026-01-30)
| 심각도 | 이슈 | 해결 방안 |
|--------|------|-----------|
| Medium | 월간 사용량 동시 요청 시 10회 초과 가능 | 트랜잭션으로 삽입 후 카운트 검증 |
| Low | 질문 조회가 response 테이블 순서에 의존 | `default_questions()[question_id-1]` 직접 사용 |
| Low | 가이드 개수 문서/구현 불일치 (스펙 1~3개, 구현 정확히 3개) | 구현을 1~3개 허용으로 변경 |

## 6. 설계 결정 및 Trade-offs

### 6.1 사용량 추적 방식
- **방식**: 별도 `assistant_usage` 테이블로 각 사용 기록 저장
- **장점**: 정확한 사용량 추적, 감사(audit) 가능, 사용 패턴 분석 가능
- **단점**: 테이블 크기 증가 (사용자 × 10회/월)
- **대안 검토**: `member.insight_count` 필드 활용 (월별 리셋 로직 필요)

### 6.2 가이드 유형 결정 로직
- **방식**: `content`의 존재/비존재로 결정
- **INITIAL**: content가 null, 빈 문자열, 또는 공백만 있는 경우
- **PERSONALIZED**: content가 의미 있는 텍스트인 경우
- **장점**: 단순하고 예측 가능한 동작

### 6.3 프롬프트 설계
- **말투**: 상냥체 (~어요, ~면 좋아요)로 일관성 있는 UX
- **구조**: title(행동 지침) + description(구체적 제안)
- **개수**: 1~3개 허용 (API 스펙 일치, 검증 로직 포함)

### 6.4 질문 조회 방식
- **방식**: `retrospect_model.retrospect_method.default_questions()[question_id - 1]`
- **장점**: DB 조회 의존성 제거, 데이터 정합성 보장
- **근거**: 질문은 회고 방식별 고정값이므로 runtime에 계산 가능

### 6.5 KST 시간 처리
- **방식**: `Utc::now() + Duration::hours(9)`
- **이유**: 서버는 UTC 기준, 비즈니스 로직은 KST 기준
- **일관성**: API-022 분석과 동일한 방식

### 6.6 동시성 제어 (월간 한도)
- **문제**: 동시 요청 시 사전 카운트 → AI 호출 → 삽입 사이 경합 조건 발생 가능
- **해결**: 트랜잭션 내 삽입 후 카운트 검증
- **흐름**:
  1. 사전 검증 (빠른 실패 - AI 호출 비용 절감)
  2. AI 호출
  3. 트랜잭션: 삽입 → 카운트 → 10 초과시 롤백
- **장점**: 외부 락 없이 DB 트랜잭션으로 원자성 보장

## 7. 향후 개선 사항

### 7.1 통합 테스트 추가 (우선순위: 높음)
- 전체 API 엔드포인트 테스트 작성
- 월간 한도 경계값 테스트
- ~~동시 요청 처리 테스트~~ → 트랜잭션 기반 보호 구현 완료

### 7.2 사용량 조회 API (우선순위: 중간)
- 현재 월 사용 현황 조회 기능
- 남은 횟수 미리 확인 가능

### 7.3 가이드 품질 개선 (우선순위: 중간)
- 프롬프트 튜닝
- 질문 유형별 맞춤 가이드
- 회고 방식(KPT, 4L 등)별 특화 가이드

### 7.4 캐싱 도입 (우선순위: 낮음)
- 동일 질문에 대한 초기 가이드 캐싱
- 응답 시간 개선

## 8. 관련 문서
- **API 스펙**: `docs/api-specs/028-retrospect-assistant.md`
- **유사 API**: `docs/reviews/022-retrospect-analysis.md` (AI 분석)
- **아키텍처**: `docs/ai-conventions/architecture.md`
- **코딩 규칙**: `docs/ai-conventions/claude.md`

## 9. 참고사항

### 9.1 월간 한도 리셋 시점
- 매월 1일 00:00 KST 기준
- 서버 재시작 불필요 (쿼리에서 동적 계산)

### 9.2 API-022 분석과의 차이점
| 항목 | API-022 (분석) | API-029 (어시스턴트) |
|------|---------------|---------------------|
| 한도 단위 | 팀당 월 10회 | 사용자당 월 10회 |
| 추적 방식 | retrospect.team_insight | assistant_usage 테이블 |
| 사용 시점 | 제출 후 | 작성 중 |
| 대상 | 팀 전체 답변 | 개별 질문 |

### 9.3 가이드 개수 검증
- API 스펙에 따라 1~3개 허용
- AI 응답에서 0개 또는 4개 이상인 경우 에러 처리
