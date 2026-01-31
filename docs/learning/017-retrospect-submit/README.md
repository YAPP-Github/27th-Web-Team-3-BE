# API-017: 회고 최종 제출

## 개요

작성한 모든 답변(총 5개)을 최종 제출하는 API입니다.
각 답변은 최대 1,000자까지 입력 가능하며, 제출 완료 시 회고 참여 상태가 `SUBMITTED`로 변경됩니다.

## 문서 목록

| 문서 | 설명 |
|------|------|
| [study-guide.md](./study-guide.md) | **(추천)** Spring 개발자를 위한 Rust API 학습 가이드 (JPA 비교, 락/트랜잭션) |
| [flow.md](./flow.md) | Handler -> Service -> DB 단계별 동작 흐름, 트랜잭션 범위, 에러 처리 |
| [key-concepts.md](./key-concepts.md) | 상태 머신, 완전성 검증, 공백 처리, 행 잠금(Lock), UTC/KST 시간 처리 등 핵심 개념 |
| [keywords.md](./keywords.md) | RetrospectStatus, lock_exclusive, HashSet, trim, ActiveModel 등 학습 키워드 정리 |

## 엔드포인트

```
POST /api/v1/retrospects/{retrospectId}/submit
```

## 주요 파일

| 파일 | 역할 |
|------|------|
| `codes/server/src/domain/retrospect/handler.rs` | HTTP 핸들러 (라인 324~347) |
| `codes/server/src/domain/retrospect/service.rs` | 비즈니스 로직 (라인 575~685) |
| `codes/server/src/domain/retrospect/service.rs` | 답변 검증 함수 (라인 1543~1578) |
| `codes/server/src/domain/retrospect/dto.rs` | 요청/응답 DTO (라인 138~176) |
| `codes/server/src/domain/member/entity/member_retro.rs` | RetrospectStatus enum (라인 11~21) |
| `codes/server/src/utils/error.rs` | 에러 타입 정의 |
| `docs/api-specs/017-retrospect-submit.md` | API 스펙 문서 |

## 핵심 동작

1. 핸들러에서 `retrospectId` 경로 파라미터 검증 (1 이상)
2. 서비스 레이어에서 답변 비즈니스 검증 수행 (정확히 5개, 공백 불가, 1000자 제한)
3. 트랜잭션 내에서 행 잠금(exclusive lock)을 통한 동시 제출 방지
4. 5개 답변을 `response` 테이블에 업데이트
5. `member_retro` 상태를 `SUBMITTED`로 변경
6. 트랜잭션 커밋 후 응답 반환

## 에러 코드

| 코드 | HTTP | 설명 |
|------|------|------|
| RETRO4002 | 400 | 답변 누락 (5개 미만 또는 질문 번호 불일치) |
| RETRO4003 | 400 | 답변 길이 초과 (1,000자 초과) |
| RETRO4007 | 400 | 공백만 입력된 답변 |
| RETRO4033 | 403 | 이미 제출 완료된 회고 |
| RETRO4041 | 404 | 존재하지 않는 회고 |

## 학습 포인트

- 상태 머신 패턴 (DRAFT -> SUBMITTED -> ANALYZED)
- 트랜잭션과 행 잠금을 이용한 동시성 제어
- 비즈니스 검증을 트랜잭션 전에 수행하여 불필요한 DB 자원 점유 방지
- UTC 저장 + KST 변환 응답 패턴
