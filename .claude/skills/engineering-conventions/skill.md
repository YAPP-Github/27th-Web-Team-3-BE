# Engineering Conventions Skill (Rust Edition)

프로젝트별 엔지니어링 컨벤션을 정의하고 적용하는 Rust 가이드입니다.

## 언제 이 스킬을 사용하나요?

- 새 Rust 프로젝트의 컨벤션 정립 시
- 기존 프로젝트에 표준 적용 시
- 팀 온보딩 시
- 코드 리뷰 시

## Rust 프로젝트 컨벤션

### 1. 프로젝트 구조

```
rust/
├── Cargo.toml
├── Cargo.lock             # 버전 고정 (커밋 필수)
├── .env.example           # 환경 변수 예시
├── src/
│   ├── main.rs            # 진입점
│   ├── lib.rs             # 라이브러리 루트 (선택)
│   ├── config.rs          # 환경 설정
│   ├── error.rs           # 에러 타입 정의
│   ├── response.rs        # 공통 응답 타입
│   ├── domain/            # 도메인별 모듈
│   │   ├── mod.rs
│   │   └── ai/
│   │       ├── mod.rs
│   │       ├── handler.rs
│   │       ├── service.rs
│   │       ├── dto.rs
│   │       └── prompt.rs
│   └── global/            # 전역 유틸리티
│       ├── mod.rs
│       └── middleware.rs
├── tests/                 # 통합 테스트
│   └── integration/
│       └── ai_test.rs
└── benches/               # 벤치마크 (선택)
    └── benchmark.rs
```

### 2. Cargo.toml 컨벤션

```toml
[package]
name = "web3-server"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"  # MSRV 명시

[dependencies]
# 웹 프레임워크
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# 직렬화
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# AI
async-openai = "0.18"

# 검증
validator = { version = "0.16", features = ["derive"] }

# 에러 처리
thiserror = "1"
anyhow = "1"

# 로깅
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json"] }

# 환경 설정
dotenvy = "0.15"

[dev-dependencies]
tokio-test = "0.4"
wiremock = "0.5"  # HTTP mocking

[profile.release]
lto = true
codegen-units = 1
strip = true
```

### 3. 코딩 스타일

```rust
// 모듈 선언 순서
mod config;
mod error;
mod response;
mod domain;
mod global;

// import 순서 (rustfmt가 자동 정렬)
// 1. std
use std::collections::HashMap;
use std::sync::Arc;

// 2. 외부 crate
use axum::{Router, routing::post};
use serde::{Deserialize, Serialize};

// 3. 내부 모듈
use crate::config::Config;
use crate::error::AppError;
```

### 4. Feature Flags 정책

```toml
# Cargo.toml
[features]
default = []
dev = ["mock-openai"]      # 개발용 목업
mock-openai = []
```

```rust
// 조건부 컴파일
#[cfg(feature = "mock-openai")]
pub fn create_mock_client() -> MockClient {
    // ...
}

#[cfg(not(feature = "mock-openai"))]
pub fn create_client(api_key: &str) -> Client {
    // ...
}
```

### 5. 성능 작업

```bash
# 벤치마크 실행
cargo bench

# 프로파일링 (flamegraph)
cargo install flamegraph
cargo flamegraph --bin web3-server

# Release 빌드 테스트
cargo build --release
./target/release/web3-server
```

### 6. 테스트 컨벤션

```rust
// 단위 테스트: 같은 파일 내
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name_describes_behavior() {
        // Arrange
        let input = create_test_input();

        // Act
        let result = function_under_test(input);

        // Assert
        assert!(result.is_ok());
    }
}

// 비동기 테스트
#[tokio::test]
async fn test_async_function() {
    // ...
}

// 통합 테스트: tests/ 폴더
// tests/integration/ai_test.rs
use web3_server::domain::ai::service::AiService;

#[tokio::test]
async fn test_guide_api_integration() {
    // ...
}
```

## Hygiene Before Commit

커밋 전 필수 체크리스트:

```bash
# 1. 포맷팅
cargo fmt

# 2. 린팅
cargo clippy -- -D warnings

# 3. 테스트
cargo test

# 4. 빌드 확인
cargo build

# 5. (선택) 보안 감사
cargo audit
```

### Git Hook 설정

```bash
# .git/hooks/pre-commit
#!/bin/sh
set -e

echo "Running cargo fmt..."
cargo fmt -- --check

echo "Running cargo clippy..."
cargo clippy -- -D warnings

echo "Running cargo test..."
cargo test

echo "All checks passed!"
```

## 환경 변수 관리

```bash
# .env.example (버전 관리됨)
OPENAI_API_KEY=sk-your-api-key-here
AI_SECRET_KEY=your-secret-key-here
DATABASE_URL=mysql://user:password@localhost/db
RUST_LOG=info

# .env (버전 관리 제외)
# .gitignore에 추가
```

```rust
// config.rs
use dotenvy::dotenv;
use std::env;

pub struct Config {
    pub openai_api_key: String,
    pub ai_secret_key: String,
    pub database_url: Option<String>,
    pub rust_log: String,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        dotenv().ok(); // .env 파일 로드 (없어도 OK)

        Ok(Self {
            openai_api_key: env::var("OPENAI_API_KEY")
                .map_err(|_| AppError::ConfigError("OPENAI_API_KEY not set"))?,
            ai_secret_key: env::var("AI_SECRET_KEY")
                .map_err(|_| AppError::ConfigError("AI_SECRET_KEY not set"))?,
            database_url: env::var("DATABASE_URL").ok(),
            rust_log: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        })
    }
}
```

## Docker 설정

```dockerfile
# Dockerfile
# Build stage
FROM rust:1.75-alpine AS builder

WORKDIR /app
RUN apk add --no-cache musl-dev

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage
FROM alpine:3.19

RUN apk add --no-cache ca-certificates

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/web3-server /usr/local/bin/

EXPOSE 8080
CMD ["web3-server"]
```

## CI/CD 파이프라인

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main, dev]
    paths:
      - 'rust/**'
  pull_request:
    branches: [main, dev]
    paths:
      - 'rust/**'

defaults:
  run:
    working-directory: rust

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Cache
        uses: Swatinem/rust-cache@v2

      - name: Format
        run: cargo fmt --check

      - name: Clippy
        run: cargo clippy -- -D warnings

      - name: Test
        run: cargo test

      - name: Build
        run: cargo build --release
```

## 체크리스트

### 컨벤션 문서 작성 시
- [ ] 프로젝트 구조 정의
- [ ] Cargo.toml 의존성 정책
- [ ] 코딩 스타일 규칙
- [ ] 테스트 컨벤션
- [ ] 커밋 전 체크리스트
- [ ] 환경 변수 관리 방법

### 컨벤션 적용 시
- [ ] rustfmt.toml 설정
- [ ] clippy.toml 설정 (필요시)
- [ ] Git hooks 설정
- [ ] CI/CD 파이프라인 구성
- [ ] 팀 전체 공유 및 합의
