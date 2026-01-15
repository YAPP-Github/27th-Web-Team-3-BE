# Coding Standards Skill (Rust Edition)

팀의 Rust 코딩 표준과 컨벤션을 일관되게 적용하기 위한 스킬입니다.

## 언제 이 스킬을 사용하나요?

- 새로운 Rust 코드 작성 시
- 코드 리뷰 시
- 리팩토링 작업 시
- 신규 팀원 온보딩 시

## Rust 코딩 원칙

### 1. 네이밍 컨벤션

```rust
// 모듈, 함수, 변수: snake_case
mod ai_service;
fn provide_guide() {}
let guide_message = "";

// 타입, Trait, Enum: PascalCase
struct GuideResponse {}
trait Validator {}
enum ToneStyle { Kind, Polite }

// 상수: SCREAMING_SNAKE_CASE
const MAX_RETRIES: u32 = 3;
const API_TIMEOUT_SECS: u64 = 30;

// 라이프타임: 짧은 소문자
fn parse<'a>(input: &'a str) -> &'a str {}
```

### 2. 코드 구조

```rust
// 좋은 예: 명확한 구조, Result 사용
pub async fn provide_guide(
    &self,
    content: &str,
) -> Result<GuideResponse, AppError> {
    let messages = self.build_messages(content);
    let response = self.client.chat().create(messages).await?;

    Ok(GuideResponse {
        current_content: content.to_string(),
        guide_message: response.choices[0].message.content.clone(),
    })
}

// 나쁜 예: 불명확한 구조
pub async fn guide(&self, c: &str) -> String {
    self.client.chat().create(c).await.unwrap().choices[0].message.content.clone()
}
```

### 3. 에러 처리

```rust
// 좋은 예: thiserror를 사용한 명확한 에러 타입
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("유효하지 않은 비밀 키입니다.")]
    InvalidSecretKey,

    #[error("유효하지 않은 말투 스타일입니다. KIND 또는 POLITE만 가능합니다.")]
    InvalidToneStyle,

    #[error("OpenAI API 호출 실패: {0}")]
    OpenAiError(#[from] async_openai::error::OpenAIError),

    #[error("서버 내부 에러: {0}")]
    Internal(String),
}

// axum IntoResponse 구현
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::InvalidSecretKey =>
                (StatusCode::UNAUTHORIZED, "AI_001", self.to_string()),
            AppError::InvalidToneStyle =>
                (StatusCode::BAD_REQUEST, "AI_002", self.to_string()),
            _ =>
                (StatusCode::INTERNAL_SERVER_ERROR, "COMMON500", self.to_string()),
        };

        let body = BaseResponse::<()>::error(code, &message);
        (status, Json(body)).into_response()
    }
}

// 나쁜 예: panic 또는 unwrap
fn process(value: &str) -> String {
    value.parse::<i32>().unwrap().to_string() // 위험!
}
```

### 4. 문서화

```rust
/// 회고 내용에 대한 AI 가이드 메시지를 생성합니다.
///
/// # Arguments
///
/// * `content` - 사용자가 작성 중인 회고 내용
///
/// # Returns
///
/// 성공 시 가이드 메시지가 포함된 `GuideResponse`를 반환합니다.
///
/// # Errors
///
/// * `AppError::OpenAiError` - OpenAI API 호출 실패 시
///
/// # Example
///
/// ```rust
/// let service = AiService::new(client);
/// let response = service.provide_guide("오늘 프로젝트를...").await?;
/// println!("{}", response.guide_message);
/// ```
pub async fn provide_guide(&self, content: &str) -> Result<GuideResponse, AppError> {
    // ...
}
```

### 5. 구조체와 Derive

```rust
use serde::{Deserialize, Serialize};
use validator::Validate;

/// 회고 가이드 요청 DTO
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct GuideRequest {
    #[validate(length(min = 1, message = "내용은 필수입니다"))]
    pub current_content: String,

    #[validate(length(min = 1, message = "비밀 키는 필수입니다"))]
    pub secret_key: String,
}

/// 회고 가이드 응답 DTO
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuideResponse {
    pub current_content: String,
    pub guide_message: String,
}
```

## 커밋 메시지 규칙

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Type 종류
- `feat`: 새로운 기능
- `fix`: 버그 수정
- `refactor`: 리팩토링
- `test`: 테스트 추가/수정
- `docs`: 문서 변경
- `style`: 코드 포맷팅
- `chore`: 빌드, 설정 변경

### 예시
```
feat(ai): 회고 요약 기능 추가

- 긴 회고 내용을 3문장으로 요약
- OpenAI GPT-4 사용
- 새로운 엔드포인트: POST /api/ai/retrospective/summarize
```

## 코드 리뷰 체크리스트

- [ ] 네이밍이 Rust 컨벤션을 따르는가?
- [ ] `unwrap()`, `expect()` 사용이 정당화되는가?
- [ ] 에러 처리가 `Result`로 적절히 되어 있는가?
- [ ] 테스트가 포함되어 있는가?
- [ ] `clippy` 경고가 없는가?
- [ ] `cargo fmt`가 적용되어 있는가?
- [ ] 문서 주석이 적절한가?
- [ ] 불필요한 `clone()`이 없는가?
