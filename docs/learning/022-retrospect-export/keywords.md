# 학습 키워드: 회고 내보내기 (PDF)

## Axum / HTTP 관련

### `IntoResponse`

- **정의**: Axum에서 HTTP 응답으로 변환 가능한 타입이 구현하는 트레이트
- **위치**: `handler.rs:504` - `Result<impl IntoResponse, AppError>`
- **용도**: JSON이 아닌 바이너리(PDF) 데이터를 응답으로 반환하기 위해 사용
- **참고**: `axum::response::IntoResponse` (`handler.rs:4`)

### `header::CONTENT_TYPE`

- **정의**: HTTP `Content-Type` 헤더 상수
- **위치**: `handler.rs:529`
- **값**: `"application/pdf; charset=utf-8"`
- **용도**: 브라우저에게 응답 본문이 PDF 바이너리임을 알림

### `header::CONTENT_DISPOSITION`

- **정의**: HTTP `Content-Disposition` 헤더 상수
- **위치**: `handler.rs:533`
- **값**: `attachment; filename="retrospect_report_{id}_{timestamp}.pdf"`
- **용도**: 브라우저에서 파일 다운로드 다이얼로그를 트리거하고 파일명을 지정
- **참고**: `axum::http::header` (`handler.rs:3`)

### `header::CACHE_CONTROL`

- **정의**: HTTP `Cache-Control` 헤더 상수
- **위치**: `handler.rs:537`
- **값**: `"no-cache, no-store, must-revalidate"`
- **용도**: 항상 최신 PDF를 생성하도록 브라우저 캐시를 방지

### `(headers, body)` 튜플 응답

- **정의**: Axum에서 헤더 배열과 바이트 배열의 튜플이 자동으로 `IntoResponse`를 구현
- **위치**: `handler.rs:542` - `Ok((headers, pdf_bytes))`
- **형태**: `([(HeaderName, String); 3], Vec<u8>)` -- 3개의 헤더(CONTENT_TYPE, CONTENT_DISPOSITION, CACHE_CONTROL)와 PDF 바이트
- **용도**: 커스텀 헤더와 바이너리 본문을 한 번에 반환

---

## genpdf 라이브러리 관련

### `genpdf::Document`

- **정의**: PDF 문서 최상위 객체
- **위치**: `service.rs:1355`
- **메서드**: `new(font_family)`, `set_title()`, `set_minimal_conformance()`, `set_page_decorator()`, `push()`, `render()`
- **용도**: PDF 문서의 생성, 요소 추가, 최종 렌더링을 담당
- **참고**: `set_minimal_conformance()`는 PDF 표준 최소 준수 모드를 활성화하여 다양한 PDF 뷰어와의 호환성을 높임

### `genpdf::fonts::from_files()`

- **정의**: 디렉토리에서 폰트 패밀리(Regular, Bold, Italic, BoldItalic)를 일괄 로딩하는 함수
- **위치**: `service.rs:1309`
- **시그니처**: `from_files(dir: &str, family_name: &str, default_glyph: Option<char>)`
- **용도**: 한글 표시를 위한 NanumGothic 폰트 로딩

### `genpdf::fonts::FontFamily`

- **정의**: Regular, Bold, Italic, BoldItalic 네 가지 폰트 데이터를 묶은 구조체
- **위치**: `service.rs:1325` (폴백 구성 시)
- **용도**: `from_files` 실패 시 Regular 폰트 하나로 네 가지 스타일을 모두 대체

### `genpdf::fonts::FontData::new()`

- **정의**: 바이트 배열에서 폰트 데이터를 생성하는 생성자
- **위치**: `service.rs:1326, 1334, 1337, 1345`
- **시그니처**: `new(bytes: Vec<u8>, index: Option<u32>)`
- **용도**: 파일에서 읽은 TTF 바이트를 genpdf 폰트 데이터로 변환

### `genpdf::elements::Paragraph`

- **정의**: 텍스트 단락을 표현하는 PDF 요소
- **위치**: `service.rs:1366, 1377, 1381-1383` 등 다수
- **메서드**: `new(text)`, `styled(style)`
- **용도**: 제목, 본문 텍스트, 답변 내용 등을 PDF에 추가

### `genpdf::elements::Break`

- **정의**: 수직 여백(줄바꿈)을 표현하는 PDF 요소
- **위치**: `service.rs:1369, 1380, 1397` 등 다수
- **메서드**: `new(height)` - height는 줄 높이 배수
- **용도**: 섹션 간 시각적 구분을 위한 여백 추가

### `genpdf::style::Style`

- **정의**: 텍스트의 시각적 스타일(글꼴 크기, 굵기 등)을 정의하는 구조체
- **위치**: `service.rs:1367, 1378, 1428` 등
- **메서드**: `new()`, `bold()`, `with_font_size(size)`
- **용도**: 제목은 18pt Bold, 섹션 제목은 14pt Bold, 질문은 기본 크기 Bold 등으로 스타일링

### `genpdf::SimplePageDecorator`

- **정의**: 페이지 레이아웃(여백 등)을 설정하는 데코레이터
- **위치**: `service.rs:1360-1362`
- **메서드**: `new()`, `set_margins(mm)`
- **용도**: 모든 페이지에 15mm 여백을 적용

### `doc.render(&mut buf)`

- **정의**: PDF 문서를 바이트 스트림으로 렌더링하는 메서드
- **위치**: `service.rs:1487`
- **시그니처**: `render(&self, writer: &mut impl Write)`
- **용도**: 메모리 내 `Vec<u8>` 버퍼에 PDF 바이너리를 기록

---

## chrono 관련

### `Utc::now()`

- **정의**: 현재 UTC 시각을 반환하는 함수
- **위치**: `handler.rs:523`
- **용도**: PDF 파일명에 포함할 타임스탬프 생성

### `.format("%Y%m%d_%H%M%S")`

- **정의**: chrono의 날짜/시간 포맷팅 메서드
- **위치**: `handler.rs:523`
- **출력 예시**: `"20250125_143022"`
- **용도**: 파일명에 사용할 수 있는 안전한 문자열(슬래시/콜론 미포함)로 변환

---

## Rust 표준 라이브러리 / 패턴

### `std::env::var()`

- **정의**: 환경변수 값을 읽는 함수
- **위치**: `service.rs:1305, 1307`
- **반환**: `Result<String, VarError>`
- **용도**: `PDF_FONT_DIR`, `PDF_FONT_FAMILY` 환경변수를 읽되, 없으면 기본값 사용

### `unwrap_or_else(|| default)`

- **정의**: `Result`/`Option`이 `Err`/`None`일 때 클로저를 실행해 기본값을 반환하는 메서드
- **위치**: `service.rs:1305, 1307, 1065, 1447`
- **용도**: 환경변수 미설정 시 기본값, 팀/멤버 미존재 시 대체 텍스트 제공

### `HashSet::insert()` 필터 패턴

- **정의**: `HashSet`에 값을 삽입하고, 새로 삽입된 경우 `true`를 반환하는 메서드
- **위치**: `service.rs:1422`
- **패턴**: `.filter(|r| seen.insert(r.question.clone()))`
- **용도**: 한 번의 순회로 중복 질문을 제거하여 고유한 질문 목록을 추출

### `HashMap<i64, String>` 매핑

- **정의**: 멤버 ID를 닉네임에 매핑하는 해시맵
- **위치**: `service.rs:1087-1090`
- **용도**: PDF에서 답변 작성자를 `[닉네임] 답변 내용` 형태로 표시하기 위한 조회 테이블

---

## 에러 타입

### `AppError::PdfGenerationFailed`

- **정의**: PDF 생성 과정에서 발생하는 에러를 표현하는 커스텀 에러 타입
- **위치**: `service.rs:1319, 1328, 1335, 1338, 1345, 1488`
- **에러 코드**: `COMMON500`
- **HTTP 상태**: 500 Internal Server Error
- **사용자 응답 메시지**: `"PDF 생성 중 서버 에러가 발생했습니다."` (내부 상세 메시지는 로그에만 기록)
- **발생 조건**: 폰트 파일 읽기 실패, 폰트 데이터 로딩 실패, PDF 렌더링 실패

### `AppError::RetrospectNotFound`

- **정의**: 회고가 존재하지 않거나 접근 권한이 없을 때 발생하는 에러
- **위치**: `service.rs:332, 346`
- **에러 코드**: `RETRO4041`
- **HTTP 상태**: 404 Not Found
- **용도**: API-021에서는 보안상 "회고 미존재"와 "팀 멤버 아님"을 동일한 404로 통합 처리

---

## 비즈니스 로직 관련

### `find_retrospect_for_member()`

- **정의**: 회고 존재 확인 + 팀 멤버십 검증을 한 번에 수행하는 헬퍼 메서드
- **위치**: `service.rs:323-352`
- **용도**: API-021, API-013 등 팀 멤버만 접근 가능한 API에서 공통으로 사용
- **보안 패턴**: 회고 미존재와 접근 권한 없음을 같은 404로 반환하여 회고 존재 여부를 비멤버에게 노출하지 않음

### `retrospect_method_display()`

- **정의**: 회고 방식 enum을 PDF 표시용 문자열로 변환하는 헬퍼
- **위치**: `service.rs:1284-1293`
- **매핑**: `Kpt` -> `"KPT"`, `FourL` -> `"4L"`, `FiveF` -> `"5F"`, `Pmi` -> `"PMI"`, `Free` -> `"Free"`
- **용도**: PDF 기본 정보 섹션의 "Method" 필드에 사용
