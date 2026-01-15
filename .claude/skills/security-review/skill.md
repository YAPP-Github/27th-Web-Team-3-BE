# Security Review Skill (Rust Edition)

Rust 코드 보안 검토와 취약점 분석을 위한 스킬입니다.

## 언제 이 스킬을 사용하나요?

- 코드 리뷰 시 보안 점검
- PR 생성 전 보안 확인
- 새로운 기능 구현 후 보안 검토
- 정기 보안 감사

## Rust 보안 강점

Rust는 언어 차원에서 많은 보안 취약점을 방지합니다:
- 메모리 안전성 (buffer overflow 방지)
- 데이터 레이스 방지
- Null pointer dereference 방지 (Option 타입)
- Use-after-free 방지 (소유권 시스템)

## OWASP Top 10 for Rust

### 1. Injection (인젝션)

SQL Injection 방지 - SQLx의 prepared statements 사용

```rust
// 취약한 코드 (절대 사용 금지)
let query = format!("SELECT * FROM users WHERE id = {}", user_id);

// 안전한 코드 - SQLx bind 사용
let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
    .bind(user_id)
    .fetch_one(&pool)
    .await?;

// 안전한 코드 - query! 매크로 (컴파일 타임 검증)
let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
    .fetch_one(&pool)
    .await?;
```

### 2. Broken Authentication (인증 실패)

```rust
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

// 비밀번호 해싱
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    Ok(argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

// 비밀번호 검증
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hash)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
```

### 3. Sensitive Data Exposure (민감 데이터 노출)

```rust
use secrecy::{ExposeSecret, Secret};
use tracing::instrument;

// Secret 타입으로 민감 정보 래핑
pub struct Config {
    pub openai_api_key: Secret<String>,
    pub ai_secret_key: Secret<String>,
}

// 로깅 시 민감 정보 제외
#[instrument(skip(secret_key))] // secret_key 로깅 제외
pub async fn validate_key(&self, secret_key: &str) -> Result<(), AppError> {
    // ...
}

// 나쁜 예: 로그에 민감 정보
tracing::info!("API Key: {}", api_key); // 위험!

// 좋은 예: 민감 정보 마스킹
tracing::info!("API Key configured: {}", !api_key.is_empty());
```

### 4. Security Misconfiguration (보안 설정 오류)

```rust
use tower_http::cors::{Any, CorsLayer};
use std::time::Duration;

// 개발 환경용 CORS (주의!)
fn dev_cors() -> CorsLayer {
    CorsLayer::permissive()
}

// 프로덕션 환경용 CORS
fn prod_cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(["https://yourdomain.com".parse().unwrap()])
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION])
        .max_age(Duration::from_secs(3600))
}

// 환경에 따른 CORS 설정
let cors = if cfg!(debug_assertions) {
    dev_cors()
} else {
    prod_cors()
};
```

### 5. Broken Access Control (접근 제어 실패)

```rust
use axum::{
    extract::{State, Path},
    middleware::from_fn_with_state,
};

// 미들웨어로 Secret Key 검증
async fn validate_secret_key(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next<Body>,
) -> Result<Response, AppError> {
    let secret_key = request
        .headers()
        .get("X-Secret-Key")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::InvalidSecretKey)?;

    if secret_key != state.config.ai_secret_key.expose_secret() {
        return Err(AppError::InvalidSecretKey);
    }

    Ok(next.run(request).await)
}

// 라우터에 미들웨어 적용
let protected_routes = Router::new()
    .route("/api/ai/retrospective/guide", post(provide_guide))
    .route_layer(from_fn_with_state(state.clone(), validate_secret_key));
```

### 6. Using Components with Known Vulnerabilities

```bash
# cargo-audit으로 취약점 검사
cargo install cargo-audit
cargo audit

# Cargo.lock 기반 취약점 검사
cargo audit --file Cargo.lock

# CI/CD에 통합
# .github/workflows/security.yml
# - name: Security audit
#   run: cargo audit --deny warnings
```

### 7. Insufficient Logging & Monitoring

```rust
use tracing::{info, warn, error, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// 구조화된 로깅 설정
tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer().json())
    .with(
        tracing_subscriber::filter::Targets::new()
            .with_target("tower_http", Level::DEBUG)
            .with_target("your_app", Level::INFO)
    )
    .init();

// 보안 이벤트 로깅
pub async fn validate_secret_key(&self, provided_key: &str) -> Result<(), AppError> {
    if provided_key != self.expected_key.expose_secret() {
        warn!(
            event = "invalid_secret_key_attempt",
            "Invalid secret key attempt detected"
        );
        return Err(AppError::InvalidSecretKey);
    }

    info!(event = "secret_key_validated", "Secret key validated successfully");
    Ok(())
}
```

## Rust 특화 보안 체크리스트

### 메모리 안전성
- [ ] `unsafe` 블록이 필요한 경우 안전성 검증되었는가?
- [ ] `unwrap()`, `expect()`가 패닉 가능한 경로에 없는가?
- [ ] 순환 참조로 인한 메모리 누수가 없는가? (`Rc` -> `Weak`)

### 입력 검증
- [ ] 모든 외부 입력이 검증되는가? (`validator` crate)
- [ ] SQL 쿼리가 prepared statement를 사용하는가?
- [ ] 파일 경로 입력에 path traversal 방지가 있는가?

### 인증/인가
- [ ] 비밀번호가 안전하게 해싱되는가? (argon2, bcrypt)
- [ ] API 키가 환경 변수로 관리되는가?
- [ ] 민감한 엔드포인트에 인증 미들웨어가 적용되는가?

### 데이터 보호
- [ ] 민감 데이터가 로그에 노출되지 않는가?
- [ ] `Secret<T>` 타입으로 민감 정보가 보호되는가?
- [ ] TLS가 프로덕션에서 활성화되는가?

### 의존성
- [ ] `cargo audit` 결과 취약점이 없는가?
- [ ] 불필요한 의존성이 제거되었는가?
- [ ] 의존성 버전이 적절히 관리되는가? (Cargo.lock)

## 보안 도구

```toml
# Cargo.toml - 보안 관련 crate
[dependencies]
secrecy = "0.8"           # 민감 정보 보호
argon2 = "0.5"            # 비밀번호 해싱
validator = "0.16"        # 입력 검증
tower-http = "0.4"        # CORS, 헤더 보안

[dev-dependencies]
cargo-audit = "0.17"      # 취약점 검사
```

## CI/CD 보안 파이프라인

```yaml
# .github/workflows/security.yml
name: Security

on: [push, pull_request]

jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install cargo-audit
        run: cargo install cargo-audit

      - name: Security audit
        run: cargo audit --deny warnings

      - name: Check for unsafe code
        run: |
          if grep -r "unsafe" src/; then
            echo "Warning: unsafe code found"
            # 필요시 실패 처리
          fi
```
