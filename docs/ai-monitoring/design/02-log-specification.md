# 로그 스펙

## 로그 포맷

### 기본 구조 (JSON)

```json
{
  "timestamp": "2025-01-31T14:23:45.123456Z",
  "level": "ERROR",
  "target": "server::domain::ai::service",
  "message": "Claude API request failed",
  "fields": {
    "request_id": "req_abc123",
    "error_code": "AI_001",
    "duration_ms": 30500,
    "retry_count": 3
  },
  "span": {
    "name": "process_retrospect_assistant",
    "request_id": "req_abc123",
    "user_id": "user_456"
  }
}
```

### 필수 필드

| 필드 | 타입 | 설명 | 예시 |
|------|------|------|------|
| `timestamp` | ISO 8601 | UTC 기준 타임스탬프 | `2025-01-31T14:23:45.123456Z` |
| `level` | string | 로그 레벨 | `ERROR`, `WARN`, `INFO`, `DEBUG` |
| `target` | string | 로그 발생 위치 (모듈 경로) | `server::domain::ai::service` |
| `message` | string | 로그 메시지 | `Claude API request failed` |

### 권장 필드 (fields)

| 필드 | 타입 | 설명 | 언제 사용 |
|------|------|------|----------|
| `request_id` | string | 요청 추적 ID | 모든 HTTP 요청 |
| `error_code` | string | 에러 코드 | 에러 발생 시 |
| `duration_ms` | number | 처리 시간 (ms) | 외부 API 호출, DB 쿼리 |
| `retry_count` | number | 재시도 횟수 | 재시도 로직 |
| `user_id` | string | 사용자 ID | 인증된 요청 |
| `retrospect_id` | string | 회고 ID | 회고 관련 작업 |

## 로그 레벨 가이드

### ERROR
**모니터링 대상: 즉시 알림**

```rust
// 사용 시점
// - 요청 처리 실패
// - 외부 API 호출 실패 (재시도 소진 후)
// - 데이터베이스 연결 실패
// - 예상치 못한 예외

error!(
    request_id = %req_id,
    error_code = "AI_001",
    message = %err,
    "Claude API request failed after retries"
);
```

**예시 상황:**
- Claude API 타임아웃 (재시도 3회 후)
- 데이터베이스 연결 끊김
- 필수 환경 변수 누락
- JSON 파싱 실패 (내부 데이터)

### WARN
**모니터링 대상: 집계 알림**

```rust
// 사용 시점
// - 재시도 발생 (아직 성공 가능)
// - 성능 저하 감지
// - 비정상적이지만 복구 가능한 상황
// - 사용자 입력 검증 실패

warn!(
    request_id = %req_id,
    retry_count = count,
    "Retrying Claude API request"
);
```

**예시 상황:**
- API 응답 지연 (5초 초과)
- Rate limit 근접
- 캐시 미스
- 입력 값 검증 실패

### INFO
**모니터링 대상: 대시보드/메트릭**

```rust
// 사용 시점
// - 요청 시작/완료
// - 주요 비즈니스 이벤트
// - 정상적인 상태 변경

info!(
    request_id = %req_id,
    duration_ms = elapsed,
    "Request completed successfully"
);
```

**예시 상황:**
- HTTP 요청 수신
- 회고 저장 완료
- 사용자 인증 성공

### DEBUG
**모니터링 대상: 디버깅 시에만**

```rust
// 사용 시점
// - 상세 처리 과정
// - 변수 값 추적
// - 개발/디버깅 목적

debug!(
    request_id = %req_id,
    prompt_tokens = tokens,
    "Prepared prompt for Claude API"
);
```

## 에러 코드 체계

### 코드 형식
```
{DOMAIN}_{NUMBER}
```

### 도메인별 코드

| 도메인 | 접두어 | 범위 | 설명 |
|--------|--------|------|------|
| AI | `AI_` | 001-099 | Claude API 관련 |
| Auth | `AUTH_` | 001-099 | 인증/인가 관련 |
| Database | `DB_` | 001-099 | 데이터베이스 관련 |
| Validation | `VAL_` | 001-099 | 입력 검증 관련 |
| External | `EXT_` | 001-099 | 외부 서비스 관련 |
| Common | `COMMON_` | 400, 500 | 일반 HTTP 에러 |

### AI 에러 코드 상세

| 코드 | HTTP | 설명 | 모니터링 액션 |
|------|------|------|---------------|
| `AI_001` | 401 | API 키 인증 실패 | Critical - 즉시 알림 |
| `AI_002` | 400 | 잘못된 프롬프트 | Warning - 집계 |
| `AI_003` | 408 | API 타임아웃 | Critical - 즉시 알림 |
| `AI_004` | 429 | Rate limit 초과 | Warning - 집계 |
| `AI_005` | 500 | API 내부 오류 | Critical - 즉시 알림 |

### Auth 에러 코드 상세

| 코드 | HTTP | 설명 | 모니터링 액션 |
|------|------|------|---------------|
| `AUTH_001` | 401 | 토큰 없음 | Info - 로그만 |
| `AUTH_002` | 401 | 토큰 만료 | Info - 로그만 |
| `AUTH_003` | 401 | 토큰 변조 | Warning - 집계 |
| `AUTH_004` | 403 | 권한 없음 | Info - 로그만 |

### Database 에러 코드 상세

| 코드 | HTTP | 설명 | 모니터링 액션 |
|------|------|------|---------------|
| `DB_001` | 500 | 연결 실패 | Critical - 즉시 알림 |
| `DB_002` | 500 | 쿼리 타임아웃 | Critical - 즉시 알림 |
| `DB_003` | 500 | 트랜잭션 실패 | Warning - 집계 |
| `DB_004` | 404 | 데이터 없음 | Info - 로그만 |

## 구현 예시

### Tracing 설정 (main.rs)

```rust
use tracing_subscriber::{
    fmt::{self, format::JsonFields},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

pub fn init_tracing() {
    // 중요: flatten_event(false)로 설정하여 fields 중첩 구조 유지
    // Log Watcher에서 .fields.error_code, .fields.request_id 등으로 접근 가능
    let fmt_layer = fmt::layer()
        .json()
        .with_timer(fmt::time::UtcTime::rfc_3339())
        .with_target(true)
        .with_current_span(true)
        .with_span_list(false)
        .flatten_event(false);  // fields 중첩 구조 유지

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,server=debug"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
}
```

### Request ID 미들웨어

```rust
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

pub async fn request_id_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    let request_id = Uuid::new_v4().to_string();
    request.extensions_mut().insert(RequestId(request_id.clone()));

    let span = tracing::info_span!(
        "request",
        request_id = %request_id,
        method = %request.method(),
        uri = %request.uri(),
    );

    let _guard = span.enter();
    next.run(request).await
}
```

### 서비스 레이어 로깅

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(self, secret_key), fields(request_id = %request_id))]
pub async fn process_assistant(
    &self,
    request_id: &str,
    secret_key: &str,
    content: &str,
) -> Result<AssistantResponse, AppError> {
    info!("Starting assistant processing");

    let start = Instant::now();

    match self.claude_client.chat(content).await {
        Ok(response) => {
            let duration = start.elapsed().as_millis() as u64;
            info!(duration_ms = duration, "Claude API call succeeded");
            Ok(response)
        }
        Err(e) => {
            let duration = start.elapsed().as_millis() as u64;
            error!(
                error_code = "AI_003",
                duration_ms = duration,
                error = %e,
                "Claude API call failed"
            );
            Err(AppError::ExternalService(e.to_string()))
        }
    }
}
```

## 로그 파일 관리

### 로테이션 설정

```rust
use tracing_appender::rolling::{RollingFileAppender, Rotation};

let file_appender = RollingFileAppender::builder()
    .rotation(Rotation::DAILY)
    .filename_prefix("server")
    .filename_suffix("log")
    .max_log_files(7)  // 7일 보관
    .build("./logs")
    .expect("Failed to create log appender");
```

### 디렉토리 구조

```
logs/
├── server.2025-01-31.log    # 당일 로그
├── server.2025-01-30.log    # 전일 로그
├── server.2025-01-29.log
└── ...                       # 최대 7일 보관
```

## 민감 정보 처리

### 로깅 금지 필드
- API 키 (`secret_key`, `api_key`)
- 비밀번호 (`password`)
- 토큰 (`access_token`, `refresh_token`)
- 개인 식별 정보 (`email`, `phone`)

### 마스킹 처리

```rust
#[instrument(skip(secret_key))]  // secret_key 로깅 제외
pub async fn call_api(secret_key: &str, data: &str) -> Result<(), Error> {
    // ...
}
```

```rust
// 부분 마스킹이 필요한 경우
fn mask_key(key: &str) -> String {
    if key.len() > 8 {
        format!("{}...{}", &key[..4], &key[key.len()-4..])
    } else {
        "****".to_string()
    }
}

info!(api_key = %mask_key(&key), "Calling external API");
```
