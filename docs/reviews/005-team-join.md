# API-005 Team Join Implementation Review

## 개요
- **API**: `POST /api/v1/teams/join`
- **기능**: 초대 코드를 통한 팀(Retro Room) 합류
- **담당자**: Gemini Agent
- **작성일**: 2026-01-26

## 구현 상세

### 1. DTO 설계 (`domain/retrospect/dto.rs`)
- **Request**: `JoinTeamRequest`
    - `inviteUrl`: URL 형식 유효성 검사 (`validator` crate)
- **Response**: `JoinTeamResponse`
    - `teamId`, `teamName`, `joinedAt` 반환

### 2. 비즈니스 로직 (`domain/retrospect/service.rs`)
- `join_team` 메서드 구현
- **초대 코드 추출**: URL 경로 또는 쿼리 파라미터에서 코드 추출 로직 구현
- **유효성 검사**:
    - `retro_room` 테이블에서 `invition_url`(초대 코드)로 조회
    - 생성일 기준 7일 만료 체크
    - `member_retro_room` 테이블 조회로 중복 가입 방지
- **데이터 처리**:
    - `member_retro_room` 테이블에 `MEMBER` 권한으로 레코드 생성

### 3. 에러 처리 (`utils/error.rs`)
새로운 에러 코드 추가:
- `TEAM4002` (InvalidInviteLink): 초대 코드 추출 실패
- `TEAM4003` (InviteLinkExpired): 7일 만료
- `TEAM4041` (TeamNotFound): 팀 존재하지 않음
- `TEAM4092` (TeamAlreadyJoined): 이미 멤버임

### 4. 특이 사항
- 사용자의 요청("member_retro.rs를 연결 테이블로 사용")에 대해 검토했으나, `retro_room` (Team)과의 연결 테이블은 `member_retro_room`이므로 이를 사용하여 구현함. (`member_retro`는 특정 회고(Retrospect)와의 연결임)

## 테스트 결과
- `cargo check` 통과 확인 필요
