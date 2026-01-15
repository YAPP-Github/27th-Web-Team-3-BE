# Java Reference Skill

Java 레거시 코드를 참조할 때 Spring -> Axum 패턴 매핑 가이드입니다.

## 언제 이 스킬을 사용하나요?

- Java 레거시 코드를 참조하여 Rust로 구현 시
- Spring Boot 패턴의 Axum 대응을 찾을 때
- 기존 API 스펙을 Rust로 구현할 때
- 레거시 코드의 로직을 이해해야 할 때

## 기술 스택 매핑

| Java/Spring | Rust | Crate |
|-------------|------|-------|
| Spring Boot | Axum | `axum` |
| Spring MVC | Axum Router | `axum` |
| `@RestController` | Handler functions | `axum` |
| `@RequestBody` | `Json<T>` | `axum` |
| `@PathVariable` | `Path<T>` | `axum` |
| `@RequestParam` | `Query<T>` | `axum` |
| Bean Validation | validator | `validator` |
| Spring AI (OpenAI) | async-openai | `async-openai` |
| Spring Data JPA | SQLx | `sqlx` |
| Lombok | derive macros | built-in |
| SLF4J/Logback | tracing | `tracing` |
| application.yaml | config-rs / dotenvy | `config`, `dotenvy` |
| SpringDoc | utoipa | `utoipa` |
| `@ControllerAdvice` | `IntoResponse` | `axum` |

## Controller -> Handler 변환

### Java (Before)
```java
@RestController
@RequestMapping("/api/ai")
@RequiredArgsConstructor
public class AiController {
    private final AiService aiService;

    @PostMapping("/retrospective/guide")
    public BaseResponse<GuideResponse> provideGuide(
        @Valid @RequestBody GuideRequest request
    ) {
        GuideResponse response = aiService.provideGuide(
            request.getCurrentContent(),
            request.getSecretKey()
        );
        return BaseResponse.onSuccess(response);
    }
}
```

### Rust (After)
```rust
use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use validator::Validate;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/ai/retrospective/guide", post(provide_guide))
        .route("/api/ai/retrospective/refine", post(refine_retrospective))
}

async fn provide_guide(
    State(state): State<AppState>,
    Json(request): Json<GuideRequest>,
) -> Result<Json<BaseResponse<GuideResponse>>, AppError> {
    request.validate()?;

    let response = state
        .ai_service
        .provide_guide(&request.current_content, &request.secret_key)
        .await?;

    Ok(Json(BaseResponse::success(response)))
}
```

## Service 변환

### Java (Before)
```java
@Service
@RequiredArgsConstructor
public class AiService {
    private final ChatModel chatModel;
    private final SecretKeyValidator secretKeyValidator;

    public GuideResponse provideGuide(String content, String secretKey) {
        secretKeyValidator.validate(secretKey);

        List<Message> messages = List.of(
            new SystemMessage(PromptTemplate.GUIDE_SYSTEM_PROMPT),
            new UserMessage(content)
        );

        String guideMessage = chatModel.call(new Prompt(messages))
            .getResult().getOutput().getText();

        return GuideResponse.builder()
            .currentContent(content)
            .guideMessage(guideMessage)
            .build();
    }
}
```

### Rust (After)
```rust
use async_openai::{
    types::{
        ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};

pub struct AiService {
    client: Client<OpenAIConfig>,
    secret_key_validator: SecretKeyValidator,
}

impl AiService {
    pub fn new(api_key: &str, secret_key: &str) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key);
        Self {
            client: Client::with_config(config),
            secret_key_validator: SecretKeyValidator::new(secret_key),
        }
    }

    pub async fn provide_guide(
        &self,
        content: &str,
        secret_key: &str,
    ) -> Result<GuideResponse, AppError> {
        self.secret_key_validator.validate(secret_key)?;

        let messages = vec![
            ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(GUIDE_SYSTEM_PROMPT)
                    .build()?
            ),
            ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessageArgs::default()
                    .content(format!("User: \"{}\"", content))
                    .build()?
            ),
        ];

        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-4o-mini")
            .messages(messages)
            .build()?;

        let response = self.client.chat().create(request).await?;
        let guide_message = response.choices[0]
            .message
            .content
            .clone()
            .unwrap_or_default();

        Ok(GuideResponse {
            current_content: content.to_string(),
            guide_message,
        })
    }
}
```

## DTO 변환

### Java (Before)
```java
@Getter
@Builder
public class GuideResponse {
    private String currentContent;
    private String guideMessage;
}

@Getter
public class GuideRequest {
    @NotBlank(message = "내용은 필수입니다")
    private String currentContent;

    @NotBlank(message = "비밀 키는 필수입니다")
    private String secretKey;
}
```

### Rust (After)
```rust
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuideResponse {
    pub current_content: String,
    pub guide_message: String,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct GuideRequest {
    #[validate(length(min = 1, message = "내용은 필수입니다"))]
    pub current_content: String,

    #[validate(length(min = 1, message = "비밀 키는 필수입니다"))]
    pub secret_key: String,
}
```

## 에러 처리 변환

### Java (Before)
```java
@RestControllerAdvice
public class ExceptionAdvice {
    @ExceptionHandler(GeneralException.class)
    public BaseResponse<Void> handleGeneralException(GeneralException e) {
        return BaseResponse.onFailure(e.getErrorCode(), e.getMessage(), null);
    }
}
```

### Rust (After)
```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("유효하지 않은 비밀 키입니다.")]
    InvalidSecretKey,

    #[error("유효하지 않은 말투 스타일입니다. KIND 또는 POLITE만 가능합니다.")]
    InvalidToneStyle,

    #[error("잘못된 요청입니다: {0}")]
    ValidationError(String),

    #[error("서버 에러: {0}")]
    Internal(String),

    #[error(transparent)]
    OpenAi(#[from] async_openai::error::OpenAIError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            AppError::InvalidSecretKey => (StatusCode::UNAUTHORIZED, "AI_001"),
            AppError::InvalidToneStyle => (StatusCode::BAD_REQUEST, "AI_002"),
            AppError::ValidationError(_) => (StatusCode::BAD_REQUEST, "COMMON400"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "COMMON500"),
        };

        let body = BaseResponse::<()> {
            is_success: false,
            code: code.to_string(),
            message: self.to_string(),
            result: None,
        };

        (status, Json(body)).into_response()
    }
}
```

## 마이그레이션 체크리스트

### API 호환성
- [ ] 모든 엔드포인트 경로 동일 (`/api/ai/retrospective/guide` 등)
- [ ] HTTP 메서드 동일 (POST, GET 등)
- [ ] 요청/응답 JSON 필드명 camelCase 유지 (`serde(rename_all = "camelCase")`)
- [ ] 에러 코드 동일 (AI_001, AI_002, COMMON400, COMMON500)
- [ ] HTTP 상태 코드 동일 (401, 400, 500)

### 기능 검증
- [ ] 회고 가이드 API 정상 동작
- [ ] 말투 정제 API 정상 동작 (KIND, POLITE 모두)
- [ ] Secret Key 검증 동작
- [ ] 입력 검증 동작 (빈 문자열, 필수값 누락)

### 성능
- [ ] 응답 시간 비교 (Cold start 포함)
- [ ] 메모리 사용량 비교
- [ ] 동시 요청 처리 능력

## 프로젝트 구조 (권장)

```
rust/
├── Cargo.toml
├── src/
│   ├── main.rs              # 진입점
│   ├── config.rs            # 환경 설정
│   ├── error.rs             # AppError 정의
│   ├── response.rs          # BaseResponse 정의
│   ├── domain/
│   │   └── ai/
│   │       ├── mod.rs
│   │       ├── handler.rs   # API 핸들러 (Controller 역할)
│   │       ├── service.rs   # 비즈니스 로직
│   │       ├── dto.rs       # Request/Response DTO
│   │       ├── prompt.rs    # 프롬프트 템플릿
│   │       └── validator.rs # SecretKeyValidator
│   └── global/
│       ├── mod.rs
│       └── middleware.rs    # CORS, 로깅 등
└── tests/
    └── integration/
        └── ai_test.rs       # 통합 테스트
```

## 주의사항

1. **JSON 필드명**: Java의 camelCase를 유지하기 위해 `#[serde(rename_all = "camelCase")]` 필수
2. **비동기**: 모든 IO 작업은 `async/await` 사용
3. **에러 전파**: `?` 연산자로 에러 전파, `impl From<SourceError>` 활용
4. **환경 변수**: `OPENAI_API_KEY`, `AI_SECRET_KEY` 등 민감 정보는 환경 변수로 관리
