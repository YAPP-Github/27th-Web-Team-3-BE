# Rust AI 협업 가이드라인 (Claude Code)

AI 에이전트가 이 프로젝트에서 코드를 작성할 때 반드시 준수해야 하는 규칙입니다.

## 1. 코드 스타일 (Code Style)

### 네이밍 컨벤션
| 대상 | 규칙 | 예시 |
|------|------|------|
| 함수, 변수, 모듈 | `snake_case` | `get_user_by_id`, `request_count` |
| 구조체, 열거형, 트레이트 | `PascalCase` | `UserService`, `AppError`, `Handler` |
| 상수 | `SCREAMING_SNAKE_CASE` | `MAX_RETRY_COUNT`, `DEFAULT_TIMEOUT` |

### 포맷팅
- 모든 코드는 `cargo fmt` 결과를 따름
- `cargo clippy -- -D warnings` 경고 없어야 함
- 줄 길이 100자 권장 (rustfmt 기본값)

## 2. 에러 처리 (Error Handling)

### 금지 사항
```rust
// 금지: panic! 유발 코드
value.unwrap()           // 금지
value.expect("...")      // 금지 (테스트 제외)
panic!("...")           // 금지
unreachable!()          // 신중히 사용
```

### 권장 사항
```rust
// 권장: Result와 Option 활용
fn process(input: &str) -> Result<Output, AppError> {
    let parsed = input.parse::<i32>()
        .map_err(|_| AppError::ValidationError("Invalid number".into()))?;

    Ok(Output::new(parsed))
}

// 권장: if let / match 패턴
if let Some(value) = optional_value {
    // 값 사용
}

// 권장: ok_or / ok_or_else
let value = optional.ok_or(AppError::NotFound)?;
```

### 에러 타입
- `thiserror`로 커스텀 에러 정의
- 에러는 `AppError` enum으로 통합 관리
- `?` 연산자로 에러 전파

## 3. 비동기 처리 (Async/Await)

### 런타임
- `tokio` 런타임 사용 (full features)
- `async-trait`으로 비동기 trait 정의

### 패턴
```rust
// 좋은 예: async 함수 시그니처
pub async fn fetch_data(&self, id: &str) -> Result<Data, AppError> {
    let response = self.client
        .get(&format!("{}/data/{}", self.base_url, id))
        .send()
        .await?;

    let data = response.json().await?;
    Ok(data)
}

// 나쁜 예: blocking 코드를 async 내에서 사용
pub async fn bad_example() {
    std::thread::sleep(Duration::from_secs(1));  // 금지!
}

// 좋은 예: tokio sleep 사용
pub async fn good_example() {
    tokio::time::sleep(Duration::from_secs(1)).await;
}
```

## 4. API 설계 (API Design)

### DTO 규칙
```rust
// Request DTO
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateItemRequest {
    #[validate(length(min = 1, message = "필수 입력입니다"))]
    pub name: String,

    #[validate(range(min = 0, max = 100))]
    pub quantity: i32,
}

// Response DTO
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ItemResponse {
    pub id: String,
    pub created_at: String,
}
```

### 핸들러 패턴
```rust
#[utoipa::path(
    post,
    path = "/api/items",
    request_body = CreateItemRequest,
    responses(
        (status = 200, body = BaseResponse<ItemResponse>),
        (status = 400, body = ErrorResponse)
    )
)]
pub async fn create_item(
    State(state): State<AppState>,
    Json(req): Json<CreateItemRequest>,
) -> Result<Json<BaseResponse<ItemResponse>>, AppError> {
    req.validate()?;
    let result = state.service.create(&req).await?;
    Ok(Json(BaseResponse::success(result)))
}
```

## 5. 테스트 (Testing)

### 구조
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_error_when_input_is_empty() {
        // Arrange
        let input = "";

        // Act
        let result = validate_input(input);

        // Assert
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_fetch_data_successfully() {
        // Arrange
        let service = MockService::new();

        // Act
        let result = service.fetch("id-123").await;

        // Assert
        assert!(result.is_ok());
    }
}
```

### 테스트 네이밍
- `should_<expected_behavior>_when_<condition>` 형식
- 예: `should_return_error_when_secret_key_invalid`

### 테스트 범위
- 모든 public 함수에 최소 1개 테스트
- 정상 케이스 + 에러 케이스 필수
- 엣지 케이스 (빈 값, 최대값, 특수문자)

## 6. 로깅 (Logging)

### tracing 사용
```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(secret_key))]  // 민감 정보 제외
pub async fn process_request(
    content: &str,
    secret_key: &str,
) -> Result<Response, AppError> {
    info!(content_length = content.len(), "Processing request");

    match do_something().await {
        Ok(result) => {
            info!("Request processed successfully");
            Ok(result)
        }
        Err(e) => {
            error!(error = %e, "Request processing failed");
            Err(e)
        }
    }
}
```

### 로그 레벨
| 레벨 | 용도 |
|------|------|
| `error!` | 복구 불가능한 에러 |
| `warn!` | 복구 가능하지만 주의 필요 |
| `info!` | 주요 비즈니스 이벤트 |
| `debug!` | 개발 중 디버깅 |
| `trace!` | 상세 추적 (거의 사용 안 함) |

## 7. 작업 전 체크리스트

### 구현 전
- [ ] 기존 API 동작 확인: `cargo test`
- [ ] 관련 규칙 파일 확인: `.claude/rules/`
- [ ] 아키텍처 확인: 올바른 레이어에 코드 배치

### 구현 후
- [ ] `cargo fmt` 적용
- [ ] `cargo clippy -- -D warnings` 통과
- [ ] `cargo test` 통과
- [ ] 새 기능에 대한 테스트 추가

## 8. 디렉토리 구조

```
codes/
├── Cargo.toml          # Workspace 설정
└── server/
    ├── src/
    │   ├── main.rs
    │   ├── config.rs       # 환경 설정
    │   ├── error.rs        # 에러 타입
    │   ├── response.rs     # 공통 응답
    │   ├── domain/
    │   │   └── ai/
    │   │       ├── handler.rs   # API 핸들러
    │   │       ├── service.rs   # 비즈니스 로직
    │   │       ├── dto.rs       # Request/Response
    │   │       └── prompt.rs    # 프롬프트 템플릿
    │   └── global/
    │       └── middleware.rs
    └── tests/              # 통합 테스트
```

## 9. 커밋 규칙 (Tidy First)

구조적 변경과 행동적 변경을 분리:

```bash
# 구조 변경 (행동 변경 없음)
git commit -m "🏗️ structure: prompt.rs를 prompt/ 디렉토리로 분할"

# 행동 변경 (기능 추가/수정)
git commit -m "✨ feat: 타임아웃 설정 추가"
git commit -m "🐛 fix: 빈 문자열 처리 버그 수정"
```
