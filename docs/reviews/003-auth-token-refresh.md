# API-003 토큰 갱신 Implementation Review

## 1. 개요
- **API 명**: `POST /api/v1/auth/token/refresh`
- **구현 목적**: 만료된 Access Token을 Refresh Token을 이용하여 재발급, Refresh Token Rotation으로 보안 강화
- **API 스펙**: `docs/api-specs/003-auth-token-refresh.md`

## 2. 구현 상세

### 2.1 도메인 구조
`codes/server/src/domain/auth/`
- `dto.rs`: `TokenRefreshRequest`, `TokenRefreshResponse`, `SuccessTokenRefreshResponse` (Swagger용)
- `service.rs`: Refresh Token 검증, 새 토큰 발급
- `handler.rs`: HTTP 핸들러 (`refresh_token`) + utoipa 문서화

### 2.2 주요 로직
1. **입력 검증**:
   - `refreshToken`: 필수 (min length 1)
2. **Refresh Token 검증**:
   - JWT 디코딩 및 유효성 검사
   - 토큰 타입 확인 (`token_type == "refresh"`)
3. **회원 존재 여부 확인**:
   - `sub` 클레임에서 `member_id` 추출
   - DB에서 회원 조회
4. **새 토큰 발급 (Refresh Token Rotation)**:
   - 새 `accessToken` 발급 (30분)
   - 새 `refreshToken` 발급 (14일)

### 2.3 토큰 유효기간 (TTL)

| 토큰 타입 | 유효기간 | 설명 |
|----------|---------|------|
| accessToken | 30분 | API 요청 시 인증에 사용 |
| refreshToken | 14일 | accessToken 재발급에 사용 |

### 2.4 에러 코드

| Code | HTTP | Description |
|------|------|-------------|
| COMMON400 | 400 | 필수 파라미터 누락 |
| AUTH4004 | 401 | 유효하지 않거나 만료된 Refresh Token |
| AUTH4005 | 401 | 로그아웃 처리된 토큰 |
| COMMON500 | 500 | 서버 내부 오류 |

### 2.5 라우트 등록
- `main.rs`에서 `POST /api/v1/auth/token/refresh` 등록
- OpenAPI 스키마 등록

## 3. 테스트 결과

### 3.1 테스트 케이스
`codes/server/tests/auth_test.rs`
- `should_refresh_tokens_successfully`: 토큰 갱신 성공
- `should_return_400_when_refresh_token_missing`: refreshToken 누락 시 400
- `should_return_401_for_expired_refresh_token`: 만료된 토큰 시 401
- `should_return_401_for_logged_out_token`: 로그아웃된 토큰 시 401

### 3.2 DTO 직렬화 테스트
- `should_serialize_token_refresh_request_with_camel_case`: camelCase 변환 확인
- `should_serialize_token_refresh_response_with_camel_case`: 응답 필드 확인

### 3.3 코드 품질
- `cargo fmt` 적용 완료
- `cargo clippy -- -D warnings` 경고 없음
- `cargo test` 24개 테스트 통과

## 4. 변경 파일 목록

| 파일 | 변경 내용 |
|------|----------|
| `src/domain/auth/dto.rs` | `TokenRefreshRequest`, `TokenRefreshResponse`, `SuccessTokenRefreshResponse` 추가 |
| `src/domain/auth/handler.rs` | `refresh_token` 핸들러 추가 |
| `src/domain/auth/service.rs` | `refresh_token` 메서드 추가 |
| `src/utils/error.rs` | `InvalidRefreshToken` (AUTH4004), `LoggedOutToken` (AUTH4005) 에러 타입 추가 |
| `src/main.rs` | 새 라우트 등록, OpenAPI 스키마 등록 |
| `tests/auth_test.rs` | 토큰 갱신 테스트 케이스 추가 |

## 5. 코드 리뷰 체크리스트

- [x] API 스펙에 맞게 구현되었는가?
- [x] Request 필드명이 camelCase로 직렬화되는가? (`refreshToken`)
- [x] Refresh Token Rotation 정책이 적용되었는가?
- [x] 에러 처리가 적절하게 되어 있는가? (Result + ? 패턴)
- [x] Swagger 문서가 등록되었는가?
- [x] `cargo test`, `cargo clippy`, `cargo fmt` 모두 통과하는가?
- [x] `unwrap()` / `expect()` 미사용 (프로덕션 코드)
- [x] `serde(rename_all = "camelCase")` 적용
- [x] 공통 유틸리티 (`BaseResponse`, `AppError`) 재사용

## 6. 참고사항

- 현재 Refresh Token 블랙리스트 기능은 미구현 상태
- TODO: Redis 또는 DB 테이블을 활용한 토큰 무효화 기능 추가 필요
