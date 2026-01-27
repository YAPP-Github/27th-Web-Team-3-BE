# API-001 소셜 로그인 Implementation Review

## 1. 개요
- **API 명**: `POST /api/v1/auth/social-login`
- **구현 목적**: 구글/카카오 Access Token을 전달받아 사용자 식별 정보를 확인하고, 기존 회원은 서비스 토큰 발급, 신규 회원은 회원가입 유도
- **API 스펙**: `docs/api-specs/001-auth-social-login.md`

## 2. 구현 상세

### 2.1 도메인 구조
`codes/server/src/domain/auth/`
- `dto.rs`: `SocialLoginRequest`, `SocialLoginResponse`, `SuccessSocialLoginResponse` (Swagger용)
- `service.rs`: 소셜 API 호출, 회원 조회, JWT 토큰 발급
- `handler.rs`: HTTP 핸들러 (`social_login`) + utoipa 문서화

### 2.2 주요 로직
1. **입력 검증**:
   - `provider`: GOOGLE 또는 KAKAO (Enum)
   - `accessToken`: 필수 (min length 1)
2. **소셜 API 호출**:
   - Kakao: `https://kapi.kakao.com/v2/user/me`
   - Google: `https://www.googleapis.com/oauth2/v2/userinfo`
   - Bearer 토큰으로 사용자 이메일 추출
3. **회원 조회**:
   - 이메일 + 소셜 타입으로 DB 조회
4. **응답 분기**:
   - **기존 회원**: `accessToken`, `refreshToken` 발급 (code: `COMMON200`)
   - **신규 회원**: `signupToken`, `email` 반환 (code: `AUTH2001`)

### 2.3 토큰 유효기간 (TTL)

| 토큰 타입 | 유효기간 | 설명 |
|----------|---------|------|
| accessToken | 30분 | API 요청 시 인증에 사용 |
| refreshToken | 14일 | accessToken 재발급에 사용 |
| signupToken | 10분 | 신규 회원의 회원가입 API 호출 시 사용 |

### 2.4 에러 코드

| Code | HTTP | Description |
|------|------|-------------|
| COMMON400 | 400 | 필수 파라미터 누락 |
| AUTH4002 | 401 | 유효하지 않은 소셜 토큰 |
| COMMON500 | 500 | 서버 내부 오류 |

### 2.5 라우트 등록
- `main.rs`에서 `POST /api/v1/auth/social-login` 등록
- 하위 호환성을 위해 구 엔드포인트 `/api/auth/login` 유지 (deprecated)

## 3. 테스트 결과

### 3.1 테스트 케이스
`codes/server/tests/auth_test.rs`
- `should_return_tokens_for_existing_member`: 기존 회원 로그인 성공
- `should_return_signup_token_for_new_member`: 신규 회원 signupToken 발급
- `should_return_400_when_provider_missing`: provider 누락 시 400
- `should_return_401_for_invalid_social_token`: 유효하지 않은 토큰 시 401

### 3.2 DTO 직렬화 테스트
- `should_serialize_social_login_request_with_camel_case`: camelCase 변환 확인
- `should_serialize_existing_member_response`: 기존 회원 응답 필드 확인
- `should_serialize_new_member_response`: 신규 회원 응답 필드 확인

### 3.3 코드 품질
- `cargo fmt` 적용 완료
- `cargo clippy -- -D warnings` 경고 없음
- `cargo test` 14개 테스트 통과

## 4. 변경 파일 목록

| 파일 | 변경 내용 |
|------|----------|
| `src/domain/auth/dto.rs` | `SocialLoginRequest`, `SocialLoginResponse` 추가, 필드명 변경 |
| `src/domain/auth/handler.rs` | `social_login` 핸들러 추가 |
| `src/domain/auth/service.rs` | `social_login` 메서드 추가, `fetch_kakao_user_info`, `fetch_google_user_info` |
| `src/utils/error.rs` | `SocialAuthFailed` (AUTH4002) 에러 타입 추가 |
| `src/main.rs` | 새 라우트 등록, OpenAPI 스키마 등록 |
| `tests/auth_test.rs` | 소셜 로그인 테스트 케이스 추가 |

## 5. 코드 리뷰 체크리스트

- [x] API 스펙에 맞게 구현되었는가?
- [x] Request 필드명이 camelCase로 직렬화되는가? (`provider`, `accessToken`)
- [x] 기존 회원/신규 회원 응답 분기가 올바른가?
- [x] 에러 처리가 적절하게 되어 있는가? (Result + ? 패턴)
- [x] Swagger 문서가 등록되었는가?
- [x] `cargo test`, `cargo clippy`, `cargo fmt` 모두 통과하는가?
- [x] `unwrap()` / `expect()` 미사용 (프로덕션 코드)
- [x] `serde(rename_all = "camelCase")` 적용
- [x] 공통 유틸리티 (`BaseResponse`, `AppError`) 재사용
