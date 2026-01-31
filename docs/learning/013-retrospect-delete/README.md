# [API-013] 회고 삭제 API 학습 노트

## 개요
- **엔드포인트**: `DELETE /api/v1/retrospects/{retrospectId}`
- **역할**: 특정 회고 세션과 연관된 모든 데이터(댓글, 좋아요, 답변, 참고자료, 참여자, 회고방)를 cascade 방식으로 영구 삭제
- **인증**: Bearer 토큰 필요
- **한 줄 설명**: 회고 ID를 받아 팀 멤버십을 검증한 후, 트랜잭션 내에서 연관 테이블을 FK 의존 순서대로 삭제하는 API

## 스펙 vs 구현 차이점

> **주의**: API 스펙(`docs/api-specs/013-retrospect-delete.md`)과 실제 구현 사이에 다음과 같은 차이가 있습니다.
> 이는 DB 스키마에 `created_by`(회고 생성자) 및 `member_team.role`(팀 역할) 필드가 아직 없기 때문입니다.

| 항목 | 스펙 (013-retrospect-delete.md) | 실제 구현 |
|------|-------------------------------|-----------|
| **삭제 권한** | 팀 Owner 또는 회고 생성자만 삭제 가능 | 팀 멤버라면 누구나 삭제 가능 |
| **403 에러 (RETRO4031)** | 권한 없는 사용자에게 403 반환 | 403 미사용. `RetroDeleteAccessDenied` 에러가 `error.rs`에 정의되어 있으나 `#[allow(dead_code)]`로 비활성 상태 |
| **404 에러 메시지** | `"존재하지 않는 회고입니다."` | `"존재하지 않는 회고이거나 접근 권한이 없습니다."` (보안상 미존재/비멤버를 동일 메시지로 통합) |
| **Swagger 403 응답** | 스펙에 403 정의됨 | `handler.rs`의 `#[utoipa::path]`에 403 응답 없음 (구현 현실 반영) |

**향후 계획**: `service.rs:1136-1138`의 TODO 주석에 따르면, 스키마 마이그레이션으로 `created_by`와 `role` 필드가 추가된 후 권한 분기(`RetroDeleteAccessDenied`)를 활성화할 예정입니다.

## 문서 목록

| 문서 | 설명 |
|------|------|
| [study-guide.md](./study-guide.md) | **(추천)** Spring 개발자를 위한 Rust API 학습 가이드 (JPA 비교, 코드 매핑) |
| [flow.md](./flow.md) | Handler -> Service -> DB 단계별 동작 흐름, cascade 삭제 순서, 트랜잭션 처리 |
| [key-concepts.md](./key-concepts.md) | cascade 삭제 패턴, 트랜잭션 순서 보장, find_retrospect_for_member 보안 패턴, SeaORM delete_many 등 핵심 개념 |
| [keywords.md](./keywords.md) | TransactionTrait, DeleteMany, cascade delete, RAII rollback 등 학습 키워드 정리 |

## 관련 소스

| 파일 | 역할 |
|------|------|
| `codes/server/src/domain/retrospect/handler.rs` (L624-668) | HTTP 핸들러 - Path 검증, 인증, 서비스 호출 |
| `codes/server/src/domain/retrospect/service.rs` (L1134-1282) | 비즈니스 로직 - 멤버십 확인, 트랜잭션 cascade 삭제 (L1136-1138에 권한 관련 TODO) |
| `codes/server/src/domain/retrospect/service.rs` (L323-352) | `find_retrospect_for_member` 헬퍼 - 보안 패턴 |
| `codes/server/src/utils/error.rs` (L120-124) | `RetroDeleteAccessDenied` 에러 타입 정의 (`#[allow(dead_code)]`, 미사용) |
| `docs/api-specs/013-retrospect-delete.md` | API 스펙 문서 (구현과 차이점 존재 - 상단 표 참조) |
