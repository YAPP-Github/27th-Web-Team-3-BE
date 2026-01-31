# Phase 1 (Foundation): 로그 기반 구축

## 개요

| 항목 | 내용 |
|------|------|
| Phase | 1: Foundation |
| 기간 | Week 1-2 |
| 목표 | 구조화된 JSON 로그, 에러 코드 체계, Request ID 전파 |
| 의존성 | 없음 (첫 번째 Phase) |

```
Phase 1 완료 상태
┌─────────────────────────────────────────────────────────────┐
│  ✅ JSON 로그 포맷    ✅ 에러 코드 체계    ✅ Request ID    │
└─────────────────────────────────────────────────────────────┘
```

## 완료 조건

- [ ] 모든 로그가 JSON 형식으로 출력
- [ ] 에러 로그에 `error_code` 필드 포함
- [ ] 모든 요청에 `request_id` 추적 가능
- [ ] 기존 테스트 통과

---

## 태스크 1.1: JSON 로그 포맷 적용

### 목표
`tracing-subscriber`의 JSON 포맷터로 구조화된 로그 출력

### 구현

**파일**: `codes/server/src/utils/logging.rs` (신규)

```rust
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

pub fn init_logging() {
    // 중요: flatten_event(false)로 설정하여 fields 중첩 구조 유지
    // Log Watcher에서 .fields.error_code, .fields.request_id 등으로 접근 가능
    let fmt_layer = fmt::layer()
        .json()
        .with_timer(fmt::time::UtcTime::rfc_3339())
        .with_current_span(true)
        .flatten_event(false);  // fields 중첩 구조 유지

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,server=debug"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
}
```

**main.rs 수정**:
```rust
mod utils;

fn main() {
    utils::logging::init_logging();
    // ...
}
```

### 의존성 추가

```toml
# Cargo.toml
[dependencies]
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
```

### 체크리스트

- [ ] `tracing-subscriber` JSON feature 추가
- [ ] `logging.rs` 모듈 생성
- [ ] `main.rs`에서 `init_logging()` 호출
- [ ] 환경변수 `RUST_LOG` 동작 확인

### 검증

```bash
RUST_LOG=debug cargo run 2>&1 | head -5 | jq .
```

예상 출력:
```json
{
  "timestamp": "2025-01-31T10:00:00.000Z",
  "level": "INFO",
  "target": "server",
  "message": "Server started"
}
```

---

## 태스크 1.2: 에러 코드 체계 적용

### 목표
모든 에러에 고유 코드 부여, 로그에서 추적 가능

### 구현

**파일**: `codes/server/src/utils/error.rs` (수정)

```rust
#[derive(Debug, Clone, Copy)]
pub enum ErrorCode {
    // AI 관련
    AiAuthFailed,       // AI_001
    AiInvalidInput,     // AI_002
    AiTimeout,          // AI_003
    AiRateLimit,        // AI_004
    AiInternalError,    // AI_005

    // Auth 관련
    AuthTokenMissing,   // AUTH_001
    AuthTokenExpired,   // AUTH_002
    AuthTokenInvalid,   // AUTH_003
    AuthForbidden,      // AUTH_004

    // DB 관련
    DbConnectionFailed, // DB_001
    DbQueryTimeout,     // DB_002
    DbNotFound,         // DB_004

    // 일반
    ValidationError,    // VAL_001
    InternalError,      // COMMON_500
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AiAuthFailed => "AI_001",
            Self::AiInvalidInput => "AI_002",
            Self::AiTimeout => "AI_003",
            Self::AiRateLimit => "AI_004",
            Self::AiInternalError => "AI_005",
            Self::AuthTokenMissing => "AUTH_001",
            Self::AuthTokenExpired => "AUTH_002",
            Self::AuthTokenInvalid => "AUTH_003",
            Self::AuthForbidden => "AUTH_004",
            Self::DbConnectionFailed => "DB_001",
            Self::DbQueryTimeout => "DB_002",
            Self::DbNotFound => "DB_004",
            Self::ValidationError => "VAL_001",
            Self::InternalError => "COMMON_500",
        }
    }
}
```

### 에러 로깅 패턴

```rust
// 에러 발생 시
tracing::error!(
    error_code = %ErrorCode::AiTimeout.as_str(),
    "Claude API timeout after {}ms", duration
);
```

### 체크리스트

- [ ] `ErrorCode` enum 정의
- [ ] 기존 `AppError`에 `error_code` 필드 추가
- [ ] 에러 로깅 시 `error_code` 포함
- [ ] 주요 에러 발생 지점 마이그레이션

---

## 태스크 1.3: Request ID 미들웨어

### 목표
모든 요청에 고유 ID 부여, 로그 추적 가능

### 구현

**파일**: `codes/server/src/global/middleware.rs`

```rust
use axum::{extract::Request, middleware::Next, response::Response};
use uuid::Uuid;

#[derive(Clone)]
pub struct RequestId(pub String);

pub async fn request_id_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    request.extensions_mut().insert(RequestId(request_id.clone()));

    // Span에 request_id 포함
    let span = tracing::info_span!(
        "request",
        request_id = %request_id,
        method = %request.method(),
        uri = %request.uri().path(),
    );

    let _guard = span.enter();

    let mut response = next.run(request).await;
    response.headers_mut().insert(
        "x-request-id",
        request_id.parse().unwrap(),
    );

    response
}
```

**라우터 설정**:
```rust
let app = Router::new()
    .route("/api/...", ...)
    .layer(axum::middleware::from_fn(request_id_middleware));
```

### 의존성

```toml
[dependencies]
uuid = { version = "1.0", features = ["v4"] }
```

### 체크리스트

- [ ] `uuid` 크레이트 추가
- [ ] 미들웨어 구현
- [ ] 라우터에 미들웨어 적용
- [ ] 응답 헤더에 `x-request-id` 포함 확인

### 검증

```bash
curl -i http://localhost:3000/health
# 응답 헤더에 x-request-id 확인
```

---

## 산출물

Phase 1 완료 시:

1. **JSON 로그 출력**
```json
{
  "timestamp": "2025-01-31T10:00:00.000Z",
  "level": "ERROR",
  "target": "server::domain::ai::service",
  "fields": {
    "error_code": "AI_003",
    "request_id": "550e8400-e29b-41d4-a716-446655440000"
  },
  "message": "Claude API timeout"
}
```

2. **에러 코드 체계** - 모든 에러에 고유 코드

3. **Request ID 추적** - 요청-응답 전체 흐름 추적 가능

---

## 다음 Phase 연결

Phase 2에서 이 로그를 기반으로:
- Log Watcher가 JSON 파싱
- `error_code`로 필터링
- `request_id`로 관련 로그 그룹핑
