# 학습 가이드: API-023 보관함 회고 검색

이 디렉토리는 API-023 보관함 회고 검색 기능을 구현하고 이해하기 위한 학습 자료를 담고 있습니다.

## 개요

| 항목 | 내용 |
|------|------|
| **엔드포인트** | `GET /api/v1/retrospects/search?keyword=...` |
| **HTTP 메서드** | GET |
| **인증** | Bearer JWT 필수 |
| **기능** | 사용자가 속한 팀의 회고를 프로젝트명(title)으로 검색 |

## 핵심 특징

- **Optional 키워드**: `keyword`를 `Option<String>`으로 받아 커스텀 에러 코드(`SEARCH4001`) 반환
- **다중 쿼리 패턴**: `member_team` -> `team` -> `retrospect` 3개 순차 쿼리 + HashMap 인메모리 매핑
- **LIKE 검색**: SeaORM `contains()`로 SQL `LIKE '%keyword%'` 생성
- **안정 정렬**: `start_time DESC, retrospect_id DESC` 이중 정렬

## 관련 소스 파일

| 파일 | 역할 | 라인 |
|------|------|------|
| `handler.rs` | HTTP 핸들러 (파라미터 추출, 인증) | 457-476 |
| `service.rs` | 키워드 검증 + 검색 로직 | 952-1040 |
| `dto.rs` | SearchQueryParams, SearchRetrospectItem | 506-541 |
| `error.rs` | SearchKeywordInvalid (SEARCH4001) | 111-112 |

## 에러 코드

| 코드 | HTTP | 설명 |
|------|------|------|
| SEARCH4001 | 400 | 검색어 누락, 공백만 입력, 100자 초과 |
| AUTH4001 | 401 | JWT 인증 실패 |
| COMMON500 | 500 | DB 연결 실패 등 서버 오류 |

## 문서 구성

| 파일명 | 설명 |
|--------|------|
| [flow.md](./flow.md) | 전체 요청-응답 흐름도와 3개 DB 쿼리 상세 |
| [key-concepts.md](./key-concepts.md) | Optional 파라미터, 다중 쿼리 패턴, 검증 함수, LIKE 검색 등 8개 핵심 개념 |
| [keywords.md](./keywords.md) | Rust/Axum, SeaORM, 에러 처리, 데이터 모델 키워드 정리 |
| [spring-guide.md](./spring-guide.md) | Spring 개발자를 위한 Java/Kotlin 비교 가이드 |

## 시작하기

Spring Framework 경험이 있는 개발자는 `spring-guide.md`를 먼저 읽어보세요. Rust 패턴을 이해하고 싶다면 `key-concepts.md`부터 시작하세요.
