# 학습 키워드

API-020 구현에서 등장하는 주요 키워드를 정리합니다. 각 키워드별로 소스 코드에서의 사용 위치를 함께 표기합니다.

---

## Rust 언어 키워드

### 1. Cursor-based Pagination (커서 기반 페이지네이션)

- **정의**: ID나 타임스탬프 등 고유한 값을 기준으로 "이 값 이후의 데이터 N개"를 조회하는 페이지네이션 방식
- **사용 위치**: `service.rs:1976-1989`
- **핵심 코드**: `response::Column::ResponseId.lt(cursor_id)` -- 커서보다 작은 ID만 조회
- **학습 포인트**: Offset 방식 대비 대규모 데이터에서의 성능 이점, 무한 스크롤 UI와의 궁합

### 2. FromStr Trait

- **정의**: 문자열(`&str`)에서 특정 타입으로의 변환을 정의하는 Rust 표준 trait
- **사용 위치**: `dto.rs:597-611`
- **핵심 코드**: `impl std::str::FromStr for ResponseCategory`
- **효과**: `.parse::<ResponseCategory>()` 메서드 호출 가능
- **학습 포인트**: 표준 라이브러리 trait을 구현하면 `.parse()` 같은 편리한 메서드를 자동으로 얻을 수 있음

### 3. Display Trait

- **정의**: 타입을 사람이 읽을 수 있는 문자열로 표현하기 위한 trait (`{}` 포맷)
- **사용 위치**: `dto.rs:570-581`
- **핵심 코드**: `impl fmt::Display for ResponseCategory`
- **활용**: tracing 로그에서 `%category`로 사용 (`service.rs:1855`)
- **학습 포인트**: `Display`와 `Debug`의 차이 -- `Display`는 사용자 대상, `Debug`는 개발자 대상

### 4. Option::unwrap_or

- **정의**: `Option<T>`에서 값이 `None`일 때 지정한 기본값을 반환하는 메서드
- **사용 위치**: `handler.rs:597`
- **핵심 코드**: `params.size.unwrap_or(10)` -- size가 없으면 기본값 10
- **학습 포인트**: `unwrap()`은 `None`일 때 패닉하지만, `unwrap_or()`은 안전하게 기본값을 제공

### 5. Option::and_then

- **정의**: `Option<T>`에서 값이 `Some`일 때만 클로저를 실행하고, `None`이면 `None`을 전파하는 메서드
- **사용 위치**: `service.rs:2066-2069`
- **핵심 코드**: `member_id.and_then(|mid| member_map.get(&mid)).and_then(|m| m.nickname.clone())`
- **학습 포인트**: `map`은 `Option<Option<T>>`을 만들지만, `and_then`은 `Option<T>`로 평탄화(flatten)

### 6. RangeInclusive::contains

- **정의**: 범위 타입이 특정 값을 포함하는지 확인하는 메서드
- **사용 위치**: `handler.rs:598`
- **핵심 코드**: `(1..=100).contains(&size)` -- 1 이상 100 이하인지 확인
- **학습 포인트**: `1..=100`은 `RangeInclusive<i64>` 타입, `1..100`은 100을 포함하지 않는 `Range<i64>`

### 7. if let Some 패턴

- **정의**: `Option`이 `Some`인 경우에만 내부 값을 바인딩하여 코드를 실행하는 패턴 매칭 문법
- **사용 위치**: `handler.rs:588`, `service.rs:1980`
- **핵심 코드**: `if let Some(cursor) = params.cursor { ... }`
- **학습 포인트**: `match`의 축약형으로, 하나의 패턴만 확인할 때 간결하게 사용

### 8. Iterator::take

- **정의**: 이터레이터에서 최대 N개의 요소만 가져오는 어댑터
- **사용 위치**: `service.rs:1992`
- **핵심 코드**: `fetched.iter().take(size as usize).collect()` -- size+1개 중 size개만 선택
- **학습 포인트**: `size + 1` 조회 후 실제 응답 크기로 잘라내는 페이지네이션 패턴에서 활용

### 9. HashMap

- **정의**: 키-값 쌍을 해시 테이블로 관리하는 컬렉션
- **사용 위치**: `service.rs:1898`, `service.rs:1914`, `service.rs:2012`, `service.rs:2030`, `service.rs:2045`, `service.rs:2059`
- **학습 포인트**: 조회 성능이 O(1)이므로, ID를 키로 사용하여 엔티티 간 관계를 빠르게 매핑할 때 유용

### 10. str::parse

- **정의**: `FromStr` trait이 구현된 타입으로 문자열을 변환하는 메서드
- **사용 위치**: `handler.rs:583`
- **핵심 코드**: `params.category.parse()` -- String을 ResponseCategory로 변환
- **학습 포인트**: `parse()`의 반환 타입은 `Result<T, T::Err>`이므로 `?` 또는 `map_err`로 에러 처리 필요

---

## 웹 개발 키워드

### 11. Query Parameter (쿼리 파라미터)

- **정의**: URL의 `?` 이후에 `key=value` 형태로 전달되는 요청 파라미터
- **사용 위치**: `handler.rs:573`
- **핵심 코드**: `Query(params): Query<ResponsesQueryParams>` -- Axum의 `Query` 추출자로 자동 역직렬화
- **학습 포인트**: Path Parameter는 리소스 식별용, Query Parameter는 필터링/정렬/페이지네이션용

### 12. IntoParams (utoipa)

- **정의**: utoipa 라이브러리에서 쿼리 파라미터를 Swagger 문서에 자동 등록하기 위한 derive 매크로
- **사용 위치**: `dto.rs:614`
- **핵심 코드**: `#[derive(Debug, Deserialize, IntoParams)]`
- **학습 포인트**: `ToSchema`는 Request/Response Body용, `IntoParams`는 Query/Path Parameter용

### 13. serde rename_all = "camelCase"

- **정의**: Rust의 snake_case 필드명을 JSON의 camelCase로 자동 변환하는 serde 어트리뷰트
- **사용 위치**: `dto.rs:615`, `dto.rs:627`, `dto.rs:643`
- **핵심 코드**: `#[serde(rename_all = "camelCase")]`
- **학습 포인트**: 프론트엔드 JavaScript 컨벤션(camelCase)과 백엔드 Rust 컨벤션(snake_case)을 자동으로 맞춤

### 14. BaseResponse 래핑

- **정의**: 모든 API 응답을 통일된 구조(`isSuccess`, `code`, `message`, `result`)로 감싸는 패턴
- **사용 위치**: `handler.rs:618-621`
- **핵심 코드**: `BaseResponse::success_with_message(result, "답변 리스트 조회를 성공했습니다.")`
- **학습 포인트**: 클라이언트가 성공/실패를 일관된 방식으로 처리할 수 있게 하는 API 설계 패턴

### 15. AppError를 통한 에러 코드 분류

- **정의**: 도메인별 에러 코드(RETRO4004, TEAM4031 등)를 Rust enum으로 관리하는 패턴
- **사용 위치**: `handler.rs:584` (`RetroCategoryInvalid`), `handler.rs:577` (`BadRequest`)
- **학습 포인트**: HTTP 상태 코드만으로는 에러 원인을 구분하기 어려우므로, 커스텀 에러 코드로 세분화

---

## 키워드 관계도

```
클라이언트 요청
    |
    +-- [Query Parameter] -- category, cursor, size
    |
    +-- [serde rename_all] -- camelCase <-> snake_case 변환
    |
    +-- [FromStr] -- 문자열 -> ResponseCategory enum 변환
    |       |
    |       +-- [str::parse] -- .parse() 메서드 사용
    |
    +-- [Option] 처리
    |       |
    |       +-- [unwrap_or] -- size 기본값
    |       +-- [if let Some] -- cursor 조건부 검증
    |       +-- [and_then] -- 멤버 닉네임 체이닝
    |
    +-- [Cursor-based Pagination]
    |       |
    |       +-- [Iterator::take] -- size개만 선택
    |       +-- [RangeInclusive::contains] -- size 범위 검증
    |
    +-- [HashMap] -- ID 기반 엔티티 매핑
    |
    +-- [Display Trait] -- 로깅 시 enum 문자열 표현
    |
    +-- [BaseResponse] -- 통일된 응답 구조
    |
    +-- [AppError] -- 도메인별 에러 코드 분류
```
