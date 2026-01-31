# API-011: 팀 회고 목록 조회

## 개요

| 항목 | 내용 |
|------|------|
| 엔드포인트 | `GET /api/v1/teams/{teamId}/retrospects` |
| 메서드 | GET |
| 인증 | Bearer Token (JWT) 필수 |

특정 팀에 속한 모든 회고 목록을 최신순으로 조회하는 API입니다. 과거/오늘/예정된 회고가 모두 포함되며, 필터링 없이 전체 목록을 반환합니다.

## 문서 목록

- [flow.md](./flow.md) - 동작 흐름 (Handler -> Service -> DB)
- [key-concepts.md](./key-concepts.md) - 핵심 개념 (Rust/Axum/SeaORM 패턴)
- [keywords.md](./keywords.md) - 학습 키워드 정리
- [study-guide.md](./study-guide.md) - Spring 개발자를 위한 학습 가이드

## 주요 소스 파일

| 파일 | 역할 |
|------|------|
| `codes/server/src/domain/retrospect/handler.rs` | HTTP 핸들러 (입력 검증, 서비스 호출) |
| `codes/server/src/domain/retrospect/service.rs` | 비즈니스 로직 (팀 존재 확인, 멤버십 확인, DB 조회) |
| `codes/server/src/domain/retrospect/dto.rs` | 요청/응답 DTO 정의 |
| `codes/server/src/domain/retrospect/entity/retrospect.rs` | 회고 엔티티 및 SeaORM 모델 |
| `codes/server/src/utils/auth.rs` | JWT 인증 Extractor |
| `codes/server/src/utils/response.rs` | 공통 응답 구조체 |
