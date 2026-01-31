# 핵심 개념: 회고 내보내기 (PDF)

## 1. `impl IntoResponse` - 바이너리 응답 반환

**소스**: `handler.rs:504`

```rust
pub async fn export_retrospect(
    ...
) -> Result<impl IntoResponse, AppError> {
```

### 왜 `impl IntoResponse`인가?

프로젝트의 다른 핸들러들은 모두 `Result<Json<BaseResponse<T>>, AppError>` 형태로 JSON 응답을 반환한다.
그러나 PDF 내보내기는 JSON이 아닌 **바이너리 데이터**를 반환해야 하므로, Axum의 `IntoResponse` 트레이트를 직접 활용한다.

### `IntoResponse` 트레이트란?

Axum에서 HTTP 응답으로 변환 가능한 모든 타입이 구현하는 트레이트이다.
다음 타입들이 기본적으로 `IntoResponse`를 구현한다:

- `String` -> `text/plain` 응답
- `Json<T>` -> `application/json` 응답
- `Vec<u8>` -> `application/octet-stream` 응답
- `(StatusCode, body)` -> 상태 코드 + 본문
- `(headers, body)` -> 커스텀 헤더 + 본문
- `(StatusCode, headers, body)` -> 상태 코드 + 커스텀 헤더 + 본문

### 이 API에서의 사용

```rust
Ok((headers, pdf_bytes))  // handler.rs:542
```

`([헤더 배열], Vec<u8>)` 튜플은 Axum이 자동으로 HTTP 응답으로 변환한다.
`Vec<u8>` 부분이 응답 본문(PDF 바이너리)이 되고, 헤더 배열이 응답 헤더에 추가된다.

---

## 2. Content-Disposition 헤더 - 파일 다운로드 트리거

**소스**: `handler.rs:533-535`

```rust
(
    header::CONTENT_DISPOSITION,
    format!("attachment; filename=\"{}\"", filename),
),
```

### 역할

`Content-Disposition` 헤더는 브라우저에게 응답 데이터를 어떻게 처리할지 지시한다.

| 값 | 동작 |
|----|------|
| `inline` | 브라우저 내에서 직접 표시 (기본값) |
| `attachment` | 파일 다운로드 다이얼로그를 띄움 |
| `attachment; filename="이름.pdf"` | 지정된 파일명으로 다운로드 |

### 동적 파일명 생성 규칙

```rust
let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();  // handler.rs:523
let filename = format!("retrospect_report_{}_{}.pdf", retrospect_id, timestamp);  // handler.rs:524
```

- 형식: `retrospect_report_{회고ID}_{UTC타임스탬프}.pdf`
- 예시: `retrospect_report_100_20250125_143022.pdf`
- 파일명에 한글을 포함하지 않아 다양한 OS/브라우저에서의 다운로드 호환성을 보장한다.

---

## 3. genpdf 라이브러리 - PDF 프로그래밍 생성

**소스**: `service.rs:4-6` (import), `service.rs:1296-1491` (사용)

```rust
use genpdf::elements::{Break, Paragraph};
use genpdf::style;
use genpdf::Element;
```

### genpdf란?

Rust에서 PDF 문서를 프로그래밍 방식으로 생성하는 라이브러리이다.
외부 바이너리(예: wkhtmltopdf)나 C 바인딩 없이 순수 Rust로 동작한다.

### 핵심 구성 요소

| 구성 요소 | 역할 | 사용 위치 |
|-----------|------|-----------|
| `genpdf::Document` | PDF 문서 객체 | service.rs:1355 |
| `genpdf::fonts::from_files()` | 디렉토리에서 폰트 패밀리 로딩 | service.rs:1309 |
| `genpdf::fonts::FontFamily` | Regular/Bold/Italic/BoldItalic 폰트 세트 | service.rs:1325 |
| `genpdf::elements::Paragraph` | 텍스트 단락 요소 | service.rs:1366 등 |
| `genpdf::elements::Break` | 줄바꿈/여백 요소 | service.rs:1369 등 |
| `genpdf::style::Style` | 글꼴 크기, 굵기 등 스타일 | service.rs:1367 등 |
| `SimplePageDecorator` | 페이지 여백 등 레이아웃 설정 | service.rs:1360-1362 |
| `doc.render(&mut buf)` | 메모리 버퍼에 PDF 바이트 렌더링 | service.rs:1487 |

### 문서 구성 패턴

```rust
// 1. 폰트 + 문서 초기화
let mut doc = genpdf::Document::new(font_family);       // service.rs:1355
doc.set_title("...");                                     // service.rs:1356
doc.set_minimal_conformance();                            // service.rs:1357 (PDF 표준 최소 준수)

// 2. 페이지 레이아웃 설정
let mut decorator = genpdf::SimplePageDecorator::new();   // service.rs:1360
decorator.set_margins(15);                                 // service.rs:1361
doc.set_page_decorator(decorator);                         // service.rs:1362

// 3. 요소 추가 (push)
doc.push(Paragraph::new("텍스트").styled(style));         // service.rs:1366
doc.push(Break::new(0.5));                                 // service.rs:1369

// 4. 바이트 버퍼로 렌더링
let mut buf = Vec::new();                                  // service.rs:1486
doc.render(&mut buf)?;                                     // service.rs:1487
```

> **참고**: `doc.set_minimal_conformance()`는 genpdf가 생성하는 PDF의 호환성을 높이기 위해 최소 표준을 설정합니다. PDF 뷰어 간 호환성 문제를 방지합니다.

---

## 4. 폰트 폴백 전략 - 방어적 프로그래밍

**소스**: `service.rs:1304-1353`

### 2단계 폰트 로딩 전략

```
1차 시도: genpdf::fonts::from_files() 로 전체 폰트 패밀리 로딩
   |
   +-- 성공 -> Regular, Bold, Italic, BoldItalic 모두 사용
   |
   +-- 실패 -> 2차 시도
                |
                +-- Regular 폰트 파일 1개만 읽어서
                    Regular/Bold/Italic/BoldItalic 모두 같은 폰트로 대체
```

### 환경변수 기반 설정

```rust
let font_dir = std::env::var("PDF_FONT_DIR").unwrap_or_else(|_| "./fonts".to_string());
let font_family_name = std::env::var("PDF_FONT_FAMILY").unwrap_or_else(|_| "NanumGothic".to_string());
```

| 환경변수 | 기본값 | 설명 |
|----------|--------|------|
| `PDF_FONT_DIR` | `./fonts` | 폰트 파일이 위치한 디렉토리 |
| `PDF_FONT_FAMILY` | `NanumGothic` | 폰트 패밀리 이름 (파일명 접두사) |

폰트 파일 예상 경로: `./fonts/NanumGothic-Regular.ttf`

---

## 5. Cache-Control 헤더 - 브라우저 캐시 방지

**소스**: `handler.rs:536-539`

```rust
(
    header::CACHE_CONTROL,
    "no-cache, no-store, must-revalidate".to_string(),
),
```

### 왜 캐시를 방지하는가?

PDF 내보내기는 호출 시점의 최신 데이터를 반영해야 한다.
브라우저가 이전에 다운로드한 PDF를 캐시에서 재사용하면 오래된 데이터가 표시될 수 있다.

| 디렉티브 | 의미 |
|----------|------|
| `no-cache` | 캐시 사용 전 서버에 재검증 요청 필수 |
| `no-store` | 응답 데이터를 캐시에 저장하지 않음 |
| `must-revalidate` | 캐시 만료 시 반드시 서버에서 재검증 |

세 가지를 함께 사용하여 가장 엄격한 캐시 방지 정책을 적용한다.

---

## 6. 질문 중복 제거 - HashSet 활용

**소스**: `service.rs:1418-1423`

```rust
let mut seen_questions = HashSet::new();
let unique_questions: Vec<&response::Model> = responses
    .iter()
    .filter(|r| seen_questions.insert(r.question.clone()))
    .collect();
```

### 왜 중복 제거가 필요한가?

`response` 테이블에는 같은 질문에 대한 여러 멤버의 답변이 각각의 행으로 저장된다.
예를 들어 "잘한 점은?" 질문에 3명이 답변하면 3개의 행이 존재한다.
PDF에서는 질문을 한 번만 출력하고 그 아래에 모든 답변을 나열해야 하므로, 먼저 고유한 질문 목록을 추출한다.

### `HashSet::insert` 활용 패턴

`HashSet::insert()`는 값이 새로 삽입되면 `true`, 이미 존재하면 `false`를 반환한다.
이를 `filter`와 결합하면 한 번의 순회로 중복을 제거할 수 있다.

---

## 7. 빈 답변 필터링 - `trim().is_empty()`

**소스**: `service.rs:1435`

```rust
.filter(|r| r.question == question_response.question && !r.content.trim().is_empty())
```

### 왜 빈 답변을 필터링하는가?

임시 저장(API-016)에서는 `content`가 빈 문자열(`""`)일 수 있다.
PDF에 빈 답변을 출력하면 의미가 없으므로, `trim()`으로 공백만 있는 답변도 제외한다.
해당 질문에 유효한 답변이 하나도 없으면 `"(No answers)"`를 표시한다.

---

## 8. 회고 방식 표시명 매핑

**소스**: `service.rs:1284-1293`

```rust
fn retrospect_method_display(method: &retrospect::RetrospectMethod) -> String {
    match method {
        retrospect::RetrospectMethod::Kpt => "KPT".to_string(),
        retrospect::RetrospectMethod::FourL => "4L".to_string(),
        retrospect::RetrospectMethod::FiveF => "5F".to_string(),
        retrospect::RetrospectMethod::Pmi => "PMI".to_string(),
        retrospect::RetrospectMethod::Free => "Free".to_string(),
    }
}
```

PDF 기본 정보 섹션에서 회고 방식을 사람이 읽기 좋은 형태로 변환한다.
enum variant 이름(`FourL`, `FiveF`)과 실제 표시명(`4L`, `5F`)이 다르므로 별도의 변환 함수가 필요하다.

---

## 9. 개인 인사이트에서 멤버 이름 폴백

**소스**: `service.rs:1473-1476`

```rust
let name = member_map
    .get(&mr.member_id)
    .cloned()
    .unwrap_or_else(|| format!("Member #{}", mr.member_id));
```

멤버 정보가 없을 때(삭제된 경우 등) `"Member #123"` 형태의 대체 텍스트를 사용한다.
팀 이름의 `"(알 수 없음)"`과 답변 작성자의 `"Anonymous"`와 함께, 누락 데이터에 대한 방어적 프로그래밍 패턴이 일관되게 적용되어 있다.
