# API-022 회고 분석 Implementation Review

## 1. 개요
- **API 명**: `POST /api/v1/retrospects/{retrospectId}/analysis`
- **구현 목적**: 회고 세션의 모든 답변을 AI 분석하여 팀 인사이트, 감정 랭킹, 개인 미션을 생성한다.
- **구현 일자**: 2026-01-26
- **브랜치**: feature/api-022-retrospect-analysis
- **API 스펙**: `docs/api-specs/022-retrospect-analysis.md`

## 2. 구현 상세

### 2.1 도메인 구조
`codes/server/src/domain/retrospect/`
- `dto.rs`: `AnalysisResponse`, `EmotionRankItem`, `MissionItem`, `PersonalMissionItem`, `SuccessAnalysisResponse`
- `service.rs`: `analyze_retrospective` 메서드 - AI 분석 비즈니스 로직
- `handler.rs`: `analyze_retrospective_handler` - HTTP 핸들러 + utoipa 문서화

`codes/server/src/utils/error.rs`
- `RetroAlreadyAnalyzed`: 이미 분석 완료된 회고 (RETRO4091, 409 Conflict)
- `AiMonthlyLimitExceeded`: 월간 분석 한도 초과 (AI4031, 403 Forbidden)
- `RetroInsufficientData`: 분석 데이터 부족 (RETRO4221, 422 Unprocessable Entity)
- `AiAnalysisFailed`: AI 분석 실패 (AI5001, 500 Internal Server Error)
- `AiConnectionFailed`: AI 연결 실패 (AI5002, 500)
- `AiServiceUnavailable`: AI 서비스 불가 (AI5031, 503)
- `AiGeneralError`: AI 일반 오류 (AI5003, 500)

`codes/server/src/domain/ai/` (분석 전용 모듈)
- `prompt.rs`: `AnalysisPrompt` — 시스템/사용자 프롬프트 생성
- `service.rs`: `AiService` — OpenAI API 호출, JSON 파싱, 응답 검증

### 2.2 주요 로직

#### 2.2.1 검증 단계 (트랜잭션 전)
1. **Path Parameter 검증**:
   - `retrospectId >= 1` 확인 (핸들러 레벨)

2. **회고 존재 확인**:
   - `retrospect::Entity::find_by_id()` 조회
   - 없으면 `RETRO4041` 반환

3. **재분석 방지 (Idempotency)**:
   - `retrospect.team_insight.is_some()` 체크
   - 이미 분석 완료된 회고는 `RETRO4091` (409 Conflict) 반환

4. **회고방 멤버십 확인**:
   - `member_retro_room` 테이블에서 사용자의 팀 소속 여부 확인
   - 권한 없으면 `RETRO4031` 반환

5. **월간 사용량 확인 (팀당 월 10회 제한)**:
   - KST 기준 현재 월 1일 00:00 계산 (UTC+9)
   - 현재 월에 `team_insight IS NOT NULL AND updated_at >= 이번 달 시작` 기준 카운트
   - 10회 이상이면 `AI4031` 반환

6. **최소 데이터 기준 확인**:
   - **참여자 수**: `member_retro`에서 status가 SUBMITTED 또는 ANALYZED인 멤버 조회
   - **답변 수**: `response` 테이블에서 `content.trim()` 후 빈 문자열 아닌 답변 카운트
   - 참여자 < 1명 또는 답변 < 3개면 `RETRO4221` (422 Unprocessable Entity) 반환

#### 2.2.2 데이터 수집
7. **참여자 정보 조회**:
   - `member` 테이블에서 제출 완료한 멤버의 `nickname` 조회
   - `member_id → nickname` 매핑으로 사용자 이름 구성
   - 다른 API(예: `get_retrospect_detail`)와 일관성 확보, PII(이메일) 노출 방지

#### 2.2.3 AI 분석
8. **프롬프트 생성 및 AI 호출** (`domain/ai/` 모듈):
   - **Input**: `MemberAnswerData[]` — 팀원별 `(userId, userName, answers: [(질문, 답변)])`
   - **프롬프트**: `AnalysisPrompt::system_prompt()` (분석 규칙) + `AnalysisPrompt::user_prompt()` (팀원 답변 데이터)
   - **AI 호출**: `AiService::call_openai()` → OpenAI gpt-4o-mini, 30초 타임아웃
   - **Output**: `AnalysisResponse` JSON 파싱 후 검증
     - `team_insight`: 팀 전체 분석 메시지 (1개, 상냥체)
     - `emotion_rank`: 감정 랭킹 (정확히 3개, 2글자 키워드, count 내림차순)
     - `personal_missions`: 사용자별 개인 미션 (사용자당 정확히 3개, 동사형 타이틀)

#### 2.2.4 결과 저장 (트랜잭션)
9. **트랜잭션 내 업데이트**:
   - `retrospect` 테이블: `team_insight` 필드 + `updated_at` 업데이트
   - `member_retro` 테이블: 각 참여자의 `personal_insight` 및 `status = ANALYZED` 업데이트
   - `personal_insight` 형식: "미션제목: 미션설명\n미션제목: 미션설명\n..."

10. **응답 반환**:
    - `team_insight`: 팀 인사이트
    - `emotion_rank`: 감정 랭킹 배열 (3개)
    - `personal_missions`: 개인 미션 배열 (userId 오름차순 정렬)

### 2.3 에러 코드
| Code | HTTP | Description | 발생 조건 |
|------|------|-------------|----------|
| AUTH4001 | 401 | 인증 정보가 유효하지 않음 | Authorization 헤더 없음, 토큰 만료 |
| RETRO4031 | 403 | 회고방 접근 권한 없음 | 회고방 멤버가 아닌 사용자가 분석 요청 |
| AI4031 | 403 | 월간 분석 가능 횟수 초과 | 현재 월(KST) 팀의 분석 횟수 >= 10회 |
| RETRO4041 | 404 | 존재하지 않는 회고 세션 | retrospectId가 DB에 없음 |
| RETRO4221 | 422 | 분석 데이터 부족 | 참여자 < 1명 또는 답변 < 3개 |
| RETRO4091 | 409 | 이미 분석 완료된 회고 | team_insight가 이미 존재 |
| AI5001 | 500 | AI 분석 실패 | AI 서비스 호출 실패 (현재는 사용되지 않음) |
| COMMON500 | 500 | 서버 내부 오류 | DB 에러 등 |

### 2.4 데이터베이스 변경
- **retrospect 테이블**: `team_insight` 필드 사용 (기존 컬럼), `updated_at` 분석 시점 기록
- **member_retro 테이블**: `personal_insight` 필드 사용, `status = ANALYZED`로 업데이트

## 3. 개선 이력

### 3.1 코드 리뷰 후 개선 사항 (2026-01-26)

| 우선순위 | 항목 | 변경 내용 |
|---------|------|----------|
| High | 월간 한도 카운트 기준 수정 | `CreatedAt` → `UpdatedAt` 기준으로 변경. 분석 실행 시 `updated_at`이 함께 갱신되므로, 분석 시점 기준으로 월간 한도를 카운트 |
| Medium | 재분석 방지 (Idempotency) | `team_insight` 이미 존재 시 `RETRO4091` (409 Conflict) 반환. 에러 코드 추가 |
| Medium | user_name을 nickname으로 변경 | `email.split('@')` 로직을 `m.nickname.clone()`으로 교체. 다른 API와 응답 일관성 확보, PII 노출 방지 |
| Low | 미사용 member_response 조회 제거 | 불필요한 `member_response` DB 조회 및 HashMap 구성 코드 삭제. DB 호출 1건 감소 |
| Low | AuthUser 패턴 일관성 | `AuthUser(claims)` 패턴을 다른 핸들러와 동일한 `user: AuthUser` + `user.0.sub` 패턴으로 통일 |

### 3.2 리뷰 피드백 반영 — RefinePrompt 제거 (2026-01-27)

**리뷰 피드백**: "프롬프트 자체가 분석과 방향성이 안 맞는 것 같습니다" (@catturtle123)

**문제**: `domain/ai/` 모듈에 분석과 무관한 말투 정제(Refine) 코드가 혼재
- RefinePrompt: Input(텍스트 1건 + ToneStyle) → Output(어투 변환 텍스트) — 개별 문장 말투 변환 용도
- AnalysisPrompt: Input(팀원 전체 답변 데이터) → Output(인사이트 + 감정 + 미션 JSON) — 팀 종합 분석 용도

**조치**: 분석 방향성과 맞지 않는 Refine 관련 코드 전체 제거

| 파일 | 제거 내용 |
|------|----------|
| `domain/ai/dto.rs` | **파일 삭제** — ToneStyle, RefineRequest, RefineResponse |
| `domain/ai/handler.rs` | **파일 삭제** — refine_retrospective 핸들러, 별도 AppState |
| `prompt.rs` | RefinePrompt 구조체/impl/테스트 3개 제거 |
| `service.rs` | refine_content, validate_secret_key, secret_key 필드, MockAiService 제거 |
| `app_config.rs` | `secret_key` 필드 및 `SECRET_KEY` 환경변수 로딩 제거 |
| `error.rs` | `InvalidSecretKey` variant 제거 |
| `state.rs` | `#[allow(dead_code)]` 제거 (ai_service가 분석에서 실사용) |

**결과**: `domain/ai/` → `prompt.rs`(AnalysisPrompt) + `service.rs`(분석 전용 AiService) 2파일만 유지

### 3.3 에러코드 개선 (2026-01-27)

| 변경 전 | 변경 후 | 사유 |
|---------|---------|------|
| `RETRO4042` (404 Not Found) | `RETRO4221` (422 Unprocessable Entity) | 회고는 존재하나 데이터 부족 → 422가 의미적으로 정확 |
| `InvalidSecretKey` (AI4011, 401) | **제거** | Refine 전용이었으므로 불필요 |

## 4. 테스트 결과

### 4.1 단위 테스트
- `dto.rs`: 직렬화/역직렬화 테스트 (API-022 DTO 포함)
- `service.rs`: 검증 로직 테스트 (validate_drafts, validate_answers)
- API-022 analyze_retrospective 로직은 DB 의존적이므로 통합 테스트로 커버

### 4.2 통합 테스트 시나리오
- 인증 실패 (401): Authorization 헤더 없음, 잘못된 토큰
- Path Parameter 검증 (400): retrospectId = 0, 음수
- 회고 없음 (404): 존재하지 않는 retrospectId
- 이미 분석 완료 (409): team_insight이 이미 존재하는 회고에 재분석 시도
- 회고방 접근 권한 없음 (403): 다른 팀의 회고 분석 시도
- 월간 한도 초과 (403): 동일 팀의 10회 분석 후 추가 요청
- 데이터 부족 (422): 참여자 0명, 답변 2개 이하
- 분석 성공 (200): 정상 요청

## 5. 코드 리뷰 체크리스트
- [x] TDD 원칙을 따라 테스트 코드가 먼저 작성되었는가? (부분적 - 통합 테스트 추가 예정)
- [x] 모든 테스트가 통과하는가?
- [x] API 문서가 `docs/reviews/` 디렉토리에 작성되었는가?
- [x] 공통 유틸리티를 재사용했는가? (AppError, BaseResponse, AuthUser)
- [x] 에러 처리가 적절하게 되어 있는가? (Result + ? 패턴)
- [x] 코드가 Rust 컨벤션을 따르는가? (camelCase DTO, snake_case 함수)
- [x] 불필요한 의존성이 추가되지 않았는가?
- [x] OpenAPI 문서화가 완료되었는가? (utoipa 적용)

## 6. 변경 파일 목록
| 파일 | 변경 유형 | 설명 |
|------|----------|------|
| `src/utils/error.rs` | 수정 | AI 관련 에러 variant 추가 (RETRO4091, AI4031, RETRO4221, AI5001 등) |
| `src/domain/retrospect/dto.rs` | 수정 | AnalysisResponse, EmotionRankItem, PersonalMissionItem, MissionItem, SuccessAnalysisResponse 추가 |
| `src/domain/retrospect/service.rs` | 수정 | analyze_retrospective 메서드 추가 |
| `src/domain/retrospect/handler.rs` | 수정 | analyze_retrospective_handler 추가 + utoipa 문서화 |
| `src/main.rs` | 수정 | 라우트 등록 및 Swagger 스키마 추가 |
| `src/state.rs` | 수정 | AppState에 ai_service 필드 추가 |
| `src/config/app_config.rs` | 수정 | `OPENAI_API_KEY` 환경변수 추가, `secret_key` 제거 |
| `src/domain/ai/prompt.rs` | 수정 | AnalysisPrompt만 유지 (RefinePrompt 제거) |
| `src/domain/ai/service.rs` | 수정 | 분석 전용 AiService (refine 관련 코드 제거) |
| `src/domain/ai/mod.rs` | 수정 | prompt, service 모듈만 노출 (dto, handler 제거) |
| `src/domain/ai/dto.rs` | **삭제** | ToneStyle, RefineRequest, RefineResponse 등 정제 관련 DTO |
| `src/domain/ai/handler.rs` | **삭제** | refine_retrospective 핸들러 및 별도 AppState |

## 7. 설계 결정 및 Trade-offs

### 7.1 AI 분석 모듈 (domain/ai/)
- **구조**: `prompt.rs`(AnalysisPrompt) + `service.rs`(AiService) — 분석 전용
- **Input**: `MemberAnswerData[]` → `[{ userId, userName, answers: [(질문, 답변)] }]`
- **AI 호출**: OpenAI gpt-4o-mini, temperature=0.7, max_tokens=4000, 30초 타임아웃
- **Output**: `AnalysisResponse` JSON → teamInsight + emotionRank(3개) + personalMissions(사용자당 3개)
- **프롬프트 규칙**: 상냥체(~어요), 2글자 감정 키워드, 동사형 미션 타이틀, count 내림차순
- **응답 검증**: emotionRank 정확히 3개, 사용자별 missions 정확히 3개 검증 후 반환

### 7.2 월간 사용량 추적 방식
- **방식**: `team_insight IS NOT NULL AND updated_at >= 이번달 시작` 카운트
- **장점**: 별도 테이블 없이 기존 컬럼으로 추적 가능, 분석 시점(updated_at) 기준으로 정확한 카운트
- **단점**: team_insight가 NULL로 업데이트되면 카운트 누락 가능 (재분석 방지 로직이 이를 방어)
- **대안**: 별도 `analysis_usage` 테이블 생성 (추후 검토)

### 7.3 재분석 방지
- **방식**: `team_insight IS NOT NULL` 체크 → 409 Conflict 반환
- **장점**: 단순하고 명확한 idempotency 보장
- **제한**: 재분석 기능이 필요한 경우 별도 설계 필요

### 7.4 KST 시간 처리
- **방식**: `Utc::now() + Duration::hours(9)`
- **이유**: 서버는 UTC 기준, 비즈니스 로직은 KST 기준
- **주의**: 일광 절약 시간(DST) 없음 (한국은 KST 고정)

### 7.5 트랜잭션 범위
- **검증 로직**: 트랜잭션 밖에서 수행
- **업데이트**: 트랜잭션 내에서 원자적 처리
- **이유**: 검증 실패 시 롤백 오버헤드 방지, 동시성 제어는 업데이트에만 적용

### 7.6 사용자 이름 추출
- **방식**: `member.nickname` 직접 사용
- **이유**: PII(이메일) 노출 방지, 다른 API(get_retrospect_detail)와 응답 일관성 확보

### 7.7 개인 인사이트 저장 형식
- **형식**: 텍스트 형식 ("미션제목: 미션설명\n...")
- **이유**: 단순 저장, 조회 시 파싱 불필요
- **대안**: JSON 형식 저장 (추후 고려)

## 8. 향후 개선 사항

### 8.1 실제 AI 서비스 연동 (우선순위: 높음)
- OpenAI API 호출 구현
- AI 프롬프트 템플릿 작성 (도메인별 전문 프롬프트)
- 응답 파싱 및 검증 로직
- AI 서비스 실패 시 재시도 로직
- 타임아웃 처리 (긴 응답 대기 시간 고려)

### 8.2 통합 테스트 추가 (우선순위: 높음)
- 전체 API 엔드포인트 테스트 작성
- 월간 한도 경계값 테스트
- 최소 데이터 기준 경계값 테스트
- 동시 요청 처리 테스트 (동시성 제어)

### 8.3 AI 응답 캐싱 (우선순위: 중간)
- 동일 회고에 대한 중복 분석 방지 (현재 RETRO4091로 대응)
- Redis 캐싱 도입 검토

### 8.4 분석 결과 별도 테이블 저장 (우선순위: 중간)
- `analysis_results` 테이블 생성
- 분석 이력 추적 (재분석 기능 지원)
- 감정 랭킹 JSON 저장

### 8.5 비동기 분석 처리 (우선순위: 낮음)
- AI 분석 시간이 긴 경우 비동기 처리 (Job Queue)
- 웹훅/폴링으로 결과 조회
- 진행 상태 표시 (ANALYZING 상태 추가)

## 9. 관련 문서
- **API 스펙**: `docs/api-specs/022-retrospect-analysis.md`
- **아키텍처**: `docs/ai-conventions/architecture.md`
- **코딩 규칙**: `docs/ai-conventions/claude.md`
- **기존 리뷰 참고**: `docs/reviews/017-retrospect-submit.md`

## 10. 참고사항

### 10.1 월간 한도 리셋 시점
- 매월 1일 00:00 KST 기준
- 서버 재시작 불필요 (쿼리에서 동적 계산)
- `updated_at` 기준이므로 분석 실행 시점으로 정확하게 카운트

### 10.2 감정 랭킹 고정 개수
- API 스펙에 따라 정확히 3개 반환
- 감정이 3개 미만인 경우에도 Mock에서 3개 생성 (실제 AI는 패딩 로직 필요)

### 10.3 개인 미션 고정 개수
- 사용자당 정확히 3개 반환
- 실제 AI 구현 시 프롬프트에 명시 필요

### 10.4 데이터 일관성
- 분석 완료 후 `member_retro.status = ANALYZED`로 업데이트
- 이후 재제출 방지 (`submit_retrospect`에서 ANALYZED 상태 체크)
- 재분석 방지 (`team_insight IS NOT NULL` 체크 → 409 Conflict)
