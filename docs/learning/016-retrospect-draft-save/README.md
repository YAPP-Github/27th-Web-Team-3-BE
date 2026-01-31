# API-016: 회고 답변 임시 저장

## 개요

진행 중인 회고의 답변을 임시로 저장하는 API입니다.
프론트엔드의 자동 저장(Auto-save) 기능에 최적화되어 있으며, 기존에 저장된 내용이 있다면 전달받은 내용으로 **덮어쓰기(Overwrite)** 합니다.

## 엔드포인트

```
PUT /api/v1/retrospects/{retrospectId}/drafts
```

## 주요 특징

- **PUT 메서드 사용**: 동일한 요청을 여러 번 보내도 결과가 동일한 멱등성(idempotency) 보장
- **부분 저장 허용**: 5개 질문 중 일부만 선택하여 저장 가능 (최소 1개, 최대 5개)
- **null/빈 문자열 허용**: `content` 필드에 `null` 또는 `""`를 전달하면 기존 내용을 삭제하는 용도로 사용
- **트랜잭션 보장**: 여러 답변의 upsert를 하나의 트랜잭션으로 원자적 처리

## 관련 소스 파일

| 파일 | 역할 |
|------|------|
| `codes/server/src/domain/retrospect/handler.rs` | HTTP 핸들러 (라인 229~252) |
| `codes/server/src/domain/retrospect/service.rs` | 비즈니스 로직 (`save_draft`, 라인 460~572) |
| `codes/server/src/domain/retrospect/service.rs` | 입력 검증 (`validate_drafts`, 라인 1494~1539) |
| `codes/server/src/domain/retrospect/dto.rs` | DTO 정의 (라인 96~132) |
| `codes/server/src/utils/auth.rs` | `AuthUser.user_id()` 헬퍼 (라인 14~19) |
| `codes/server/src/utils/error.rs` | 에러 타입 정의 |
| `docs/api-specs/016-retrospect-draft-save.md` | API 스펙 문서 |

## 문서 목록

- [spring-guide.md](./spring-guide.md) - **(추천)** Spring 개발자를 위한 Rust 코드 분석 가이드
- [flow.md](./flow.md) - 동작 흐름 (Handler -> Service -> DB 트랜잭션)
- [key-concepts.md](./key-concepts.md) - 핵심 개념 (부분 업데이트, Nullable 처리)
- [keywords.md](./keywords.md) - 학습 키워드 정리

