# API-021 학습 가이드 (Spring 개발자용)

이 문서는 JVM/Spring 기반 개발자가 Rust/Axum/SeaORM으로 구현된 **API-021 회고 내보내기(PDF)**를 온전히 이해할 수 있도록 정리했다. Rust 문법, 프레임워크 개념, ORM 동작을 Spring 관점에서 대응시켜 설명한다.

---

## 1) 이 API가 하는 일 (한 문장 요약)

`GET /api/v1/retrospects/{retrospectId}/export` 요청을 받으면 **회고 전체 내용**을 PDF로 생성해 **바이너리(PDF)로 다운로드**시킨다.

---

## 2) Spring ↔ Rust/Axum/SeaORM 대응표

| Spring 개념 | 이 프로젝트(Rust/Axum/SeaORM) | 파일 |
|---|---|---|
| Controller | Axum Handler 함수 | `codes/server/src/domain/retrospect/handler.rs` |
| Service | Service 메서드 | `codes/server/src/domain/retrospect/service.rs` |
| DTO | 없음 (PDF 바이너리 반환) | - |
| Repository/JPA | SeaORM Entity + Query | `codes/server/src/domain/` |
| Security (@AuthenticationPrincipal) | `AuthUser` Extractor | `codes/server/src/utils/auth.rs` |
| 공통 응답 래핑 | 사용 안 함 (바이너리 직접 반환) | - |
| ResponseEntity<byte[]> | `(headers, Vec<u8>)` + `IntoResponse` | `handler.rs` |

---

## 3) Spring 관점에서 보는 전체 흐름

Spring 스타일로 해석하면 아래와 같다:

1. `@GetMapping("/retrospects/{id}/export")`  
2. `@AuthenticationPrincipal`로 사용자 ID 추출  
3. 서비스에서 회고 데이터 조회 + 권한 확인  
4. PDF 생성 → `byte[]` 반환  
5. `ResponseEntity<byte[]>`로 헤더(Content-Disposition, Content-Type 등) 세팅 후 응답

Axum에서는 이를 **Handler → Service → PDF 생성 → IntoResponse**로 처리한다.

---

## 4) Handler (Controller) 역할

**파일**: `codes/server/src/domain/retrospect/handler.rs`

핸들러(`handler.rs:500-543`)는 다음을 수행한다:
1. `retrospectId` 검증 (1 이상의 양수)
2. JWT에서 `user_id` 추출 (`user.0.sub.parse()` -- 이 API는 `user.user_id()` 헬퍼 대신 인라인 파싱을 사용)
3. 서비스 호출로 PDF 바이트(`Vec<u8>`) 획득
4. 동적 파일명 생성 (UTC 타임스탬프 포함)
5. HTTP 헤더(Content-Type, Content-Disposition, Cache-Control) 구성 후 바이너리 응답

핵심 차이점:
```rust
pub async fn export_retrospect(...) -> Result<impl IntoResponse, AppError>
```
JSON 대신 **PDF 바이트**를 반환해야 하므로 `Json<BaseResponse<T>>`가 아니라 `impl IntoResponse`를 사용한다.

> **참고**: 같은 프로젝트 내에서도 API-016의 `save_draft`는 `user.user_id()` 헬퍼를 사용하고, API-021의 `export_retrospect`는 `user.0.sub.parse()`를 인라인으로 사용한다. 기능은 동일하지만 구현 시점에 따라 스타일이 다르다.

Spring 대응:
- `ResponseEntity<byte[]>` 혹은 `ResponseEntity<Resource>`

---

## 5) Service 역할 (DB 조회 + PDF 생성)

**파일**: `codes/server/src/domain/retrospect/service.rs`

서비스는 다음 단계로 동작한다:

1. **회고 존재 + 팀 멤버 여부 확인**  
   - `find_retrospect_for_member()` 사용  
   - 회고가 없거나 멤버가 아니면 **404**로 통합 처리 (보안 목적)

2. **팀 이름 조회**  
   - 팀이 없어도 `"(알 수 없음)"`으로 안전하게 처리

3. **참여 멤버 목록 조회**  
   - `member_retro`로 참여자 ID 조회  
   - `member`에서 닉네임 조회 후 `HashMap`으로 매핑

4. **질문/답변 조회 + 응답-멤버 매핑**  
   - `response`로 질문과 답변 조회  
   - `member_response`로 `response_id -> member_id` 매핑

5. **PDF 생성**  
   - `generate_pdf(...)`에서 genpdf로 문서 구성

---

## 6) SeaORM (JPA/Hibernate와 비교)

| JPA/Hibernate | SeaORM |
|---|---|
| `findById()` | `Entity::find_by_id().one()` |
| `findAll()` | `Entity::find().all()` |
| `where` 조건 | `.filter(Column::X.eq(value))` |
| `IN` 쿼리 | `.filter(Column::X.is_in(vec))` |
| 정렬 | `.order_by_asc()` |

**중요 포인트**
SeaORM은 자동 JOIN 대신, 여러 번 쿼리 후 메모리에서 매핑하는 방식이 일반적이다.
Spring에서 `@OneToMany` + `JOIN FETCH`를 쓰는 것을 수동으로 구현한 느낌이다.

이 API에서 수행되는 실제 DB 쿼리 순서:
1. `retrospect` - 회고 조회 (`find_by_id`)
2. `member_team` - 팀 멤버십 확인 (`filter`)
3. `team` - 팀 이름 조회 (`find_by_id`)
4. `member_retro` - 참여자 목록 조회 (`filter` + `order_by_asc`)
5. `member` - 멤버 닉네임 조회 (`filter` + `is_in`)
6. `response` - 답변 조회 (`filter` + `order_by_asc`)
7. `member_response` - 답변-멤버 매핑 조회 (`filter` + `is_in`)

총 7번의 SELECT 쿼리가 발생한다. JPA라면 `@EntityGraph`나 `JOIN FETCH`로 줄일 수 있지만, SeaORM에서는 명시적 다중 쿼리가 관례다.

---

## 7) PDF 내용은 어떻게 구성되는가?

**생성 순서 (generate_pdf)**:

1. **폰트 로딩**
   - `PDF_FONT_DIR`, `PDF_FONT_FAMILY` 환경변수 사용
   - 전체 폰트 패밀리 로딩 실패 시 **Regular 폰트로 폴백**

2. **문서 초기화**
   - 제목 설정, `set_minimal_conformance()` 호출, 페이지 여백 15mm

3. **제목 섹션**
   - `{회고 제목} - Retrospect Report`

4. **기본 정보 섹션**
   - 팀명, 날짜/시간, 회고 방식, 참여자 목록

5. **팀 인사이트 섹션**
   - `retrospects.team_insight`가 있을 때만 추가

6. **질문/답변 섹션**
   - 질문을 중복 제거 (`HashSet`)
   - 각 질문 아래에 모든 답변 나열
   - 답변이 비어 있으면 `(No answers)` 표시

7. **개인 인사이트 섹션**
   - `member_retro.personal_insight` 있는 멤버만 출력

---

## 8) Rust 문법 포인트 (Spring 개발자 입장에서)

### 8-1. `Result<T, E>` + `?`
- 에러가 나면 즉시 반환하는 **조기 리턴** 패턴

### 8-2. `Option<T>`
- Java의 `null` 대신 `Option`
- `unwrap_or_else`로 기본값 지정

### 8-3. `HashMap`, `HashSet`
- `HashMap`: 멤버 ID → 닉네임 매핑
- `HashSet`: 질문 중복 제거

### 8-4. `format!` 매크로
- Java `String.format` 역할

---

## 9) 바이너리 응답 이해 (Spring과의 비교)

Spring 예시:
```java
@GetMapping("/retrospects/{id}/export")
public ResponseEntity<byte[]> export(@PathVariable Long id) {
    byte[] pdf = service.export(id);
    return ResponseEntity.ok()
        .header(HttpHeaders.CONTENT_DISPOSITION, "attachment; filename=\"...\"")
        .contentType(MediaType.APPLICATION_PDF)
        .body(pdf);
}
```

Axum에서는:
```rust
let headers = [
    (header::CONTENT_TYPE, "application/pdf; charset=utf-8".to_string()),
    (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", filename)),
    (header::CACHE_CONTROL, "no-cache, no-store, must-revalidate".to_string()),
];
Ok((headers, pdf_bytes))
```

즉, `ResponseEntity` 대신 **헤더 배열 + 바이트 배열 튜플**을 반환한다.

---

## 10) 에러 처리 포인트

- `retrospectId < 1` → 400
- JWT 문제 → 401
- 회고 없음 / 팀 멤버 아님 → 404 (보안상 통합)
- PDF 생성 실패 → 500

Spring에서는 `@ControllerAdvice`로 처리할 로직이 Rust에서는 `AppError`와 `IntoResponse`로 처리된다.

---

## 11) 이해를 위한 추천 읽기 순서

1. `docs/api-specs/021-retrospect-export.md` (스펙 이해)
2. `codes/server/src/domain/retrospect/handler.rs` (핸들러)
3. `codes/server/src/domain/retrospect/service.rs` (서비스 + PDF 생성)
4. `docs/learning/021-retrospect-export/flow.md` (전체 흐름)
5. `docs/learning/021-retrospect-export/key-concepts.md` (핵심 패턴)

---

## 12) 한 번 더 요약 (핵심만)

- JSON이 아니라 **PDF 바이너리**를 반환하는 특수 API
- `impl IntoResponse`로 헤더 + `Vec<u8>` 응답 구성
- SeaORM으로 회고/멤버/답변 조회 후 genpdf로 PDF 생성
- 보안상 **회고 없음/권한 없음 → 404 통합**
