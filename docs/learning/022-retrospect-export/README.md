# [API-022] 회고 내보내기 (PDF) 학습 노트

## 개요

- 엔드포인트: `GET /api/v1/retrospects/{retrospectId}/export`
- 역할: 특정 회고 세션의 전체 내용(팀 인사이트, 팀원별 답변, 개인 인사이트 등)을 PDF 파일로 생성하여 바이너리 응답으로 반환
- 인증: Bearer 토큰 필수 (팀 멤버만 접근 가능)

## 이 API의 특징

이 API는 프로젝트 내 다른 API들과 달리 **JSON이 아닌 PDF 바이너리**를 직접 응답으로 반환한다.
일반적인 `Json<BaseResponse<T>>` 패턴 대신 `impl IntoResponse`를 사용하여 HTTP 헤더와 바이트 배열을 직접 구성한다.

## 소스 파일

| 파일 | 역할 |
|------|------|
| `codes/server/src/domain/retrospect/handler.rs` (500-543행) | HTTP 핸들러: 인증, Path 검증, PDF 바이트 수신, 응답 헤더 구성 |
| `codes/server/src/domain/retrospect/service.rs` (1042-1132행) | 비즈니스 로직: 회고 데이터 수집 및 PDF 바이트 생성 위임 |
| `codes/server/src/domain/retrospect/service.rs` (1284-1293행) | 회고 방식 표시명 헬퍼 (`retrospect_method_display`) |
| `codes/server/src/domain/retrospect/service.rs` (1295-1491행) | PDF 생성 로직: genpdf 라이브러리를 사용한 문서 렌더링 (`generate_pdf`) |
| `codes/server/src/domain/retrospect/service.rs` (320-352행) | 회고 조회 + 멤버십 확인 헬퍼 (`find_retrospect_for_member`) |
| `codes/server/src/utils/error.rs` | `AppError::PdfGenerationFailed` 에러 정의 |
| `docs/api-specs/021-retrospect-export.md` | API 스펙 문서 |

## 학습 문서 구성

| 문서 | 내용 |
|------|------|
| [flow.md](./flow.md) | 전체 동작 흐름 (핸들러 -> 서비스 -> PDF 생성 -> 바이너리 응답) |
| [key-concepts.md](./key-concepts.md) | 핵심 개념 (IntoResponse, Content-Disposition, genpdf 등) |
| [keywords.md](./keywords.md) | 학습 키워드 정리 |
| [spring-dev-guide.md](./spring-dev-guide.md) | Spring 개발자용 전체 학습 가이드 |

## 핵심 학습 포인트

1. **바이너리 응답 패턴**: Axum에서 JSON이 아닌 바이너리 데이터를 반환하는 방법
2. **genpdf 라이브러리**: Rust에서 PDF를 프로그래밍 방식으로 생성하는 방법
3. **HTTP 헤더 직접 구성**: Content-Type, Content-Disposition, Cache-Control 헤더를 수동으로 설정하는 방법
4. **폰트 폴백 전략**: 전체 폰트 패밀리 로딩 실패 시 Regular 폰트로 대체하는 방어적 프로그래밍

## 참고

- 관련 소스: `codes/server/src/domain/retrospect/handler.rs`, `service.rs`
- API 스펙: `docs/api-specs/021-retrospect-export.md`
