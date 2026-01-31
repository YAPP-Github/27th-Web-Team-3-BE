# 주요 용어 (Keywords)

## Rust / Axum 키워드

| 용어 | 설명 | 소스 위치 |
|------|------|-----------|
| **`Query<T>` Extractor** | URL 쿼리 파라미터(`?key=value`)를 구조체 `T`로 자동 매핑하는 Axum 추출기. `#[derive(Deserialize)]` 필수. | `handler.rs:460` |
| **`Option<String>`** | 값이 있을 수도 없을 수도 있는 타입. `keyword`를 Optional로 선언하여 누락 시에도 핸들러에 도달하게 함. | `dto.rs:512` |
| **`unwrap_or("")`** | `Option`이 `None`이면 기본값을 반환. `Some(v)`이면 `v` 반환. | `service.rs:954` |
| **`trim()`** | 문자열 앞뒤 공백(whitespace) 제거. 공백만 입력한 경우를 빈 문자열로 변환. | `service.rs:954` |
| **`chars().count()`** | 유니코드 문자 수 카운팅. `len()`은 바이트 수를 반환하므로 한국어에서는 `chars().count()` 사용 필요 (예: "안녕" = `len()` 6, `chars().count()` 2). | `service.rs:962` |
| **`HashMap<K, V>`** | 키-값 해시 맵. 팀 ID -> 팀 이름 매핑에 사용. `.get(&key)` 조회 시간복잡도 O(1). | `service.rs:1006-1007` |
| **`AuthUser`** | JWT 토큰을 검증하고 클레임을 추출하는 프로젝트 커스텀 Axum Extractor. 핸들러 파라미터로 선언하면 미들웨어 레벨에서 자동 실행. | `handler.rs:458` |
| **`IntoParams`** | utoipa의 derive 매크로. Swagger 문서에 쿼리 파라미터 스키마를 자동 생성. | `dto.rs:506` |
| **`tracing::info!`** | 구조화된 로깅 매크로. 키-값 쌍으로 컨텍스트를 기록하여 로그 분석 도구에서 필터링 가능. | `service.rs:980-984` |

## SeaORM 키워드

| 용어 | 설명 | 소스 위치 |
|------|------|-----------|
| **`contains()`** | 문자열 부분 일치 검색. SQL `LIKE '%value%'`로 변환. 파라미터 바인딩으로 SQL Injection 방지. | `service.rs:1012` |
| **`is_in()`** | 값 목록 포함 여부 검사. SQL `IN (?, ?, ...)`으로 변환. | `service.rs:1011` |
| **`filter()`** | `WHERE` 절 조건 추가. 체이닝으로 다중 조건(AND) 적용. | `service.rs:1011-1012` |
| **`order_by_desc()`** | 내림차순 정렬. 체이닝 순서가 SQL `ORDER BY` 절의 정렬 우선순위. | `service.rs:1013-1014` |

## 에러 처리 키워드

| 용어 | 설명 | 소스 위치 |
|------|------|-----------|
| **`SearchKeywordInvalid`** | 검색어 관련 에러 타입. 에러 코드 `SEARCH4001`, HTTP 400. 누락/공백/100자 초과 시 발생. | `error.rs:111-112` |
| **`map_err()`** | `Result`의 에러 타입을 변환하는 메서드. SeaORM의 `DbErr`을 `AppError`로 변환할 때 사용. | `service.rs:991` |

## 데이터 모델 키워드

| 용어 | 설명 |
|------|------|
| **`member_team`** | 유저(member)와 팀(team)의 N:M 관계를 표현하는 중간 테이블. `member_id`와 `team_id` 외래키 보유. |
| **`title` 컬럼** | `retrospect` 테이블의 프로젝트 이름 컬럼. API 응답의 `projectName` 필드에 매핑. |
| **`start_time` 컬럼** | `retrospect` 테이블의 날짜+시간 datetime 컬럼. API 응답에서 `retrospectDate`와 `retrospectTime`으로 분리하여 포맷팅. |
