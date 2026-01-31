# 핵심 개념

API-020 구현에서 배울 수 있는 주요 개념들을 정리합니다.

---

## 1. 커서 기반 페이지네이션 (Cursor-based Pagination)

### Offset 방식 vs Cursor 방식

| 구분 | Offset 방식 | Cursor 방식 |
|------|------------|------------|
| 원리 | `OFFSET N LIMIT M` | `WHERE id < cursor LIMIT M` |
| 성능 | 페이지가 깊어질수록 느려짐 (앞의 N개를 스킵해야 함) | 항상 일정한 성능 (인덱스로 바로 접근) |
| 데이터 정합성 | 중간에 데이터가 삽입/삭제되면 중복 또는 누락 발생 | 커서 기준으로 조회하므로 정합성 유지 |
| 적합한 UI | 페이지 번호 기반 네비게이션 | 무한 스크롤, "더 보기" 버튼 |
| 단점 | 대규모 데이터에서 비효율적 | 임의의 페이지로 바로 이동 불가 |

### 이 프로젝트에서의 구현

`response_id`를 커서로 사용하며, 내림차순 정렬(최신순)로 조회합니다.

```rust
// 커서가 있으면 해당 ID보다 작은 레코드만 조회
if let Some(cursor_id) = cursor {
    query = query.filter(response::Column::ResponseId.lt(cursor_id));
}
```

> 소스: `service.rs:1980-1982`

클라이언트는 응답의 `nextCursor` 값을 다음 요청의 `cursor` 파라미터로 전달하여 다음 페이지를 조회합니다.

---

## 2. size + 1 조회 Trick

다음 페이지 존재 여부를 판단하기 위해, 실제 필요한 개수보다 **1개 더 조회**하는 기법입니다.

```rust
// size + 1개 조회하여 다음 페이지 존재 여부 확인
let fetched = query
    .limit(Some((size + 1) as u64))
    .all(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

let has_next = fetched.len() as i64 > size;
let page_responses: Vec<&response::Model> = fetched.iter().take(size as usize).collect();
```

> 소스: `service.rs:1984-1992`

### 동작 원리

- `size = 10`이면 **11개**를 DB에서 조회
- 조회 결과가 11개이면: `has_next = true`, 실제 응답에는 10개만 포함
- 조회 결과가 10개 이하이면: `has_next = false`, 모든 결과를 응답에 포함

### 장점

- 별도의 COUNT 쿼리 없이 다음 페이지 존재 여부를 알 수 있음
- DB 요청을 1회로 줄여 성능 향상

---

## 3. FromStr Trait 구현

`ResponseCategory` enum에 `FromStr` trait을 구현하여 문자열에서 enum으로의 변환을 지원합니다.
이를 통해 `.parse()` 메서드를 사용할 수 있습니다.

```rust
impl std::str::FromStr for ResponseCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ALL" => Ok(ResponseCategory::All),
            "QUESTION_1" => Ok(ResponseCategory::Question1),
            "QUESTION_2" => Ok(ResponseCategory::Question2),
            "QUESTION_3" => Ok(ResponseCategory::Question3),
            "QUESTION_4" => Ok(ResponseCategory::Question4),
            "QUESTION_5" => Ok(ResponseCategory::Question5),
            _ => Err(format!("유효하지 않은 카테고리: {}", s)),
        }
    }
}
```

> 소스: `dto.rs:597-611`

### 왜 FromStr을 구현하는가?

`ResponsesQueryParams`에서 `category`가 `String` 타입으로 정의되어 있습니다.
이는 쿼리 파라미터 역직렬화 시 유효하지 않은 값이라도 핸들러까지 도달하게 하여,
**핸들러에서 직접 에러 코드(RETRO4004)를 지정**할 수 있도록 하기 위함입니다.

```rust
// 핸들러에서 직접 파싱 + 에러 코드 지정
let category: ResponseCategory = params.category.parse().map_err(|_| {
    AppError::RetroCategoryInvalid("유효하지 않은 카테고리 값입니다.".to_string())
})?;
```

> 소스: `handler.rs:583-585`

만약 `category`를 직접 `ResponseCategory` 타입으로 역직렬화했다면,
Axum이 400 에러를 자동 반환하며 커스텀 에러 코드를 설정할 수 없습니다.

---

## 4. ResponseCategory enum과 question_index() 매핑

`ResponseCategory` enum은 카테고리 값을 0-based 질문 인덱스로 변환하는 `question_index()` 메서드를 제공합니다.

```rust
impl ResponseCategory {
    pub fn question_index(&self) -> Option<usize> {
        match self {
            ResponseCategory::All => None,       // 전체 조회
            ResponseCategory::Question1 => Some(0),
            ResponseCategory::Question2 => Some(1),
            ResponseCategory::Question3 => Some(2),
            ResponseCategory::Question4 => Some(3),
            ResponseCategory::Question5 => Some(4),
        }
    }
}
```

> 소스: `dto.rs:583-595`

### 설계 의도

- `None`이 반환되면 **모든 질문의 답변을 통합 조회**
- `Some(idx)`가 반환되면 **해당 인덱스의 질문에 대한 답변만 필터링**
- `Option`을 사용하여 "전체 조회"와 "특정 질문 조회"를 깔끔하게 분기

서비스 레이어에서의 활용:

```rust
let target_response_ids: Vec<i64> = match category.question_index() {
    Some(idx) => { /* 특정 질문 필터링 */ }
    None => { /* 전체 응답 */ }
};
```

> 소스: `service.rs:1923-1945`

---

## 5. Display Trait 구현

`ResponseCategory`에 `fmt::Display` trait을 구현하여 로깅 시 가독성 있는 문자열로 출력합니다.

```rust
impl fmt::Display for ResponseCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResponseCategory::All => write!(f, "ALL"),
            ResponseCategory::Question1 => write!(f, "QUESTION_1"),
            // ...
        }
    }
}
```

> 소스: `dto.rs:570-581`

서비스 레이어의 `tracing` 로그에서 `%category` 포맷 지시자로 사용됩니다:

```rust
info!(
    category = %category,  // Display trait 사용
    "회고 답변 카테고리별 조회 요청"
);
```

> 소스: `service.rs:1855`

`%` 접두사는 tracing에서 `Display` trait을 사용하여 값을 출력하라는 의미입니다.

---

## 6. Option 활용 패턴

이 API에서는 `Option` 타입이 여러 곳에서 활용됩니다.

### unwrap_or - 기본값 제공

```rust
let size = params.size.unwrap_or(10);
```

> 소스: `handler.rs:597`

`Option<i64>`에서 값이 `None`이면 기본값 10을 사용합니다.

### if let Some - 조건부 실행

```rust
if let Some(cursor) = params.cursor {
    if cursor < 1 { /* ... */ }
}
```

> 소스: `handler.rs:588-594`

cursor가 `Some`인 경우에만 검증 로직을 실행합니다.

### and_then - Option 체이닝

```rust
let user_name = member_id
    .and_then(|mid| member_map.get(&mid))
    .and_then(|m| m.nickname.clone())
    .unwrap_or_default();
```

> 소스: `service.rs:2066-2069`

여러 단계의 `Option` 변환을 체이닝하여, 하나라도 `None`이면 최종적으로 빈 문자열을 반환합니다.

---

## 7. contains를 활용한 범위 검증

Rust의 Range 타입이 제공하는 `contains` 메서드를 사용하여 범위 체크를 간결하게 수행합니다.

```rust
if !(1..=100).contains(&size) {
    return Err(AppError::BadRequest(
        "size는 1~100 범위의 정수여야 합니다.".to_string(),
    ));
}
```

> 소스: `handler.rs:598-601`

- `1..=100`은 1부터 100까지를 포함하는 `RangeInclusive<i64>` 타입
- `contains(&size)`로 size가 해당 범위 안에 있는지 확인
- `if size < 1 || size > 100`과 동일하지만 더 읽기 쉬움

---

## 8. 핸들러와 서비스의 책임 분리

| 계층 | 책임 | 예시 |
|------|------|------|
| **핸들러** | HTTP 요청/응답 처리, 입력 검증, 파라미터 파싱 | retrospectId 범위 검증, category 파싱, size 기본값 설정 |
| **서비스** | 비즈니스 로직, DB 조회, 데이터 변환 | 팀 멤버십 확인, 카테고리별 필터링, 페이지네이션, DTO 변환 |

핸들러는 "무엇이 유효한 입력인가"를, 서비스는 "유효한 입력으로 무엇을 해야 하는가"를 담당합니다.
이 분리 덕분에 서비스 로직을 독립적으로 테스트할 수 있습니다.
