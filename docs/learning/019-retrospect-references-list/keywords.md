# 학습 키워드: 회고 참고자료 목록 조회 (API-018)

## 1. `find_retrospect_for_member`

- **정의**: 회고 존재 여부와 팀 멤버십을 동시에 확인하는 서비스 내부 헬퍼 함수
- **소스**: `codes/server/src/domain/retrospect/service.rs:323~352`
- **시그니처**: `async fn find_retrospect_for_member(state: &AppState, user_id: i64, retrospect_id: i64) -> Result<retrospect::Model, AppError>`
- **동작**:
  1. `retrospect::Entity::find_by_id`로 회고 조회 (라인 328~336)
  2. `member_team::Entity::find`로 팀 멤버십 확인 (라인 338~349)
  3. 두 실패 모두 동일한 `AppError::RetrospectNotFound` 반환 (보안 통합 처리)
- **사용 위치**: API-014, API-018(라인 437), API-020, API-021, API-022 서비스에서 호출

---

## 2. RetroReference 엔티티

- **정의**: `retro_reference` 테이블에 대응하는 SeaORM 엔티티 모델
- **소스**: `codes/server/src/domain/retrospect/entity/retro_reference.rs:4~12`
- **테이블명**: `retro_reference`
- **Derive 매크로**: `Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize`
- **필드 구성**:

| 필드 | 타입 | 설명 |
|------|------|------|
| `retro_reference_id` | `i64` | PK, 자동 증가 (`#[sea_orm(primary_key)]`) |
| `title` | `String` | 참고자료 제목 (현재 구현에서는 URL 원본 값이 그대로 저장됨, service.rs:162) |
| `url` | `String` | 참고자료 원본 URL |
| `retrospect_id` | `i64` | FK, 소속 회고 ID |

- **관계**: `retrospect` 엔티티와 N:1 관계 (`belongs_to`, 라인 16~24)
- **주의**: API 명세서에서는 `title`을 "자료 별칭"으로 정의하고 최대 50자 제약을 두었으나, 실제 구현에서는 URL을 그대로 복사하므로 별칭 기능은 미구현 상태

---

## 3. ReferenceItem (DTO)

- **정의**: 참고자료 목록 조회 API의 응답 DTO
- **소스**: `codes/server/src/domain/retrospect/dto.rs:251~260`
- **직렬화**: `#[serde(rename_all = "camelCase")]` 적용
- **필드 매핑**:

| DTO 필드 | JSON 키 | 원본 (엔티티 필드) |
|----------|---------|-------------------|
| `reference_id: i64` | `referenceId` | `retro_reference_id` |
| `url_name: String` | `urlName` | `title` |
| `url: String` | `url` | `url` |

---

## 4. URL 별칭 생성 (title 저장 방식)

- **정의**: 회고 생성 시 참고자료 URL을 저장할 때 `title` 필드에 값을 설정하는 로직
- **소스**: `codes/server/src/domain/retrospect/service.rs:159~172`
- **현재 동작**: URL 문자열을 그대로 `title`에 복사 (`title: Set(url.clone())`, 라인 162)
- **결과**: 조회 시 `urlName`과 `url` 값이 동일하게 반환됨
- **참고**: API 스펙에서는 `urlName`을 "자료 별칭 (예: 깃허브 레포지토리)"로 정의하고 있어, 향후 사용자가 별칭을 직접 입력하는 기능 추가 가능성이 있음

---

## 5. `Entity::find().filter().order_by_asc().all()`

- **정의**: SeaORM에서 조건 필터링과 정렬을 적용하여 전체 결과를 조회하는 쿼리 빌더 체인
- **소스**: `codes/server/src/domain/retrospect/service.rs:440~445`
- **SQL 등가**: `SELECT * FROM retro_reference WHERE retrospect_id = ? ORDER BY retro_reference_id ASC`
- **구성 요소**:
  - `Entity::find()` -- SELECT 시작
  - `.filter(Column::RetrospectId.eq(retrospect_id))` -- WHERE 조건
  - `.order_by_asc(Column::RetroReferenceId)` -- 정렬 기준
  - `.all(&state.db)` -- 전체 결과 반환 (`Vec<Model>`)
- **반환 타입**: `Result<Vec<retro_reference::Model>, DbErr>`

---

## 6. `into_iter().map().collect()`

- **정의**: Rust의 이터레이터 패턴으로 컬렉션의 각 요소를 변환하는 관용적 방법
- **소스**: `codes/server/src/domain/retrospect/service.rs:448~455`
- **동작 과정**:
  1. `into_iter()` -- `Vec<Model>`의 소유권을 가져와 이터레이터 생성
  2. `.map(|r| ReferenceItem { ... })` -- 각 엔티티를 DTO로 변환
  3. `.collect()` -- 이터레이터를 다시 `Vec<ReferenceItem>`으로 수집
- **사용 이유**: 엔티티와 DTO의 필드 이름이 다르므로 수동 매핑 필요 (`retro_reference_id` -> `reference_id`, `title` -> `url_name`)

---

## 7. `BaseResponse::success_with_message`

- **정의**: 모든 API 성공 응답을 표준 형식으로 래핑하는 유틸리티 함수
- **소스**: `codes/server/src/domain/retrospect/handler.rs:199~202`
- **출력 형식**:
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "참고자료 목록을 성공적으로 조회했습니다.",
  "result": [...]
}
```
- **학습 포인트**: 프로젝트의 모든 API가 동일한 응답 구조를 따르도록 공통 유틸리티를 사용

---

## 8. `AppError::RetrospectNotFound`

- **정의**: 회고가 존재하지 않거나 접근 권한이 없을 때 반환하는 에러 타입
- **사용 위치**: `codes/server/src/domain/retrospect/service.rs:333~336, 346~348`
- **에러 코드**: `RETRO4041` (정의: `codes/server/src/utils/error.rs:189`)
- **HTTP 상태 코드**: 404 Not Found (정의: `codes/server/src/utils/error.rs:230`)
- **에러 메시지**: "존재하지 않는 회고이거나 접근 권한이 없습니다."
- **보안 의미**: 비인가 사용자에게 회고의 존재 여부를 노출하지 않기 위해, "존재하지 않음"과 "접근 권한 없음"을 동일한 에러로 통합 처리
- **스펙 차이**: API 명세서에는 접근 권한 없음 시 `403 (TEAM4031)`을 별도로 정의하고 있으나, 구현에서는 이 에러를 사용하지 않고 `RetrospectNotFound`(404)로 통합 처리

---

## 9. `AuthUser`

- **정의**: JWT 토큰에서 추출된 인증 사용자 정보를 담는 Axum 추출자(extractor)
- **소스**: `codes/server/src/domain/retrospect/handler.rs:182` (매개변수로 사용)
- **동작**: Axum의 `FromRequestParts` 트레잇을 구현하여, 요청의 `Authorization` 헤더에서 JWT를 자동 파싱
- **사용법**: `user.user_id()?`로 사용자 ID(`i64`)를 추출 (라인 194)

---

## 10. `#[serde(rename_all = "camelCase")]`

- **정의**: Rust의 `snake_case` 필드명을 JSON의 `camelCase`로 자동 변환하는 serde 어트리뷰트
- **소스**: `codes/server/src/domain/retrospect/dto.rs:252` (`ReferenceItem` 구조체)
- **변환 예시**:
  - `reference_id` -> `referenceId`
  - `url_name` -> `urlName`
- **프로젝트 규칙**: 모든 DTO에 필수 적용 (CLAUDE.md 코딩 규칙 참조)
