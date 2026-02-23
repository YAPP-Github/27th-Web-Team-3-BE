# API-128: Open Graph 메타데이터 조회 API 리뷰

## 개요

외부 URL의 Open Graph 태그를 파싱하여 메타데이터(제목, 설명, 썸네일 이미지)를 반환하는 API입니다.
클라이언트에서 CORS로 직접 외부 URL에 접근할 수 없으므로 백엔드에서 프록시 역할을 합니다.

## 엔드포인트

```
GET /api/v1/og?url={url}
```

## 구현 내용

### 새 도메인 모듈: `og`

| 파일 | 역할 |
|------|------|
| `src/domain/og/mod.rs` | 모듈 정의 |
| `src/domain/og/dto.rs` | OgMetadataQuery, OgMetadataResponse |
| `src/domain/og/service.rs` | URL 요청 및 OG 태그 파싱 로직 |
| `src/domain/og/handler.rs` | HTTP 핸들러 |

### 주요 설계 결정

1. **Graceful Fallback**: 외부 URL 요청 실패 시에도 200 OK로 URL만 반환 (에러 X)
2. **타임아웃 5초**: 외부 사이트 응답 지연에 대비
3. **User-Agent 설정**: 봇 차단 우회를 위해 브라우저 유사 User-Agent 사용
4. **인증 필수**: Bearer Token으로 남용 방지

### 의존성 추가

- `scraper = "0.22"` : HTML 파싱 및 CSS 셀렉터 기반 OG 태그 추출

## 테스트

### 단위 테스트 (8개)

| 테스트 | 검증 내용 |
|--------|----------|
| `should_return_true_for_valid_http_url` | http:// URL 검증 |
| `should_return_true_for_valid_https_url` | https:// URL 검증 |
| `should_return_false_for_invalid_url` | 잘못된 URL 형식 거부 |
| `should_return_false_for_ftp_url` | ftp:// URL 거부 |
| `should_parse_og_tags_from_html` | OG 태그 전체 파싱 |
| `should_return_none_for_missing_og_tags` | OG 태그 없는 HTML 처리 |
| `should_parse_partial_og_tags` | 일부 OG 태그만 있는 경우 |
| `should_handle_empty_content_attribute` | 빈 content 속성 처리 |
| `should_return_fallback_response_with_url_only` | fallback 응답 검증 |
| `should_return_error_for_invalid_url_format` | 잘못된 URL에 대한 에러 |

## 코드 리뷰 체크리스트

- [x] TDD 원칙을 따라 테스트 코드가 작성되었는가?
- [x] 모든 테스트가 통과하는가?
- [x] API 문서가 작성되었는가?
- [x] 공통 유틸리티(BaseResponse, AppError)를 재사용했는가?
- [x] 에러 처리가 적절하게 되어 있는가? (unwrap 미사용)
- [x] 코드가 Rust 컨벤션을 따르는가? (clippy 경고 없음)
- [x] serde rename_all = "camelCase" 적용되었는가?
- [x] Swagger 문서가 작성되었는가?
