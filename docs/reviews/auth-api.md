# Auth API Implementation Review

## 1. 개요
- **API 명**: `POST /api/auth/login`
- **구현 목적**: Google 및 Kakao 소셜 로그인을 통해 JWT 토큰을 발급하고, 신규 유저의 경우 회원가입을 자동으로 처리한다.
- **담당자**: Gemini Agent

## 2. 구현 상세

### 2.1 도메인 구조
`codes/server/src/domain/auth/`
- `dto.rs`: `LoginRequest` (소셜 타입, 토큰), `LoginResponse` (JWT)
- `service.rs`: 소셜 API 통신, 유저 조회/생성 로직
- `handler.rs`: 요청 검증 및 응답 처리
- `mod.rs`: 모듈 노출

### 2.2 주요 로직
1. **요청 수신**: 클라이언트로부터 `SocialType` (KAKAO/GOOGLE)과 `token` (Access Token)을 수신.
2. **소셜 정보 조회**:
    - `reqwest`를 사용하여 각 소셜 제공자의 UserInfo API 호출.
    - 이메일 정보를 추출.
3. **유저 식별 및 가입**:
    - DB에서 `(email, social_type)` 조합으로 유저 조회.
    - 존재하면 해당 유저 정보 반환.
    - 존재하지 않으면 신규 Member 생성 (회원가입).
4. **JWT 발급**:
    - 유저 ID(`sub`), 발급 시간(`iat`), 만료 시간(`exp`)을 포함한 JWT Access Token 생성.
    - `AppConfig`의 `jwt_secret`과 `jwt_expiration` 사용.

### 2.3 보안 및 설정
- **JWT**: `jsonwebtoken` 크레이트 사용 (HS256).
- **환경 변수**:
    - `JWT_SECRET`: 토큰 서명 키.
    - `JWT_EXPIRATION`: 토큰 유효 기간 (초 단위).
    - `GOOGLE_...`, `KAKAO_...`: 소셜 연동 설정 (확장성 고려).
- **에러 처리**:
    - `AppError`를 확장하여 `Unauthorized`(401), `Forbidden`(403) 추가.
    - 소셜 API 호출 실패, 파싱 실패, DB 에러 등을 적절한 HTTP 상태 코드로 매핑.

## 3. 테스트 결과

### 3.1 유닛 테스트
- `utils::jwt::tests`:
    - `test_encode_and_decode`: 토큰 생성 및 검증, Payload 일치 확인 (Pass).
    - `test_invalid_token`: 잘못된 토큰에 대한 검증 실패 확인 (Pass).

### 3.2 수동 검증 포인트 (Postman/Curl)
- `POST /api/auth/login` 호출 시:
    - 유효한 소셜 토큰 -> 200 OK + JWT 반환.
    - 유효하지 않은 토큰 -> 401 Unauthorized.
    - 지원하지 않는 소셜 타입 -> 400 Bad Request (JSON 파싱/Validation).

## 4. 코드 리뷰 체크리스트 (Self-Check)

- [x] **TDD 원칙 준수**: 유틸리티(`jwt`)에 대한 테스트 작성 후 구현. (통합 테스트는 추후 보강 필요)
- [x] **테스트 통과**: `cargo test utils::jwt` 통과.
- [x] **API 문서 작성**: `docs/reviews/auth-api.md` 작성 완료.
- [x] **공통 유틸리티 재사용**: `BaseResponse`, `AppError`, `AppState` 활용.
- [x] **에러 처리**: `unwrap` 사용 지양, `AppError`로 래핑하여 반환.
- [x] **Rust 컨벤션**: `cargo check`, `clippy` (warning 해결) 준수.
- [x] **의존성 관리**: `jsonwebtoken`, `reqwest` 등 필요한 최소한의 크레이트 추가.

## 5. 향후 개선 사항
- **Refresh Token**: 현재 Access Token만 발급하므로, 보안 강화를 위해 Refresh Token 도입 고려.
- **테스트 격리**: 외부 소셜 API에 의존하는 통합 테스트 작성을 위해 Mocking 전략 수립 필요.
- **Swagger**: `utoipa`를 통한 API 문서 구체화 (완료).
