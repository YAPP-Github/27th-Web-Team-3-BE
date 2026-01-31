# API-011: 회고 생성

## 개요

| 항목 | 내용 |
|------|------|
| 엔드포인트 | `POST /api/v1/retrospects` |
| 메서드 | POST |
| 인증 | Bearer Token (JWT) 필수 |

프로젝트 회고 세션을 생성하고, 선택한 회고 방식에 따라 기본 질문 5개를 자동 생성하는 API입니다. RetroRoom, Retrospect, Response(×5), RetroReference(×N)를 하나의 트랜잭션으로 생성합니다.

## 문서 목록

- [spring-guide.md](./spring-guide.md) - **(추천)** Spring 개발자를 위한 Rust 코드 분석 가이드
- [flow.md](./flow.md) - 동작 흐름 (Handler -> Service -> DB 트랜잭션)
- [key-concepts.md](./key-concepts.md) - 핵심 개념 (DTO 검증, chrono 날짜/시간 처리, 트랜잭션, Enum 매핑, 에러 처리)
- [keywords.md](./keywords.md) - 학습 키워드 정리 (chrono, SeaORM DateTime 포함)

## 주요 소스 파일

| 파일 | 역할 |
|------|------|
| `codes/server/src/domain/retrospect/handler.rs` | HTTP 핸들러 (입력 검증, 서비스 호출) |
| `codes/server/src/domain/retrospect/service.rs` | 비즈니스 로직 (URL/날짜 검증, 트랜잭션 처리) |
| `codes/server/src/domain/retrospect/dto.rs` | 요청/응답 DTO 정의 |
| `codes/server/src/domain/retrospect/entity/retrospect.rs` | 회고 엔티티, RetrospectMethod Enum |
| `codes/server/src/utils/auth.rs` | JWT 인증 Extractor |
| `codes/server/src/utils/error.rs` | AppError 정의 및 IntoResponse 구현 |
| `codes/server/src/utils/response.rs` | 공통 응답 구조체 |
