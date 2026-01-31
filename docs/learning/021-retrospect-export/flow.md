# 동작 흐름: 회고 내보내기 (PDF)

## 전체 흐름 요약

```
클라이언트 요청
  -> [핸들러] 인증 + Path 검증
  -> [서비스] 회고 조회 + 멤버십 확인
  -> [서비스] 데이터 수집 (팀, 멤버, 질문/답변, 인사이트)
  -> [서비스] PDF 생성 (genpdf)
  -> [핸들러] 동적 파일명 생성 + HTTP 헤더 구성
  -> 바이너리 응답 반환
```

## 단계별 상세 흐름

### 1단계: 핸들러 진입 및 인증/검증

**소스**: `handler.rs:500-517`

```rust
pub async fn export_retrospect(
    user: AuthUser,
    State(state): State<AppState>,
    Path(retrospect_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    if retrospect_id < 1 {
        return Err(AppError::BadRequest(
            "retrospectId는 1 이상의 양수여야 합니다.".to_string(),
        ));
    }
    let user_id: i64 = user.0.sub.parse()
        .map_err(|_| AppError::Unauthorized("유효하지 않은 사용자 ID입니다.".to_string()))?;
```

- `AuthUser` 미들웨어가 JWT 토큰을 자동으로 검증하고 사용자 정보를 추출한다.
- `retrospect_id`가 1 미만이면 `BadRequest` 에러를 반환한다.
- 반환 타입이 `Result<impl IntoResponse, AppError>`로, JSON이 아닌 임의의 응답을 반환할 수 있다.

### 2단계: 서비스 호출 - 회고 조회 및 멤버십 확인

**소스**: `service.rs:1054-1056`

```rust
let retrospect_model =
    Self::find_retrospect_for_member(&state, user_id, retrospect_id).await?;
```

내부적으로 `find_retrospect_for_member` (`service.rs:323-352`)가 두 가지를 수행한다:

1. **회고 존재 확인**: `retrospect::Entity::find_by_id(retrospect_id)` 로 회고를 DB에서 조회
2. **팀 멤버십 확인**: `member_team::Entity`를 통해 해당 사용자가 회고가 속한 팀(`retrospect_model.team_id`)의 멤버인지 검증

```rust
let is_member = member_team::Entity::find()
    .filter(member_team::Column::MemberId.eq(user_id))
    .filter(member_team::Column::TeamId.eq(retrospect_model.team_id))
    .one(&state.db).await?;
```

둘 중 하나라도 실패하면 동일한 `AppError::RetrospectNotFound`를 반환한다.
보안 정책상 "존재하지 않음"과 "권한 없음"을 동일한 404로 처리하여 비멤버에게 회고 존재 여부를 노출하지 않는다.

> **API-016과의 차이**: API-016(임시 저장)은 `find_retrospect_for_member`를 사용하지 않고, 회고 존재(404)와 참석자 확인(403, `member_retro` 테이블 기준)을 별도로 수행한다. API-021은 팀 멤버십(`member_team` 테이블 기준)을 확인하여 더 넓은 범위의 접근을 허용한다.

### 3단계: 팀 이름 조회

**소스**: `service.rs:1058-1065`

```rust
let team_model = team::Entity::find_by_id(retrospect_model.team_id)
    .one(&state.db).await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
let team_name = team_model
    .map(|t| t.name)
    .unwrap_or_else(|| "(알 수 없음)".to_string());
```

- 팀이 삭제된 경우에도 안전하게 `"(알 수 없음)"`으로 대체한다.
- PDF 문서의 "Basic Information" 섹션에 팀 이름이 표시된다.

### 4단계: 참여 멤버 조회

**소스**: `service.rs:1067-1090`

```rust
let member_retros = member_retro::Entity::find()
    .filter(member_retro::Column::RetrospectId.eq(retrospect_id))
    .order_by_asc(member_retro::Column::MemberRetroId)
    .all(&state.db).await?;
```

- `member_retro` 테이블에서 해당 회고에 참여한 멤버 ID를 조회한다 (`MemberRetroId` 오름차순).
- 멤버 ID가 비어 있으면 빈 배열로 처리, 아니면 `member` 테이블에서 닉네임을 가져온다.
- `HashMap<i64, String>` (member_id -> nickname) 매핑을 구성한다. 닉네임이 `None`이면 빈 문자열로 대체 (`unwrap_or_default()`).
- 이 매핑은 PDF에서 답변 작성자와 참여 멤버 목록을 표시하는 데 사용된다.

### 5단계: 질문/답변 조회 및 멤버 매핑

**소스**: `service.rs:1092-1113`

```rust
let responses = response::Entity::find()
    .filter(response::Column::RetrospectId.eq(retrospect_id))
    .order_by_asc(response::Column::ResponseId)
    .all(&state.db).await?;
```

- `response` 테이블에서 해당 회고의 모든 답변을 조회한다 (`ResponseId` 오름차순).
- 답변 ID 목록이 비어 있으면 빈 `HashMap`을 사용하고, 아니면 `member_response` 테이블을 통해 각 답변(response_id)이 어느 멤버(member_id)의 것인지 매핑한다.
- `response_member_map: HashMap<i64, i64>` (response_id -> member_id) 구조로 구성한다.

> **참고**: `response` 엔티티의 DB 컬럼명은 `response`이지만, Rust 필드명은 `content`이다 (`#[sea_orm(column_name = "response")]`). 또한 `question` 필드에 질문 텍스트가 저장되어 있어, 같은 질문에 여러 멤버의 답변이 행으로 존재한다.

### 6단계: PDF 생성

**소스**: `service.rs:1115-1123`, `service.rs:1295-1491`

```rust
let pdf_bytes = Self::generate_pdf(
    &retrospect_model, &team_name, &member_retros,
    &member_map, &responses, &response_member_map,
)?;
```

> **참고**: `generate_pdf`는 비동기가 아닌 동기 함수이다. DB 접근 없이 메모리 내 데이터만으로 PDF를 구성하므로 `async`가 불필요하다.

`generate_pdf` 함수는 다음 순서로 PDF를 구성한다:

1. **폰트 로딩** (1304-1353행): 환경변수 `PDF_FONT_DIR`, `PDF_FONT_FAMILY`에서 폰트 경로를 읽고, 전체 패밀리 로딩 실패 시 Regular 폰트로 대체
2. **문서 초기화** (1355-1362행): `genpdf::Document::new()`, 제목 설정, `set_minimal_conformance()`, 페이지 여백 15mm 설정
3. **제목 섹션** (1364-1369행): "{회고 제목} - Retrospect Report" (18pt, Bold)
4. **기본 정보 섹션** (1371-1397행): 팀명, 날짜/시간, 회고 방식(`retrospect_method_display()` 사용), 참여 멤버 목록 (인원 수 포함)
5. **팀 인사이트 섹션** (1399-1408행): `retrospect_model.team_insight`가 `Some`인 경우에만 AI가 생성한 팀 인사이트 표시
6. **질문/답변 섹션** (1410-1456행): 중복 제거된 질문별(`HashSet`)로 빈 답변을 제외한 모든 답변을 `[작성자] 내용` 형식으로 표시. 빈 답변만 있으면 `"(No answers)"` 표시
7. **개인 인사이트 섹션** (1458-1483행): `personal_insight`가 `Some`인 멤버만 출력. 멤버 이름이 없으면 `"Member #ID"` 형태로 대체
8. **PDF 렌더링** (1486-1488행): `doc.render(&mut buf)` 로 메모리 버퍼에 렌더링하고 `Vec<u8>` 반환

### 7단계: 동적 파일명 생성 및 응답 헤더 구성

**소스**: `handler.rs:522-543`

```rust
let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
let filename = format!("retrospect_report_{}_{}.pdf", retrospect_id, timestamp);

let headers = [
    (header::CONTENT_TYPE, "application/pdf; charset=utf-8".to_string()),
    (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", filename)),
    (header::CACHE_CONTROL, "no-cache, no-store, must-revalidate".to_string()),
];

Ok((headers, pdf_bytes))
```

- 파일명 형식: `retrospect_report_{id}_{YYYYMMdd_HHmmss}.pdf` (한글 미포함, UTC 기준)
- `(headers, body)` 튜플이 Axum의 `IntoResponse` 트레이트를 구현하고 있어 자동으로 HTTP 응답으로 변환된다.

## 에러 흐름

| 발생 위치 | 조건 | 에러 타입 | 에러 코드 | HTTP 상태 |
|-----------|------|-----------|-----------|-----------|
| 핸들러 (506행) | retrospectId < 1 | `AppError::BadRequest` | COMMON400 | 400 |
| 핸들러 (513-517행) | JWT sub 파싱 실패 | `AppError::Unauthorized` | AUTH4001 | 401 |
| 서비스 (332행) | 회고 미존재 | `AppError::RetrospectNotFound` | RETRO4041 | 404 |
| 서비스 (346행) | 팀 멤버가 아님 | `AppError::RetrospectNotFound` | RETRO4041 | 404 |
| 서비스 (1062행) | 팀 조회 DB 오류 | `AppError::InternalError` | COMMON500 | 500 |
| 서비스 (1318행) | 폰트 파일 읽기 실패 | `AppError::PdfGenerationFailed` | COMMON500 | 500 |
| 서비스 (1326-1345행) | 폰트 데이터 로딩 실패 | `AppError::PdfGenerationFailed` | COMMON500 | 500 |
| 서비스 (1487행) | PDF 렌더링 실패 | `AppError::PdfGenerationFailed` | COMMON500 | 500 |
