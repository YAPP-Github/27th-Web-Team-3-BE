# [API-018] 회고 참고자료 목록 조회 API 구현 리뷰

## 구현 일자
2026-01-25

## API 개요
- **엔드포인트**: `GET /api/v1/retrospects/{retrospectId}/references`
- **기능**: 특정 회고에 등록된 모든 참고자료(URL) 목록 조회
- **인증**: Bearer 토큰 필요

---

## Summary

회고 참고자료 목록 조회 API를 구현했습니다. 회고 생성 시 등록했던 외부 링크들을 확인할 수 있습니다.

### 주요 기능
- 회고 존재 여부 확인
- 팀 멤버십 검증
- 참고자료 목록 조회 (referenceId 오름차순)
- 빈 배열 반환 지원 (참고자료가 없는 경우)

### 테스트 현황
- **단위 테스트**: 41개 통과
- **통합 테스트**: 32개 통과 (기존 25개 + API-018 7개)
- **총 73개 테스트 모두 통과**

---

## 파일 구조

```
codes/server/src/
├── domain/
│   └── retrospect/
│       ├── dto.rs           # ReferenceItem DTO 추가
│       ├── service.rs       # list_references 메서드 추가
│       └── handler.rs       # list_references 핸들러 추가
├── main.rs                  # 라우터 및 OpenAPI 등록
└── tests/
    └── retrospect_test.rs   # API-018 통합 테스트 7개 추가
```

---

## 구현 사항

### 1. 생성/수정된 파일

| 파일 | 변경 유형 | 설명 |
|------|----------|------|
| `src/domain/retrospect/dto.rs` | 수정 | ReferenceItem, SuccessReferencesListResponse DTO 추가 |
| `src/domain/retrospect/service.rs` | 수정 | list_references 메서드 구현 |
| `src/domain/retrospect/handler.rs` | 수정 | list_references 핸들러 추가 |
| `src/main.rs` | 수정 | 라우터 등록, OpenAPI 스키마 추가 |
| `tests/retrospect_test.rs` | 수정 | 통합 테스트 7개 추가 |

### 2. DTO (`dto.rs`)

```rust
/// 참고자료 아이템 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceItem {
    pub reference_id: i64,  // 자료 고유 식별자 (retro_refrence.retro_refrence_id)
    pub url_name: String,   // 자료 별칭 (retro_refrence.title)
    pub url: String,        // 참고자료 주소
}

/// Swagger용 참고자료 목록 성공 응답 타입
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessReferencesListResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: Vec<ReferenceItem>,
}
```

### 3. 에러 코드 체계

| 코드 | HTTP | 설명 | 발생 조건 |
|------|------|------|---------|
| `COMMON400` | 400 | 잘못된 요청 | retrospectId가 0 이하의 값 |
| `AUTH4001` | 401 | 인증 정보가 유효하지 않음 | 토큰 누락, 만료 또는 잘못된 Bearer 토큰 |
| `RETRO4041` | 404 | 존재하지 않는 회고이거나 접근 권한 없음 | 회고 미존재 또는 비멤버 (동일 응답으로 존재 여부 노출 방지) |
| `COMMON500` | 500 | 서버 내부 에러 | DB 연결 실패, 쿼리 오류 등 |

### 4. 서비스 로직 (`service.rs`)

```rust
/// 회고 참고자료 목록 조회 (API-018)
pub async fn list_references(
    state: AppState,
    user_id: i64,
    retrospect_id: i64,
) -> Result<Vec<ReferenceItem>, AppError>
```

**비즈니스 로직 흐름:**
```
1. 회고 존재 여부 확인 → RetrospectNotFound (404)
2. 회고의 team_id로 팀 멤버십 확인 → RetrospectNotFound (404, 동일 메시지로 존재 여부 노출 방지)
3. retro_refrence 테이블에서 참고자료 조회 (referenceId 오름차순)
4. ReferenceItem DTO로 변환하여 반환
```

### 5. 핸들러 (`handler.rs`)

```rust
#[utoipa::path(
    get,
    path = "/api/v1/retrospects/{retrospectId}/references",
    params(
        ("retrospectId" = i64, Path, description = "조회를 원하는 회고의 고유 ID")
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, body = SuccessReferencesListResponse),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    ),
    tag = "Retrospect"
)]
pub async fn list_references(
    user: AuthUser,
    State(state): State<AppState>,
    Path(retrospect_id): Path<i64>,
) -> Result<Json<BaseResponse<Vec<ReferenceItem>>>, AppError>
```

---

## 테스트

### 단위 테스트 (41개)
- 기존 DTO 검증 테스트 유지
- 기존 URL/날짜 검증 테스트 유지

### 통합 테스트 - API-018 (7개)

| 테스트 | 검증 내용 | 예상 상태 코드 |
|--------|---------|--------------|
| `api018_should_return_401_when_authorization_header_missing` | 인증 헤더 없음 | 401 |
| `api018_should_return_400_when_retrospect_id_is_zero` | retrospectId가 0 | 400 |
| `api018_should_return_400_when_retrospect_id_is_negative` | retrospectId가 음수 | 400 |
| `api018_should_return_404_when_retrospect_not_found` | 존재하지 않는 회고 | 404 |
| `api018_should_return_404_when_not_team_member` | 팀 멤버가 아님 (존재 여부 노출 방지) | 404 |
| `api018_should_return_200_with_empty_array_when_no_references` | 참고자료 없음 | 200 (빈 배열) |
| `api018_should_return_200_with_references_list_when_valid_request` | 정상 요청 | 200 |

### 테스트 실행 결과

```
running 73 tests
test domain::retrospect::dto::tests::should_fail_validation_when_project_name_is_empty ... ok
...
test api018_should_return_200_with_references_list_when_valid_request ... ok
test api018_should_return_200_with_empty_array_when_no_references ... ok

test result: ok. 73 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

---

## 코드 품질

- [x] `cargo test` 통과 (73개 테스트)
- [x] `cargo clippy -- -D warnings` 경고 없음
- [x] `cargo fmt --check` 포맷팅 확인

---

## API 사용 예시

### 요청
```bash
curl -X GET http://localhost:8080/api/v1/retrospects/100/references \
  -H "Authorization: Bearer {accessToken}"
```

### 성공 응답 (200 OK)
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "참고자료 목록을 성공적으로 조회했습니다.",
  "result": [
    {
      "referenceId": 1,
      "urlName": "프로젝트 저장소",
      "url": "https://github.com/jayson/my-project"
    },
    {
      "referenceId": 2,
      "urlName": "기획 문서",
      "url": "https://notion.so/doc/123"
    }
  ]
}
```

### 빈 결과 응답 (200 OK)
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "참고자료 목록을 성공적으로 조회했습니다.",
  "result": []
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

#### 존재하지 않는 회고 또는 접근 권한 없음 (404)
```json
{
  "isSuccess": false,
  "code": "RETRO4041",
  "message": "존재하지 않는 회고이거나 접근 권한이 없습니다.",
  "result": null
}
```

---

## 설계 결정 및 Trade-offs

### 1. 필드 매핑
- **DB 필드** → **API 필드**
  - `retro_refrence_id` → `referenceId`
  - `title` → `urlName`
  - `url` → `url`
- **이유**: API 스펙에 명시된 필드명 사용
- **참고**: DB 테이블명 오타(`retro_refrence`)는 그대로 유지 (기존 스키마 호환)

### 2. 정렬 순서
- **결정**: `referenceId` 오름차순 (등록 순서대로)
- **이유**: API 스펙 요구사항
- **구현**: `order_by_asc(retro_reference::Column::RetroRefrenceId)`

### 3. 빈 배열 처리
- **결정**: 참고자료가 없는 경우 빈 배열 `[]` 반환
- **이유**: API 스펙에 명시, 클라이언트 처리 단순화
- **응답 코드**: 200 OK (404가 아님)

### 4. 권한 검사 (IDOR 방지)
- **결정**: 회고 존재 확인과 팀 멤버십 확인 실패 시 동일한 404 응답 반환
- **이유**: 비멤버가 retrospect_id 존재 여부를 추측할 수 없도록 방지 (IDOR 보안)
- **구현**: 회고 미존재와 멤버십 실패 모두 `RetrospectNotFound` (404) 반환

---

## 리뷰 포인트

리뷰어 분들이 다음 부분을 중점적으로 확인해주시면 감사하겠습니다:

1. **필드 매핑** (`dto.rs`)
   - DB 필드와 API 응답 필드 매핑의 적절성
   - `title` → `urlName` 변환의 의미 전달 명확성

2. **권한 검사 로직** (`service.rs`)
   - 회고 존재 확인과 팀 멤버십 검사 순서의 적절성
   - 기존 API들(014, 010)과의 일관성

3. **정렬 순서** (`service.rs`)
   - `order_by_asc`로 오름차순 정렬 구현의 정확성

---

## 사용 테이블

| 테이블 | 용도 |
|--------|------|
| `retrospects` | 회고 존재 확인, team_id 조회 |
| `member_team` | 팀 멤버십 확인 |
| `retro_refrence` | 참고자료 목록 조회 |

---

## 기존 API와의 관계

API-018은 기존 API들과 동일한 도메인(retrospect)에서 작동합니다:

- **API-011 (회고 생성)**: 참고자료 등록 시점
- **API-018 (참고자료 조회)**: 등록된 참고자료 확인
- **공유 엔티티**: Retrospect, MemberTeam, RetroReference
- **공유 에러 코드**: `RETRO4041` (회고 미존재 또는 접근 권한 없음)

---

## 참고 문서
- API 스펙: `docs/api-specs/018-retrospect-references-list.md`
- 아키텍처 가이드: `docs/ai-conventions/architecture.md`
- API-010 리뷰: `docs/reviews/010-team-retrospects-list.md`
- API-011 리뷰: `docs/reviews/011-retrospect-create.md`
- API-014 리뷰: `docs/reviews/014-retrospect-participant-create.md`
