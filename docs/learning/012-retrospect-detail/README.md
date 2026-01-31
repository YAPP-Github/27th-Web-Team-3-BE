# API-012: 회고 상세 조회

## 요약

| 항목 | 내용 |
|------|------|
| 엔드포인트 | `GET /api/v1/retrospects/{retrospectId}` |
| HTTP 메서드 | GET |
| 인증 | Bearer 토큰 필수 (`Authorization` 헤더) |
| 설명 | 특정 회고 세션의 상세 정보(제목, 일시, 유형, 참여 멤버, 질문 리스트 및 전체 좋아요/댓글 통계)를 조회한다 |

## 응답 데이터

- `teamId` - 회고가 속한 팀(레트로룸) ID
- `title` - 프로젝트명
- `startTime` - 회고 시작 날짜 (YYYY-MM-DD)
- `retroCategory` - 회고 유형 (KPT, FOUR_L, FIVE_F, PMI, FREE)
- `members` - 참여 멤버 리스트 (등록일 기준 오름차순)
- `totalLikeCount` - 전체 좋아요 합계
- `totalCommentCount` - 전체 댓글 합계
- `questions` - 질문 리스트 (index 기준 오름차순, 최대 5개)

## 문서 목록

| 문서 | 설명 |
|------|------|
| [flow.md](./flow.md) | Handler -> Service -> DB 전체 동작 흐름 |
| [key-concepts.md](./key-concepts.md) | 이 API에서 사용된 Rust/Axum/SeaORM/chrono 핵심 패턴 |
| [keywords.md](./keywords.md) | 학습 키워드 정리 및 코드 위치 참조 |
| [spring-dev-guide.md](./spring-dev-guide.md) | Spring 개발자용 전체 학습 가이드 |

## 관련 소스 파일

| 파일 | 역할 |
|------|------|
| `codes/server/src/domain/retrospect/handler.rs` | HTTP 핸들러 (라인 276-298) |
| `codes/server/src/domain/retrospect/service.rs` | 비즈니스 로직 (라인 808-950) |
| `codes/server/src/domain/retrospect/dto.rs` | 요청/응답 DTO (라인 369-423) |
| `codes/server/src/domain/retrospect/entity/retrospect.rs` | 엔티티 모델 (DateTime=chrono::NaiveDateTime) |
| `docs/api-specs/012-retrospect-detail.md` | API 명세서 |
