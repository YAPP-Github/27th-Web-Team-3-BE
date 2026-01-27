# API-002 회원가입 Implementation Review

## 1. 개요
- **API 명**: `POST /api/v1/auth/signup`
- **구현 목적**: 소셜 로그인 단계에서 획득한 signupToken과 사용자가 입력한 닉네임을 전달하여 회원가입 완료
- **API 스펙**: `docs/api-specs/002-auth-signup.md`

## 2. 구현 상세

### 2.1 도메인 구조
`codes/server/src/domain/auth/`
- `dto.rs`: `SignupRequest`, `SignupResponse`, `SuccessSignupResponse` (Swagger용)
- `service.rs`: signupToken 검증, 닉네임 중복 확인, 회원 생성, JWT 발급
- `handler.rs`: HTTP 핸들러 (`signup`) + utoipa 문서화

### 2.2 주요 로직
1. **입력 검증**:
   - `email`: 이메일 형식 검증
   - `nickname`: 1~20자 길이 검증
2. **인증**:
   - Authorization 헤더에서 Bearer 토큰 추출
   - signupToken 디코딩 및 검증
   - 토큰 타입 확인 (`token_type: "signup"`)
   - 토큰 내 이메일과 요청 이메일 일치 확인
3. **닉네임 중복 확인**:
   - DB에서 동일 닉네임 존재 여부 확인
4. **회원 생성**:
   - Member 엔티티에 email, nickname, social_type 저장
5. **토큰 발급**:
   - accessToken, refreshToken 발급 후 반환

### 2.3 에러 코드

| Code | HTTP | Description |
|------|------|-------------|
| COMMON400 | 400 | 유효성 검증 실패 (닉네임 길이) |
| AUTH4001 | 401 | 인증 실패 (signupToken 누락/만료/잘못된 타입) |
| MEMBER4041 | 404 | 유효하지 않은 회원가입 정보 |
| MEMBER4091 | 409 | 닉네임 중복 |
| COMMON500 | 500 | 서버 내부 오류 |

### 2.4 Member 엔티티 변경
- `nickname` 필드 추가 (`Option<String>`)
- 회원가입 시 닉네임 설정, 기존 회원은 null 허용

### 2.5 라우트 등록
- `main.rs`에서 `POST /api/v1/auth/signup` 등록
- OpenAPI paths 및 schemas에 등록 완료

## 3. 테스트 결과

### 3.1 테스트 케이스
`codes/server/tests/auth_test.rs`
- `should_complete_signup_successfully`: 회원가입 성공
- `should_return_400_for_empty_nickname`: 빈 닉네임 시 400
- `should_return_409_for_duplicate_nickname`: 닉네임 중복 시 409
- `should_return_401_when_signup_token_missing`: signupToken 누락 시 401
- `should_return_401_for_expired_signup_token`: 만료된 토큰 시 401

### 3.2 DTO 직렬화 테스트
- `should_serialize_signup_request_with_camel_case`: camelCase 변환 확인
- `should_serialize_signup_response_with_camel_case`: 응답 필드 확인

### 3.3 코드 품질
- `cargo fmt` 적용 완료
- `cargo clippy -- -D warnings` 경고 없음
- `cargo test` 14개 테스트 통과

## 4. 변경 파일 목록

| 파일 | 변경 내용 |
|------|----------|
| `src/domain/auth/dto.rs` | `SignupRequest`, `SignupResponse` 추가 |
| `src/domain/auth/handler.rs` | `signup` 핸들러 추가 |
| `src/domain/auth/service.rs` | `signup` 메서드 추가 |
| `src/domain/member/entity/member.rs` | `nickname` 필드 추가 |
| `src/utils/error.rs` | `Conflict` (MEMBER4091), `NotFound` (MEMBER4041) 에러 타입 추가 |
| `src/main.rs` | 새 라우트 등록, OpenAPI 스키마 등록 |
| `tests/auth_test.rs` | 회원가입 테스트 케이스 추가 |

## 5. 코드 리뷰 체크리스트

- [x] API 스펙에 맞게 구현되었는가?
- [x] signupToken 검증이 올바르게 동작하는가?
- [x] 닉네임 중복 체크가 구현되었는가?
- [x] 에러 처리가 적절하게 되어 있는가? (Result + ? 패턴)
- [x] Swagger 문서가 등록되었는가?
- [x] `cargo test`, `cargo clippy`, `cargo fmt` 모두 통과하는가?
- [x] `unwrap()` / `expect()` 미사용 (프로덕션 코드)
- [x] `serde(rename_all = "camelCase")` 적용
- [x] 공통 유틸리티 (`BaseResponse`, `AppError`) 재사용

## 6. 향후 개선 사항

- [ ] signupToken에 소셜 provider 정보 포함 (현재 Kakao로 하드코딩)
- [ ] 닉네임 특수문자 검증 추가 (현재 길이만 검증)
