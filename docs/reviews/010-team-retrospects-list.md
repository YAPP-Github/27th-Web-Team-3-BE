# API-010 팀 회고 목록 조회 구현 리뷰

## 개요

| 항목 | 내용 |
|------|------|
| API 번호 | API-010 |
| 엔드포인트 | `GET /api/v1/teams/{teamId}/retrospects` |
| 기능 | 특정 팀에 속한 모든 회고 목록 조회 |
| 브랜치 | `feature/api-010-team-retrospects-list` |
| 기반 브랜치 | `feature/api-011-retrospect-create` |

## 파일 구조

```
codes/server/src/
├── domain/
│   └── retrospect/
│       ├── dto.rs           # TeamRetrospectListItem DTO 추가
│       ├── service.rs       # list_team_retrospects 메서드 추가
│       └── handler.rs       # list_team_retrospects 핸들러 추가
├── main.rs                  # 라우터 및 OpenAPI 등록
└── tests/
    └── retrospect_test.rs   # API-010 통합 테스트 7개 추가
```

## 구현 내용

### 1. DTO (`dto.rs`)

```rust
/// 팀 회고 목록 아이템 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TeamRetrospectListItem {
    pub retrospect_id: i64,
    pub project_name: String,
    pub retrospect_method: RetrospectMethod,
    pub retrospect_date: String,  // yyyy-MM-dd
    pub retrospect_time: String,  // HH:mm
}

impl From<RetrospectModel> for TeamRetrospectListItem {
    fn from(model: RetrospectModel) -> Self { ... }
}
```

### 2. 서비스 (`service.rs`)

```rust
/// 팀 회고 목록 조회 (API-010)
pub async fn list_team_retrospects(
    state: AppState,
    user_id: i64,
    team_id: i64,
) -> Result<Vec<TeamRetrospectListItem>, AppError>
```

**비즈니스 로직:**
1. 팀 존재 여부 확인 → `TeamNotFound` (404)
2. 팀 멤버십 확인 → `TeamAccessDenied` (403)
3. 팀에 속한 회고 목록 조회 (최신순 정렬)
4. DTO 변환 후 반환

### 3. 핸들러 (`handler.rs`)

```rust
#[utoipa::path(
    get,
    path = "/api/v1/teams/{team_id}/retrospects",
    params(("team_id" = i64, Path, description = "조회를 원하는 팀의 고유 ID")),
    security(("bearer_auth" = [])),
    responses(...)
)]
pub async fn list_team_retrospects(
    user: AuthUser,
    State(state): State<AppState>,
    Path(team_id): Path<i64>,
) -> Result<Json<BaseResponse<Vec<TeamRetrospectListItem>>>, AppError>
```

## 응답 형식

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "팀 내 전체 회고 목록 조회를 성공했습니다.",
  "result": [
    {
      "retrospectId": 101,
      "projectName": "오늘 진행할 정기 회고",
      "retrospectMethod": "KPT",
      "retrospectDate": "2026-01-24",
      "retrospectTime": "16:00"
    },
    {
      "retrospectId": 100,
      "projectName": "지난 주 프로젝트 회고",
      "retrospectMethod": "PMI",
      "retrospectDate": "2026-01-20",
      "retrospectTime": "10:00"
    }
  ]
}
```

### 빈 결과 (회고가 없는 경우)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "팀 내 전체 회고 목록 조회를 성공했습니다.",
  "result": []
}
```

## 에러 코드

| 코드 | HTTP | 발생 조건 |
|------|------|----------|
| `AUTH4001` | 401 | 인증 헤더 누락 또는 잘못된 토큰 |
| `COMMON400` | 400 | teamId가 1 미만 |
| `TEAM4031` | 403 | 해당 팀의 멤버가 아님 |
| `TEAM4041` | 404 | 존재하지 않는 팀 |
| `COMMON500` | 500 | 서버 내부 오류 |

## 테스트 결과

### 전체 테스트 현황

```
running 50 tests
test result: ok. 50 passed; 0 failed; 0 ignored
```

- **단위 테스트**: 33개
- **통합 테스트**: 17개 (기존 10개 + API-010 7개)

### API-010 통합 테스트 상세

| 테스트 | 검증 내용 | 예상 코드 |
|--------|---------|----------|
| `api010_should_return_401_when_authorization_header_missing` | 인증 헤더 없음 | 401 |
| `api010_should_return_401_when_authorization_header_format_invalid` | Bearer 형식 아님 | 401 |
| `api010_should_return_404_when_team_not_found` | 존재하지 않는 팀 | 404 |
| `api010_should_return_403_when_not_team_member` | 팀 멤버가 아님 | 403 |
| `api010_should_return_200_with_retrospect_list_when_valid_request` | 정상 요청 | 200 |
| `api010_should_return_200_with_empty_array_when_no_retrospects` | 빈 결과 | 200 |
| `api010_should_return_400_when_team_id_is_zero` | teamId가 0 | 400 |

## 코드 품질 검사

- [x] `cargo test` - 50개 테스트 통과
- [x] `cargo clippy -- -D warnings` - 경고 없음
- [x] `cargo fmt --check` - 포맷팅 통과

## API-011과의 관계

API-010은 API-011 (회고 생성) 브랜치를 기반으로 구현되었습니다:

- **공유 엔티티**: Retrospect, Team, MemberTeam
- **공유 에러 코드**: `TEAM4031`, `TEAM4041`
- **검증 로직 재사용**: 팀 존재 여부, 멤버십 확인

## 설계 결정

### 1. 정렬 방식
- **결정**: `start_time` 기준 내림차순 (최신순)
- **이유**: 사용자가 최근 회고를 먼저 확인하는 UX 고려

### 2. 빈 결과 처리
- **결정**: 회고가 없으면 빈 배열 `[]` 반환
- **이유**: 클라이언트에서 null 체크 불필요, 일관된 응답 형식

### 3. From trait 구현
- **결정**: `RetrospectModel` → `TeamRetrospectListItem` 변환에 `From` trait 사용
- **이유**: 타입 안전성 보장, 코드 재사용성 향상

## 참고 문서

- API 스펙: `docs/api-specs/010-team-retrospects-list.md`
- 아키텍처 가이드: `docs/ai-conventions/architecture.md`
- API-011 리뷰: `docs/reviews/011-retrospect-create.md`
