# [API-014] 회고 참석자 등록 API 구현 리뷰

## 구현 일자
2026-01-25

## API 개요
- **엔드포인트**: `POST /api/v1/retrospects/{retrospectId}/participants`
- **기능**: 진행 예정인 회고에 참석자로 등록
- **인증**: Bearer 토큰 필요

---

## Summary

회고 참석자 등록 API를 구현했습니다. 사용자가 팀 멤버인 경우, 진행 예정인 회고에 참석자로 등록할 수 있습니다.

### 주요 기능
- 회고 존재 여부 및 팀 멤버십 검증
- 진행 예정인 회고만 참석 가능 (과거/진행중 불가)
- 중복 참석 방지 (애플리케이션 레벨 + DB 제약)
- 닉네임 자동 추출 (이메일 @ 앞부분)

### 테스트 현황
- **단위 테스트**: 41개 통과
- **통합 테스트**: 25개 통과 (기존 17개 + API-014 8개)
- **총 66개 테스트 모두 통과**

---

## 파일 구조

```text
codes/server/src/
├── domain/
│   └── retrospect/
│       ├── dto.rs           # CreateParticipantResponse DTO 추가
│       ├── service.rs       # create_participant 메서드 추가
│       └── handler.rs       # create_participant 핸들러 추가
├── utils/
│   └── error.rs             # RetrospectNotFound, ParticipantDuplicate, RetrospectAlreadyStarted 추가
├── main.rs                  # 라우터 및 OpenAPI 등록
└── tests/
    └── retrospect_test.rs   # API-014 통합 테스트 8개 추가
```

---

## 구현 사항

### 1. 생성/수정된 파일

| 파일 | 변경 유형 | 설명 |
|------|----------|------|
| `src/domain/retrospect/dto.rs` | 수정 | CreateParticipantResponse DTO 추가 |
| `src/domain/retrospect/service.rs` | 수정 | create_participant 메서드 구현 |
| `src/domain/retrospect/handler.rs` | 수정 | create_participant 핸들러 추가 |
| `src/utils/error.rs` | 수정 | 에러 타입 3개 추가 |
| `src/main.rs` | 수정 | 라우터 등록 |
| `tests/retrospect_test.rs` | 수정 | 통합 테스트 8개 추가 |

### 2. DTO (`dto.rs`)

```rust
/// 회고 참석자 등록 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateParticipantResponse {
    pub participant_id: i64,  // 참석자 등록 고유 ID (member_retro.member_retro_id)
    pub member_id: i64,       // 유저 고유 ID
    pub nickname: String,     // 유저 닉네임 (이메일에서 추출)
}
```

### 3. 에러 코드 체계

| 코드 | HTTP | 설명 | 발생 조건 |
|------|------|------|---------|
| `RETRO4041` | 404 | 존재하지 않는 회고 | 없는 retrospectId 입력 |
| `RETRO4002` | 400 | 이미 시작된 회고 | 과거/진행중인 회고에 참석 시도 |
| `RETRO4091` | 409 | 중복 참석 등록 | 이미 참석자로 등록된 경우 |
| `TEAM4031` | 403 | 팀 접근 권한 없음 | 팀 멤버가 아닌 경우 |

### 4. 서비스 로직 (`service.rs`)

```rust
/// 회고 참석자 등록 (API-014)
pub async fn create_participant(
    state: AppState,
    user_id: i64,
    retrospect_id: i64,
) -> Result<CreateParticipantResponse, AppError>
```

**비즈니스 로직 흐름:**
```text
1. 회고 존재 여부 확인 → RetrospectNotFound (404)
2. 회고의 team_id로 팀 멤버십 확인 → TeamAccessDenied (403)
3. 진행 예정인 회고인지 확인 (start_time > now_kst) → RetrospectAlreadyStarted (400)
4. 이미 참석 등록 여부 확인 → ParticipantDuplicate (409)
5. 멤버 정보 조회
6. 닉네임 추출 (이메일 @ 앞부분)
7. member_retro 테이블에 레코드 삽입
   └── DB 유니크 제약 위반 시 ParticipantDuplicate (409)로 매핑
8. 응답 반환
```

### 5. 핸들러 (`handler.rs`)

```rust
#[utoipa::path(
    post,
    path = "/api/v1/retrospects/{retrospect_id}/participants",
    params(
        ("retrospect_id" = i64, Path, description = "참석할 회고의 고유 ID")
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, body = SuccessCreateParticipantResponse),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
        (status = 409, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    ),
    tag = "Retrospect"
)]
pub async fn create_participant(
    user: AuthUser,
    State(state): State<AppState>,
    Path(retrospect_id): Path<i64>,
) -> Result<Json<BaseResponse<CreateParticipantResponse>>, AppError>
```

---

## 테스트

### 단위 테스트 (41개)
- 기존 DTO 검증 테스트 유지
- 기존 URL/날짜 검증 테스트 유지

### 통합 테스트 - API-014 (8개)

| 테스트 | 검증 내용 | 예상 상태 코드 |
|--------|---------|--------------|
| `api014_should_return_401_when_authorization_header_missing` | 인증 헤더 없음 | 401 |
| `api014_should_return_400_when_retrospect_id_is_zero` | retrospectId가 0 | 400 |
| `api014_should_return_400_when_retrospect_id_is_negative` | retrospectId가 음수 | 400 |
| `api014_should_return_404_when_retrospect_not_found` | 존재하지 않는 회고 | 404 |
| `api014_should_return_403_when_not_team_member` | 팀 멤버가 아님 | 403 |
| `api014_should_return_400_when_retrospect_already_started` | 과거/진행중 회고 | 400 |
| `api014_should_return_409_when_already_participant` | 중복 참석 | 409 |
| `api014_should_return_200_when_valid_request` | 정상 요청 | 200 |

### 테스트 실행 결과

```text
running 66 tests
test domain::retrospect::dto::tests::should_fail_validation_when_project_name_is_empty ... ok
...
test api014_should_return_200_when_valid_request ... ok
test api014_should_return_400_when_retrospect_id_is_negative ... ok
test api014_should_return_409_when_already_participant ... ok

test result: ok. 66 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

---

## 코드 품질

- [x] `cargo test` 통과 (66개 테스트)
- [x] `cargo clippy -- -D warnings` 경고 없음
- [x] `cargo fmt --check` 포맷팅 확인

---

## API 사용 예시

### 요청
```bash
curl -X POST http://localhost:8080/api/v1/retrospects/123/participants \
  -H "Authorization: Bearer {accessToken}"
```

### 성공 응답 (200 OK)
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "회고 참석자로 성공적으로 등록되었습니다.",
  "result": {
    "participantId": 5001,
    "memberId": 456,
    "nickname": "user"
  }
}
```

### 에러 응답 예시

#### retrospectId 유효성 오류 (400)
```json
{
  "isSuccess": false,
  "code": "COMMON400",
  "message": "retrospectId는 1 이상의 양수여야 합니다.",
  "result": null
}
```

#### 이미 시작된 회고 (400)
```json
{
  "isSuccess": false,
  "code": "RETRO4002",
  "message": "이미 시작되었거나 종료된 회고에는 참석할 수 없습니다.",
  "result": null
}
```

#### 팀 멤버 아님 (403)
```json
{
  "isSuccess": false,
  "code": "TEAM4031",
  "message": "해당 회고가 속한 팀의 멤버가 아닙니다.",
  "result": null
}
```

#### 존재하지 않는 회고 (404)
```json
{
  "isSuccess": false,
  "code": "RETRO4041",
  "message": "존재하지 않는 회고입니다.",
  "result": null
}
```

#### 중복 참석 (409)
```json
{
  "isSuccess": false,
  "code": "RETRO4091",
  "message": "이미 참석자로 등록되어 있습니다.",
  "result": null
}
```

---

## 설계 결정 및 Trade-offs

### 1. 닉네임 추출 방식
- **결정**: 이메일 주소의 `@` 앞부분을 닉네임으로 사용
- **이유**: 현재 DB 스키마에 nickname 필드가 없음
- **Trade-off**: 향후 스키마에 nickname 필드 추가 시 수정 필요
- **예시**: `user@example.com` → `user`

### 2. 중복 참석 방지 전략
- **결정**: 애플리케이션 레벨 검사 + DB 유니크 제약 활용
- **이유**:
  - 애플리케이션 검사: 빠른 피드백, 명확한 에러 메시지
  - DB 제약: 동시 요청 시 Race Condition 방지
- **Trade-off**: DB 에러 메시지 파싱의 취약성 (문자열 매칭)

### 3. 시간 검증 로직
- **결정**: `start_time <= now_kst` 검사로 과거/현재 시작 회고 차단
- **이유**: 진행 예정인 회고만 참석 등록 허용
- **Trade-off**: 정확히 시작 시간에 요청하면 참석 불가 (경계 조건)

### 4. KST 시간대 처리
- **결정**: `Utc::now() + 9시간`으로 KST 계산
- **이유**: 한국 서비스 대상, DST 없음
- **Trade-off**: 하드코딩된 시간대 오프셋

---

## 리뷰 포인트

리뷰어 분들이 다음 부분을 중점적으로 확인해주시면 감사하겠습니다:

1. **중복 참석 처리 로직** (`service.rs`)
   - 애플리케이션 검사와 DB 제약의 조합이 적절한가?
   - DB 에러 메시지 파싱 방식의 안전성

2. **시간 검증 로직** (`service.rs`)
   - `start_time <= now_kst` 경계 조건 처리의 적절성
   - KST 계산 방식의 정확성

3. **닉네임 추출 로직** (`service.rs`)
   - 이메일 파싱 fallback 처리의 충분성
   - 향후 nickname 필드 추가 시 마이그레이션 계획

4. **에러 코드 매핑** (`error.rs`)
   - HTTP 상태 코드와 비즈니스 에러 코드 매핑의 적절성

---

## 사용 테이블

| 테이블 | 용도 |
|--------|------|
| `retrospects` | 회고 존재 확인, team_id 조회 |
| `member_team` | 팀 멤버십 확인 |
| `member` | 멤버 정보 (이메일) 조회 |
| `member_retro` | 참석자 등록 (INSERT) |

---

## API-010, API-011과의 관계

API-014는 API-010, API-011과 동일한 도메인(retrospect)에서 작동합니다:

- **공유 엔티티**: Retrospect, MemberTeam, Member
- **공유 에러 코드**: `TEAM4031` (팀 접근 권한 없음)
- **검증 로직 재사용**: 팀 멤버십 확인

---

## 참고 문서
- API 스펙: `docs/api-specs/014-retrospect-participant-create.md`
- 아키텍처 가이드: `docs/ai-conventions/architecture.md`
- API-010 리뷰: `docs/reviews/010-team-retrospects-list.md`
- API-011 리뷰: `docs/reviews/011-retrospect-create.md`
