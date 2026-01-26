# Team Create API Implementation Review

## 1. 개요
- **API 명**: `POST /api/v1/teams`
- **구현 목적**: 새로운 회고 팀(방)을 생성하고, 생성한 유저에게 관리자(Owner) 권한을 부여하며 초대 코드를 발급한다.
- **담당자**: Gemini Agent

## 2. 요구사항 및 구현 상세

### 2.1 요구사항 분석
- **팀 생성**: 팀 이름(필수, 20자 이내)과 설명(선택, 50자 이내)을 입력받아 생성.
- **초대 코드**: 팀 생성 시 8자리의 고유 초대 코드(`INV-XXXX-XXXX`) 자동 생성.
- **권한 부여**: 생성 요청을 보낸 유저는 자동으로 해당 팀의 `OWNER`가 되어야 함.
- **중복 검사**: 동일한 팀 이름으로 생성을 시도할 경우 `409 Conflict` 에러 반환.
- **유효성 검사**: 입력값 길이 제한 검증.

### 2.2 구현 상세 (수정한 부분)

#### Database Schema
- **`retro_room.rs`**:
    - `description` 필드 추가 (팀 설명).
- **`member_retro_room.rs`**:
    - `role` 필드 추가 (`RoomRole` Enum: `OWNER`, `MEMBER`).
    - 팀과 멤버의 관계 및 권한을 관리하기 위함.

#### Domain Logic (`codes/server/src/domain/retrospect/`)
- **`dto.rs`**:
    - `TeamCreateRequest`: `validator`를 통한 길이 검증 적용.
    - `SuccessTeamCreateResponse`: Swagger 문서화를 위해 `BaseResponse` 구조 반영.
- **`service.rs`**:
    - `create_team`: 트랜잭션 단위 로직 구현 (현재는 단일 연결 사용).
        1.  팀 이름 중복 조회.
        2.  초대 코드 생성 (`INV-` + 랜덤 문자열 알고리즘).
        3.  `retro_room` insert.
        4.  `member_retro_room` insert (Role: `OWNER`).
- **`handler.rs`**:
    - JWT에서 추출한 `AuthUser`를 통해 요청자 ID 식별.
    - `RetrospectService` 호출 및 에러 매핑.

#### Error Handling (`utils/error.rs`)
- **에러 코드 추가**:
    - `TEAM4001`: 팀 이름 길이 초과 (Bad Request).
    - `TEAM4091`: 팀 이름 중복 (Conflict).
- **HTTP Status**: `409 Conflict` 지원 추가.

## 3. 테스트 결과

### 3.1 빌드 및 정적 분석
- `cargo check`: 통과.
- Unused Import Warning 정리 완료 (매크로 관련 제외).

### 3.2 기능 검증 포인트
1.  **성공 케이스**:
    - 유효한 이름과 설명으로 요청 시 200 OK.
    - `result`에 `teamId`, `teamName`, `inviteCode` 포함 확인.
    - DB에 `retro_room` 및 `member_retro_room` 데이터 적재 확인.
2.  **실패 케이스**:
    - **중복 이름**: 이미 존재하는 팀 이름으로 요청 시 409 Conflict (`TEAM4091`).
    - **길이 초과**: 20자 넘는 이름 요청 시 400 Bad Request (`TEAM4001`).
    - **인증 실패**: 토큰 없이 요청 시 401 Unauthorized.

## 4. 코드 리뷰 체크리스트 (Self-Check)

- [x] **API 명세 준수**: `004-team-create.md`의 요청/응답 형식을 정확히 구현했는가?
- [x] **데이터 무결성**: 팀 생성과 동시에 관리자 권한이 부여되는가?
- [x] **보안**: 인증된 사용자만 접근 가능한가? (Bearer Token)
- [x] **컨벤션**: `camelCase` 응답, `BaseResponse` 포맷, 에러 코드 체계를 따랐는가?
- [x] **확장성**: 추후 멤버 초대 및 역할 관리를 위한 `RoomRole` 확장이 용이한가?