# [API-011] 회고 생성 API 구현 리뷰

## 구현 일자
2026-01-25

## API 개요
- **엔드포인트**: `POST /api/v1/retrospects`
- **기능**: 프로젝트 회고 세션 생성
- **인증**: Bearer 토큰 필요

## 구현 사항

### 1. 생성/수정된 파일

| 파일 | 변경 유형 | 설명 |
|------|----------|------|
| `src/domain/team/entity/team.rs` | 생성 | Team 엔티티 정의 |
| `src/domain/team/entity/member_team.rs` | 생성 | Member-Team 조인 테이블 엔티티 |
| `src/domain/team/entity/mod.rs` | 생성 | Team 엔티티 모듈 |
| `src/domain/team/mod.rs` | 생성 | Team 도메인 모듈 |
| `src/domain/retrospect/entity/retrospect.rs` | 수정 | RetrospectMethod enum 확장, team_id 추가 |
| `src/domain/retrospect/dto.rs` | 생성 | Request/Response DTO |
| `src/domain/retrospect/service.rs` | 생성 | 비즈니스 로직 + 단위 테스트 |
| `src/domain/retrospect/handler.rs` | 생성 | HTTP 핸들러 |
| `src/domain/retrospect/mod.rs` | 수정 | 모듈 추가 |
| `src/utils/error.rs` | 수정 | 에러 코드 추가 |
| `src/config/database.rs` | 수정 | Team, MemberTeam 테이블 생성 추가 |
| `src/domain/mod.rs` | 수정 | Team 도메인 추가 |
| `src/main.rs` | 수정 | 라우터, OpenAPI 문서 추가 |

### 2. RetrospectMethod Enum

기존 `RetroCategory`를 `RetrospectMethod`로 확장:

| Value | 설명 |
|-------|------|
| KPT | Keep-Problem-Try 방식 |
| FOUR_L | 4L (Liked-Learned-Lacked-Longed for) |
| FIVE_F | 5F (Facts-Feelings-Findings-Future-Feedback) |
| PMI | Plus-Minus-Interesting |
| FREE | 자유 형식 |

### 3. 에러 코드 추가

| 코드 | HTTP | 설명 |
|------|------|------|
| RETRO4001 | 400 | 프로젝트 이름 길이 유효성 검사 실패 |
| RETRO4005 | 400 | 유효하지 않은 회고 방식 |
| RETRO4006 | 400 | 유효하지 않은 URL 형식 |
| TEAM4031 | 403 | 팀 접근 권한 없음 |
| TEAM4041 | 404 | 존재하지 않는 팀 |

### 4. 회고 방식별 기본 질문

회고 생성 시 `RetrospectMethod`에 따라 5개의 기본 질문이 자동 생성됩니다.

```rust
impl RetrospectMethod {
    pub fn default_questions(&self) -> Vec<&'static str> { ... }
}
```

### 5. 검증 규칙

- **teamId**: 1 이상의 양수
- **projectName**: 1자 이상 20자 이하
- **retrospectDate**: YYYY-MM-DD 형식, 미래 날짜만 허용
- **referenceUrls**: 최대 10개, http/https 스키마, 최대 2048자, 중복 불가

## 테스트

### 단위 테스트 (19개)

#### URL 검증 테스트
- `should_pass_valid_https_url`
- `should_pass_valid_http_url`
- `should_pass_multiple_valid_urls`
- `should_pass_empty_urls`
- `should_fail_for_duplicate_urls`
- `should_fail_for_ftp_url`
- `should_fail_for_url_without_scheme`
- `should_fail_for_url_exceeding_max_length`
- `should_fail_for_url_without_host`

#### 날짜 검증 테스트
- `should_pass_valid_future_date`
- `should_fail_for_past_date`
- `should_fail_for_today_date`
- `should_fail_for_invalid_date_format`
- `should_fail_for_invalid_date_string`

#### RetrospectMethod 질문 테스트
- `should_return_5_questions_for_kpt`
- `should_return_5_questions_for_four_l`
- `should_return_5_questions_for_five_f`
- `should_return_5_questions_for_pmi`
- `should_return_5_questions_for_free`

## 코드 품질

- [x] `cargo test` 통과 (21개 테스트)
- [x] `cargo clippy -- -D warnings` 경고 없음
- [x] `cargo fmt` 적용

## API 사용 예시

### 요청
```bash
curl -X POST https://api.example.com/api/v1/retrospects \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {accessToken}" \
  -d '{
    "teamId": 789,
    "projectName": "나만의 회고 플랫폼",
    "retrospectDate": "2026-01-30",
    "retrospectMethod": "KPT",
    "referenceUrls": [
      "https://github.com/example/project"
    ]
  }'
```

### 성공 응답
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공입니다.",
  "result": {
    "retrospectId": 12345,
    "teamId": 789,
    "projectName": "나만의 회고 플랫폼"
  }
}
```

## 추가 작업 필요

1. **RetroRoom 연결**: 현재 `retrospect_room_id`가 0으로 고정되어 있음. RetroRoom 생성 로직 연결 필요
2. **통합 테스트**: 실제 DB를 사용한 E2E 테스트 추가 권장

## 참고
- API 스펙: `docs/api-specs/011-retrospect-create.md`
- 아키텍처 가이드: `docs/ai-conventions/architecture.md`
