# API-004 로그아웃 Implementation Review

## 1. 개요
- **API 명**: `POST /api/v1/auth/logout`
- **구현 목적**: 현재 사용자의 로그아웃 처리, Refresh Token 무효화로 보안 유지
- **API 스펙**: `docs/api-specs/004-auth-logout.md`

## 2. 구현 상세

### 2.1 도메인 구조
`codes/server/src/domain/auth/`
- `dto.rs`: `LogoutRequest`, `SuccessLogoutResponse` (Swagger용)
- `service.rs`: Refresh Token 검증 및 무효화
- `handler.rs`: HTTP 핸들러 (`logout`) + utoipa 문서화

### 2.2 주요 로직
1. **인증 확인**:
   - Authorization 헤더에서 Bearer accessToken 검증
   - `AuthUser` extractor로 user_id 추출
2. **입력 검증**:
   - `refreshToken`: 필수 (min length 1)
3. **Refresh Token 검증**:
   - JWT 디코딩 및 유효성 검사
   - 토큰 타입 확인 (`token_type == "refresh"`)
4. **토큰 무효화**:
   - 현재는 JWT 검증만 수행
   - TODO: 실제 토큰 블랙리스트 구현 필요

### 2.3 토큰 유효기간 (TTL)

| 토큰 타입 | 유효기간 | 설명 |
|----------|---------|------|
| accessToken | 30분 | API 요청 시 인증에 사용 |
| refreshToken | 14일 | accessToken 재발급에 사용, 로그아웃 시 무효화 대상 |

### 2.4 에러 코드

| Code | HTTP | Description |
|------|------|-------------|
| AUTH4001 | 401 | 인증 정보가 유효하지 않음 (accessToken 누락/만료) |
| AUTH4003 | 400 | 이미 로그아웃되었거나 유효하지 않은 토큰 |
| COMMON500 | 500 | 서버 내부 오류 |

### 2.5 라우트 등록
- `main.rs`에서 `POST /api/v1/auth/logout` 등록
- OpenAPI 스키마 등록
- `security(("bearer_auth" = []))` 설정으로 인증 필요 명시

## 3. 테스트 결과

### 3.1 테스트 케이스
`codes/server/tests/auth_test.rs`
- `should_logout_successfully`: 로그아웃 성공
- `should_return_401_when_access_token_missing`: accessToken 누락 시 401
- `should_return_400_for_invalid_refresh_token`: 유효하지 않은 refreshToken 시 400

### 3.2 DTO 직렬화 테스트
- `should_serialize_logout_request_with_camel_case`: camelCase 변환 확인

### 3.3 코드 품질
- `cargo fmt` 적용 완료
- `cargo clippy -- -D warnings` 경고 없음
- `cargo test` 24개 테스트 통과

## 4. 변경 파일 목록

| 파일 | 변경 내용 |
|------|----------|
| `src/domain/auth/dto.rs` | `LogoutRequest`, `SuccessLogoutResponse` 추가 |
| `src/domain/auth/handler.rs` | `logout` 핸들러 추가 |
| `src/domain/auth/service.rs` | `logout` 메서드 추가 |
| `src/utils/error.rs` | `InvalidToken` (AUTH4003) 에러 타입 추가 |
| `src/main.rs` | 새 라우트 등록, OpenAPI 스키마 등록 |
| `tests/auth_test.rs` | 로그아웃 테스트 케이스 추가 |

## 5. 코드 리뷰 체크리스트

- [x] API 스펙에 맞게 구현되었는가?
- [x] Request 필드명이 camelCase로 직렬화되는가? (`refreshToken`)
- [x] 인증 요구사항이 적용되었는가? (Bearer accessToken)
- [x] 에러 처리가 적절하게 되어 있는가? (Result + ? 패턴)
- [x] Swagger 문서가 등록되었는가?
- [x] `cargo test`, `cargo clippy`, `cargo fmt` 모두 통과하는가?
- [x] `unwrap()` / `expect()` 미사용 (프로덕션 코드)
- [x] `serde(rename_all = "camelCase")` 적용
- [x] 공통 유틸리티 (`BaseResponse`, `AppError`, `AuthUser`) 재사용

## 6. 참고사항

- 다중 기기 로그인 지원을 위해 `refreshToken`을 Body에 포함
- 현재 Refresh Token 블랙리스트 기능은 미구현 상태
- TODO: Redis 또는 DB 테이블을 활용한 토큰 무효화 기능 추가 필요
