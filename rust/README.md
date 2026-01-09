# Rust 인증 서버

Rust와 Axum을 기반으로 한 JWT 인증 시스템입니다.

## 🚀 빠른 시작

### 1. 환경 설정
`.env` 파일이 이미 준비되어 있습니다:
```env
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
SECRET_KEY=test_secret_key_123
RUST_LOG=info
```

### 2. 서버 실행
```bash
# 개발 모드로 실행
cargo run

# 또는 릴리즈 빌드로 실행
cargo build --release
./target/release/rust-server
```

서버가 `http://127.0.0.1:8080`에서 실행됩니다.

### 3. 테스트 실행
```bash
# 모든 테스트 실행 (29개 테스트)
cargo test

# 특정 테스트만 실행
cargo test auth::
cargo test common::
```

## 📚 API 엔드포인트

### 회원가입
```bash
POST /api/auth/register
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "password123",
  "name": "홍길동"
}
```

### 로그인
```bash
POST /api/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "password123"
}
```

### 사용자 정보 조회 (인증 필요)
```bash
GET /api/auth/me
Authorization: Bearer {access_token}
```

### 헬스 체크
```bash
GET /health
```

## 📖 상세 문서

모든 API 명세, 에러 처리, 사용 예제는 다음 문서를 참고하세요:
- **[인증 API 문서](docs/인증API.md)** - 전체 API 명세 및 구현 상세

## 🏗️ 프로젝트 구조

```
src/
├── main.rs              # 애플리케이션 진입점
├── common/              # 공통 모듈
│   ├── response.rs      # API 응답 구조
│   └── error.rs         # 에러 처리
└── auth/                # 인증 도메인
    ├── jwt.rs           # JWT 토큰
    ├── models.rs        # 데이터 모델
    ├── service.rs       # 비즈니스 로직
    ├── handlers.rs      # HTTP 핸들러
    ├── middleware.rs    # 인증 미들웨어
    └── routes.rs        # 라우트 정의
```

## ✅ 구현된 기능

- ✅ JWT 기반 인증 (Access Token + Refresh Token)
- ✅ bcrypt 비밀번호 해싱
- ✅ 공통 응답 구조체
- ✅ 체계적인 에러 처리
- ✅ 유효성 검사 (이메일, 비밀번호)
- ✅ 인증 미들웨어
- ✅ 29개 단위 테스트

## 🔧 개발 도구

```bash
# 코드 포맷팅
cargo fmt

# 린트 체크
cargo clippy

# 빌드
cargo build
```

## 📝 코딩 규칙

프로젝트는 `CLAUDE.md`의 규칙을 따릅니다:
- 도메인형 폴더 구조
- Result와 anyhow를 통한 에러 처리
- TDD 방식의 테스트 우선 작성
- clippy 린터 규칙 준수

## ⚠️ 현재 제한사항

- 사용자 데이터는 메모리에 저장됩니다 (프로덕션에서는 DB 필요)
- Refresh Token 갱신 로직 미구현

## 🔐 보안

- bcrypt (cost 12)를 통한 안전한 비밀번호 해싱
- JWT Secret Key는 환경 변수로 관리
- 비밀번호 평문 노출 방지

---

**작성일:** 2026-01-10

