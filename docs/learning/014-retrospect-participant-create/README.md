# API-014: 회고 참여자 등록

## 개요

진행 예정인 회고에 참석자로 등록하는 API입니다.

- **엔드포인트**: `POST /api/v1/retrospects/{retrospectId}/participants`
- **인증**: Bearer JWT 토큰 (Request Body 없음)
- **핵심 동작**: JWT에서 추출한 유저 정보를 기반으로 회고 참석을 처리합니다.

## 관련 소스 파일

| 파일 | 역할 |
|------|------|
| `codes/server/src/domain/retrospect/handler.rs` | HTTP 핸들러 (라인 112~157) |
| `codes/server/src/domain/retrospect/service.rs` | 비즈니스 로직 (라인 354~427) |
| `codes/server/src/domain/retrospect/dto.rs` | 응답 DTO (라인 220~244) |
| `codes/server/src/domain/member/entity/member_retro.rs` | member_retro 엔티티 정의 |
| `codes/server/src/utils/error.rs` | 에러 타입 정의 (라인 72~76) |
| `docs/api-specs/014-retrospect-participant-create.md` | API 명세서 |

## 비즈니스 규칙 요약

1. 해당 회고가 속한 팀의 멤버만 참석 가능
2. 이미 시작되었거나 종료된 회고에는 참석 불가 (시간 기반 검증)
3. 동일 유저의 동일 회고 중복 참석 불가 (애플리케이션 레벨 + DB 유니크 제약)
4. 성공 시 `member_retro` 테이블에 새 레코드 생성

## 에러 응답 코드

| 코드 | HTTP 상태 | 설명 |
|------|-----------|------|
| COMMON400 | 400 | retrospectId가 0 이하 |
| RETRO4002 | 400 | 과거/진행중 회고 참석 시도 |
| AUTH4001 | 401 | 인증 실패 |
| TEAM4031 | 403 | 팀 멤버 아님 |
| RETRO4041 | 404 | 존재하지 않는 회고 |
| RETRO4091 | 409 | 중복 참석 |
| COMMON500 | 500 | 서버 내부 오류 |

## 학습 문서 목차

- [flow.md](./flow.md) - 동작 흐름
- [key-concepts.md](./key-concepts.md) - 핵심 개념
- [keywords.md](./keywords.md) - 학습 키워드
- [spring-guide.md](./spring-guide.md) - Spring 개발자를 위한 가이드
