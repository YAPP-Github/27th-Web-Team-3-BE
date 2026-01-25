# [API-011] 회고 생성 API 구현 리뷰

## 구현 일자
2026-01-25

## API 개요
- **엔드포인트**: `POST /api/v1/retrospects`
- **기능**: 프로젝트 회고 세션 생성
- **인증**: Bearer 토큰 필요

---

## Summary

회고록 작성 서비스의 **회고 생성 API**를 구현했습니다. 사용자가 팀 내에서 새로운 회고 세션을 생성하고, 선택한 회고 방식에 따라 기본 질문이 자동 생성됩니다.

### 주요 기능
- 5가지 회고 방식 지원 (KPT, 4L, 5F, PMI, FREE)
- 회고 방식별 기본 질문 5개 자동 생성
- 참고 URL 첨부 기능 (최대 10개)
- 트랜잭션 기반 데이터 생성 (RetroRoom → Retrospect → Response → Reference)
- 팀 멤버십 검증 및 권한 관리

### 테스트 현황
- **단위 테스트**: 33개 통과 (DTO 검증 12개 + Service 검증 17개 + JWT 2개 + 기타 2개)
- **통합 테스트**: 10개 통과 (HTTP 엔드포인트 검증)
- **총 43개 테스트 모두 통과**

---

## 구현 사항

### 1. 생성/수정된 파일

| 파일 | 변경 유형 | 설명 |
|------|----------|------|
| `src/domain/team/entity/team.rs` | 생성 | Team 엔티티 정의 |
| `src/domain/team/entity/member_team.rs` | 생성 | Member-Team 조인 테이블 엔티티 |
| `src/domain/team/entity/mod.rs` | 생성 | Team 엔티티 모듈 |
| `src/domain/team/mod.rs` | 생성 | Team 도메인 모듈 |
| `src/domain/retrospect/entity/retrospect.rs` | 수정 | RetrospectMethod enum 확장, team_id 추가 |
| `src/domain/retrospect/dto.rs` | 생성 | Request/Response DTO (299줄) |
| `src/domain/retrospect/service.rs` | 생성 | 비즈니스 로직 + 단위 테스트 (510줄) |
| `src/domain/retrospect/handler.rs` | 생성 | HTTP 핸들러 (55줄) |
| `src/domain/retrospect/mod.rs` | 수정 | 모듈 추가 |
| `src/utils/error.rs` | 수정 | 에러 코드 추가 (5개 신규) |
| `src/config/database.rs` | 수정 | Team, MemberTeam 테이블 생성 추가 |
| `src/domain/mod.rs` | 수정 | Team 도메인 추가 |
| `src/main.rs` | 수정 | 라우터, OpenAPI 문서 추가 |
| `tests/retrospect_test.rs` | 생성 | HTTP 통합 테스트 (444줄) |

### 2. RetrospectMethod Enum

5가지 회고 방식이 정의되어 있으며, 각 방식에 따라 5개의 기본 질문이 자동 생성됩니다:

| Value | 설명 | 기본 질문 예시 |
|-------|------|--------------|
| `KPT` | Keep-Problem-Try | "유지하고 싶은 점은?", "문제점은?", "시도해볼 점은?" |
| `FOUR_L` | Liked-Learned-Lacked-Longed for | "좋았던 점은?", "배운 점은?", "부족한 점은?" |
| `FIVE_F` | Facts-Feelings-Findings-Future-Feedback | "사실은?", "감정은?", "발견한 점은?" |
| `PMI` | Plus-Minus-Interesting | "긍정적인 점은?", "부정적인 점은?", "흥미로운 점은?" |
| `FREE` | 자유 형식 | "기억에 남는 순간은?", "개선할 점은?" |

### 3. 에러 코드 체계

| 코드 | HTTP | 설명 | 발생 조건 |
|------|------|------|---------|
| `RETRO4001` | 400 | 프로젝트 이름 유효성 검사 실패 | 1자 미만 또는 20자 초과 |
| `RETRO4005` | 400 | 유효하지 않은 회고 방식 | Enum 외의 값 입력 |
| `RETRO4006` | 400 | 유효하지 않은 URL 형식 | http/https 아닌 URL |
| `TEAM4031` | 403 | 팀 접근 권한 없음 | 팀 멤버가 아닌 경우 |
| `TEAM4041` | 404 | 존재하지 않는 팀 | 없는 teamId 입력 |

### 4. 검증 규칙

| 필드 | 규칙 | 에러 시 코드 |
|------|------|------------|
| `teamId` | 1 이상의 양수 | COMMON400 |
| `projectName` | 1자 이상 20자 이하 | RETRO4001 |
| `retrospectDate` | YYYY-MM-DD 형식, **미래 날짜만 허용** | COMMON400 |
| `retrospectMethod` | KPT, FOUR_L, FIVE_F, PMI, FREE 중 하나 | RETRO4005 |
| `referenceUrls` | 최대 10개, http/https 스키마, 최대 2048자, 중복 불가 | RETRO4006 |

### 5. 비즈니스 로직 흐름

```
1. 참고 URL 검증 (중복, 형식, 길이)
2. 날짜 형식 및 미래 날짜 검증
3. 팀 존재 여부 확인 → TeamNotFound (404)
4. 팀 멤버십 확인 → TeamAccessDenied (403)
5. 트랜잭션 시작
   ├── 회고방(RetroRoom) 생성 (초대 URL 포함)
   ├── 회고(Retrospect) 생성
   ├── 기본 질문 5개 생성 (Response)
   └── 참고 URL 저장 (RetroReference)
6. 트랜잭션 커밋
7. 응답 반환
```

---

## 테스트

### 단위 테스트 (33개)

#### DTO 검증 테스트 (12개) - `dto.rs`
- `should_fail_validation_when_project_name_is_empty`
- `should_fail_validation_when_project_name_exceeds_20_chars`
- `should_pass_validation_when_project_name_is_exactly_20_chars`
- `should_fail_validation_when_team_id_is_zero`
- `should_fail_validation_when_team_id_is_negative`
- `should_pass_validation_when_team_id_is_positive`
- `should_fail_validation_when_retrospect_date_is_too_short`
- `should_fail_validation_when_retrospect_date_is_too_long`
- `should_pass_validation_when_retrospect_date_has_correct_format`
- `should_fail_validation_when_reference_urls_exceed_10`
- `should_pass_validation_when_reference_urls_are_exactly_10`
- `should_pass_validation_when_reference_urls_are_empty`

#### URL 검증 테스트 (9개) - `service.rs`
- `should_pass_valid_https_url`
- `should_pass_valid_http_url`
- `should_pass_multiple_valid_urls`
- `should_pass_empty_urls`
- `should_fail_for_duplicate_urls`
- `should_fail_for_ftp_url`
- `should_fail_for_url_without_scheme`
- `should_fail_for_url_exceeding_max_length`
- `should_fail_for_url_without_host`

#### 날짜 검증 테스트 (5개) - `service.rs`
- `should_pass_valid_future_date`
- `should_fail_for_past_date`
- `should_fail_for_today_date`
- `should_fail_for_invalid_date_format`
- `should_fail_for_invalid_date_string`

#### RetrospectMethod 질문 테스트 (5개) - `service.rs`
- `should_return_5_questions_for_kpt`
- `should_return_5_questions_for_four_l`
- `should_return_5_questions_for_five_f`
- `should_return_5_questions_for_pmi`
- `should_return_5_questions_for_free`

### 통합 테스트 (10개) - `tests/retrospect_test.rs`

| 테스트 | 검증 내용 | 예상 상태 코드 |
|--------|---------|--------------|
| `should_return_401_when_authorization_header_missing` | 인증 헤더 없음 | 401 |
| `should_return_401_when_authorization_header_format_invalid` | Bearer 형식 아님 | 401 |
| `should_return_400_when_request_body_is_invalid_json` | 잘못된 JSON | 400 |
| `should_return_400_when_required_field_missing` | 필수 필드 누락 | 400 |
| `should_return_400_when_project_name_exceeds_max_length` | 20자 초과 | 400 |
| `should_return_400_when_project_name_is_empty` | 빈 프로젝트 이름 | 400 |
| `should_return_400_when_team_id_is_invalid` | teamId가 0 | 400 |
| `should_return_200_when_request_is_valid` | 정상 요청 | 200 |
| `should_return_400_when_content_type_missing` | Content-Type 없음 | 400 |
| `should_return_400_when_request_body_is_empty` | 빈 요청 바디 | 400 |

### 테스트 실행 결과

```
running 43 tests
test domain::retrospect::dto::tests::should_fail_validation_when_project_name_is_empty ... ok
test domain::retrospect::dto::tests::should_fail_validation_when_project_name_exceeds_20_chars ... ok
test domain::retrospect::dto::tests::should_pass_validation_when_project_name_is_exactly_20_chars ... ok
...
test retrospect_test::should_return_200_when_request_is_valid ... ok
test retrospect_test::should_return_401_when_authorization_header_missing ... ok

test result: ok. 43 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

---

## 코드 품질

- [x] `cargo test` 통과 (43개 테스트)
- [x] `cargo clippy -- -D warnings` 경고 없음
- [x] `cargo fmt --check` 포맷팅 확인

---

## API 사용 예시

### 요청
```bash
curl -X POST http://localhost:8080/api/v1/retrospects \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {accessToken}" \
  -d '{
    "teamId": 789,
    "projectName": "나만의 회고 플랫폼",
    "retrospectDate": "2026-01-30",
    "retrospectMethod": "KPT",
    "referenceUrls": [
      "https://github.com/example/project",
      "https://notion.so/project-docs"
    ]
  }'
```

### 성공 응답 (200 OK)
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "회고가 성공적으로 생성되었습니다.",
  "result": {
    "retrospectId": 12345,
    "teamId": 789,
    "projectName": "나만의 회고 플랫폼"
  }
}
```

### 에러 응답 예시

#### 프로젝트 이름 길이 초과 (400)
```json
{
  "isSuccess": false,
  "code": "RETRO4001",
  "message": "프로젝트 이름은 1자 이상 20자 이하여야 합니다.",
  "result": null
}
```

#### 팀 접근 권한 없음 (403)
```json
{
  "isSuccess": false,
  "code": "TEAM4031",
  "message": "해당 팀의 멤버가 아닙니다.",
  "result": null
}
```

#### 존재하지 않는 팀 (404)
```json
{
  "isSuccess": false,
  "code": "TEAM4041",
  "message": "존재하지 않는 팀입니다.",
  "result": null
}
```

---

## 설계 결정 및 Trade-offs

### 1. 트랜잭션 기반 생성
- **결정**: 모든 관련 엔티티(RetroRoom, Retrospect, Response, RetroReference)를 하나의 트랜잭션으로 처리
- **이유**: 데이터 일관성 보장, 부분 생성 방지
- **Trade-off**: 트랜잭션 시간 증가 가능성, 락 경합 가능성

### 2. 미래 날짜만 허용
- **결정**: `retrospectDate`는 오늘 이후 날짜만 허용
- **이유**: 회고는 미래에 진행할 예정인 세션을 생성하는 기능
- **Trade-off**: 과거 회고 기록을 위한 별도 API 필요할 수 있음

### 3. URL 형식 검증
- **결정**: http/https 프로토콜만 허용, 최대 2048자
- **이유**: 보안(악성 프로토콜 차단) 및 데이터 무결성
- **Trade-off**: 일부 내부 프로토콜(file://, ftp://) 사용 불가

---

## 리뷰 포인트

리뷰어 분들이 다음 부분을 중점적으로 확인해주시면 감사하겠습니다:

1. **에러 처리 플로우** (`utils/error.rs`)
   - AppError → HTTP 응답 변환 로직
   - 에러 코드와 HTTP 상태 코드 매핑의 적절성

2. **RetrospectMethod enum 설계** (`domain/retrospect/entity/retrospect.rs`)
   - 회고 방식별 기본 질문의 적절성
   - 확장 가능성 (새 방식 추가 시)

3. **URL 검증 로직** (`domain/retrospect/service.rs`)
   - 보안 관점에서의 검증 충분성
   - 에지 케이스 처리

4. **트랜잭션 처리** (`domain/retrospect/service.rs`)
   - 롤백 시나리오 대응
   - 동시성 이슈 가능성

---

## 참고
- API 스펙: `docs/api-specs/011-retrospect-create.md`
- 아키텍처 가이드: `docs/ai-conventions/architecture.md`
