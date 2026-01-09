# Java에서 Rust로: 마이그레이션 완료 보고서

## 1. 개요

### 1.1 마이그레이션 목적

기존 Java 17 / Spring Boot 기반의 백엔드 서버를 Rust로 완전히 마이그레이션했습니다.

**마이그레이션 이유**
- **Cold Start 제거**: JVM의 긴 시작 시간(수 초) → Rust의 빠른 시작(밀리초 단위)
- **메모리 효율성**: JVM Heap 오버헤드 제거로 메모리 사용량 대폭 감소
- **런타임 안정성**: Rust의 강력한 타입 시스템과 컴파일 타임 검증
- **고성능 비동기**: Tokio 기반의 효율적인 비동기 처리

### 1.2 마이그레이션 범위

| 항목 | 상태 |
|------|------|
| 회고 작성 가이드 API | ✅ 완료 |
| 회고 말투 정제 API | ✅ 완료 |
| Secret Key 인증 | ✅ 완료 |
| 에러 처리 및 응답 포맷 | ✅ 완료 |
| OpenAI 연동 | ✅ 완료 |
| 단위 테스트 | ✅ 완료 |

---

## 2. 기술 스택 변경

### 2.1 Before vs After

| 구분 | Java (Before) | Rust (After) |
|------|---------------|--------------|
| **언어** | Java 17 | Rust 1.84+ |
| **프레임워크** | Spring Boot 3.5.x | Axum 0.7 |
| **비동기 처리** | Spring MVC (Thread-per-request) | Tokio (async/await) |
| **AI 연동** | Spring AI (OpenAI) | async-openai 0.18 |
| **유효성 검사** | Bean Validation | validator 0.16 |
| **직렬화** | Jackson | serde 1.x |
| **에러 처리** | @RestControllerAdvice | thiserror + IntoResponse |
| **로깅** | Logback / SLF4J | tracing 0.1 |
| **설정 관리** | application.yaml | dotenvy (환경 변수) |
| **API 문서** | SpringDoc (Swagger) | utoipa (예정) |

### 2.2 의존성 비교

**Java (build.gradle)**
```groovy
dependencies {
    implementation 'org.springframework.boot:spring-boot-starter-web'
    implementation 'org.springframework.ai:spring-ai-openai-spring-boot-starter'
    implementation 'org.springdoc:springdoc-openapi-starter-webmvc-ui'
    // ... 추가 의존성
}
```

**Rust (Cargo.toml)**
```toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
async-openai = "0.18"
serde = { version = "1", features = ["derive"] }
validator = { version = "0.16", features = ["derive"] }
thiserror = "1"
tracing = "0.1"
```

---

## 3. 프로젝트 구조 비교

### 3.1 디렉토리 구조

```
# Java (jvm/)                          # Rust (rust/)
├── src/main/java/                     ├── src/
│   └── com/yapp/web3/Server/          │   ├── main.rs           # 진입점
│       ├── ServerApplication.java     │   ├── lib.rs            # 라이브러리 루트
│       ├── domain/                    │   ├── config.rs         # 설정
│       │   └── ai/                    │   ├── error.rs          # 에러 타입
│       │       ├── controller/        │   ├── response.rs       # 공통 응답
│       │       ├── dto/               │   ├── domain/
│       │       │   ├── request/       │   │   └── ai/
│       │       │   └── response/      │   │       ├── mod.rs    # 라우터
│       │       ├── prompt/            │   │       ├── handler.rs
│       │       ├── service/           │   │       ├── service.rs
│       │       └── validator/         │   │       ├── dto.rs
│       └── global/                    │   │       └── prompt.rs
│           ├── common/                │   └── global/
│           ├── config/                │       ├── mod.rs
│           └── error/                 │       └── validator.rs
└── src/main/resources/                └── .env                  # 환경 변수
    └── application.yaml
```

### 3.2 파일 매핑

| Java 파일 | Rust 파일 | 설명 |
|-----------|-----------|------|
| `AiController.java` | `domain/ai/handler.rs` | API 핸들러 |
| `AiService.java` | `domain/ai/service.rs` | 비즈니스 로직 |
| `GuideRequest.java` | `domain/ai/dto.rs` | Request DTO |
| `GuideResponse.java` | `domain/ai/dto.rs` | Response DTO |
| `RefineRequest.java` | `domain/ai/dto.rs` | Request DTO |
| `RefineResponse.java` | `domain/ai/dto.rs` | Response DTO |
| `PromptTemplate.java` | `domain/ai/prompt.rs` | AI 프롬프트 |
| `SecretKeyValidator.java` | `global/validator.rs` | 인증 검증 |
| `BaseResponse.java` | `response.rs` | 공통 응답 |
| `ErrorStatus.java` | `error.rs` | 에러 코드 |
| `ExceptionAdvice.java` | `error.rs` | 에러 핸들러 |
| `application.yaml` | `.env` | 환경 설정 |

---

## 4. 코드 비교

### 4.1 컨트롤러 → 핸들러

**Java (AiController.java)**
```java
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
```

**Rust (handler.rs)**
```rust
pub async fn provide_guide(
    State(state): State<AppState>,
    request: Result<Json<GuideRequest>, JsonRejection>,
) -> Result<Json<BaseResponse<GuideResponse>>, AppError> {
    let Json(request) = request.map_err(AppError::from)?;
    request.validate()?;

    let response = state.ai_service
        .provide_guide(&request.current_content, &request.secret_key)
        .await?;

    Ok(Json(BaseResponse::success(response)))
}
```

**주요 변경점**
- `@Valid` 어노테이션 → `request.validate()` 메서드 호출
- 동기 방식 → `async/await` 비동기 방식
- 예외 던지기 → `Result<T, E>` 반환

### 4.2 서비스 레이어

**Java (AiService.java)**
```java
public GuideResponse provideGuide(String currentContent, String secretKey) {
    secretKeyValidator.validate(secretKey);

    List<Message> messages = List.of(
        new SystemMessage(PromptTemplate.GUIDE_SYSTEM_PROMPT),
        new UserMessage(PromptTemplate.GUIDE_FEW_SHOT_EXAMPLES),
        new UserMessage("User: \"" + currentContent + "\"")
    );

    Prompt prompt = new Prompt(messages);
    String guideMessage = chatModel.call(prompt)
        .getResult().getOutput().getText();

    return GuideResponse.builder()
        .currentContent(currentContent)
        .guideMessage(guideMessage)
        .build();
}
```

**Rust (service.rs)**
```rust
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
        // ... 나머지 메시지
    ];

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4o-mini")
        .messages(messages)
        .build()?;

    let response = self.client.chat().create(request).await?;
    let guide_message = response.choices[0].message.content.clone().unwrap_or_default();

    Ok(GuideResponse {
        current_content: content.to_string(),
        guide_message,
    })
}
```

**주요 변경점**
- Spring AI ChatModel → async-openai Client
- 예외 발생 → `Result` 타입의 `?` 연산자 사용
- Lombok Builder → Rust 구조체 직접 생성

### 4.3 DTO 정의

**Java (GuideRequest.java)**
```java
@Getter
public class GuideRequest {
    @NotBlank(message = "내용은 필수입니다")
    private String currentContent;

    @NotBlank(message = "비밀 키는 필수입니다")
    private String secretKey;
}
```

**Rust (dto.rs)**
```rust
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct GuideRequest {
    #[validate(length(min = 1, message = "내용은 필수입니다"))]
    pub current_content: String,

    #[validate(length(min = 1, message = "비밀 키는 필수입니다"))]
    pub secret_key: String,
}
```

**주요 변경점**
- Lombok `@Getter` → Rust의 `pub` 필드
- Jackson → serde의 `#[serde(rename_all = "camelCase")]`
- Bean Validation → validator derive 매크로

### 4.4 에러 처리

**Java (ExceptionAdvice.java)**
```java
@RestControllerAdvice
public class ExceptionAdvice {
    @ExceptionHandler(GeneralException.class)
    public ResponseEntity<BaseResponse<?>> handleGeneralException(GeneralException e) {
        // 에러 응답 생성
    }
}
```

**Rust (error.rs)**
```rust
#[derive(Error, Debug)]
pub enum AppError {
    #[error("유효하지 않은 비밀 키입니다.")]
    InvalidSecretKey,

    #[error("유효하지 않은 말투 스타일입니다.")]
    InvalidToneStyle,
    // ...
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            AppError::InvalidSecretKey => (StatusCode::UNAUTHORIZED, "AI_001"),
            AppError::InvalidToneStyle => (StatusCode::BAD_REQUEST, "AI_002"),
            // ...
        };

        let body = BaseResponse::<()>::error(code, &self.to_string());
        (status, Json(body)).into_response()
    }
}
```

**주요 변경점**
- `@ExceptionHandler` 어노테이션 → `IntoResponse` trait 구현
- Exception 클래스 계층 → enum 기반 에러 타입
- `thiserror` crate로 에러 메시지 정의

---

## 5. API 스펙 (변경 없음)

### 5.1 회고 작성 가이드 API

```
POST /api/ai/retrospective/guide
```

**Request**
```json
{
  "currentContent": "오늘 프로젝트를 진행하면서...",
  "secretKey": "your-secret-key"
}
```

**Response (200 OK)**
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공입니다.",
  "result": {
    "currentContent": "오늘 프로젝트를 진행하면서...",
    "guideMessage": "좋은 시작이에요! ..."
  }
}
```

### 5.2 회고 말투 정제 API

```
POST /api/ai/retrospective/refine
```

**Request**
```json
{
  "content": "오늘 일 힘들었음",
  "toneStyle": "KIND",
  "secretKey": "your-secret-key"
}
```

**Response (200 OK)**
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공입니다.",
  "result": {
    "originalContent": "오늘 일 힘들었음",
    "refinedContent": "오늘 일이 많이 힘들었어요.",
    "toneStyle": "KIND"
  }
}
```

### 5.3 에러 응답 (변경 없음)

| 코드 | HTTP Status | 설명 |
|------|-------------|------|
| AI_001 | 401 | 유효하지 않은 비밀 키 |
| AI_002 | 400 | 유효하지 않은 말투 스타일 |
| COMMON400 | 400 | 잘못된 요청 |
| COMMON500 | 500 | 서버 내부 에러 |

---

## 6. 개발 환경 설정

### 6.1 Rust 설치

```bash
# Rust 설치 (처음인 경우)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 설치 후 PATH 적용
source $HOME/.cargo/env

# 버전 확인
rustc --version   # rustc 1.84.x 이상
cargo --version
```

### 6.2 개발 도구 설치

```bash
# 코드 포맷터 및 린터
rustup component add rustfmt clippy
```

### 6.3 환경 변수 설정

```bash
# rust/.env 파일 생성
OPENAI_API_KEY=sk-your-api-key
AI_SECRET_KEY=your-secret-key
SERVER_PORT=8080      # 선택 (기본값: 8080)
RUST_LOG=info         # 선택 (debug, info, warn, error)
```

---

## 7. 빌드 및 실행

### 7.1 개발 모드

```bash
cd rust

# 빌드
cargo build

# 실행
cargo run

# 디버그 로그와 함께 실행
RUST_LOG=debug cargo run
```

### 7.2 릴리즈 빌드

```bash
# 최적화된 릴리즈 빌드
cargo build --release

# 실행 파일 위치
./target/release/web3-server
```

### 7.3 Docker

```bash
# 이미지 빌드
docker build -t web3-server .

# 컨테이너 실행
docker run -p 8080:8080 \
  -e OPENAI_API_KEY=sk-xxx \
  -e AI_SECRET_KEY=xxx \
  web3-server
```

---

## 8. 테스트

### 8.1 테스트 실행

```bash
cd rust

# 모든 테스트 실행
cargo test

# 특정 테스트만 실행
cargo test test_name

# 테스트 출력 표시
cargo test -- --nocapture

# 특정 모듈 테스트
cargo test domain::ai::dto
```

### 8.2 테스트 구조

```
rust/src/
├── config.rs           # Config 테스트
├── error.rs            # AppError 테스트
├── response.rs         # BaseResponse 테스트
├── domain/ai/
│   ├── dto.rs          # DTO 직렬화/검증 테스트
│   ├── service.rs      # Service 테스트 (Secret Key 검증)
│   └── prompt.rs       # 프롬프트 테스트
└── global/
    └── validator.rs    # SecretKeyValidator 테스트
```

### 8.3 테스트 예시

```rust
#[test]
fn guide_request_should_deserialize_from_camel_case() {
    let json = r#"{"currentContent": "test", "secretKey": "key"}"#;
    let request: GuideRequest = serde_json::from_str(json).unwrap();

    assert_eq!(request.current_content, "test");
    assert_eq!(request.secret_key, "key");
}

#[test]
fn guide_request_should_validate_empty_content() {
    let request = GuideRequest {
        current_content: "".to_string(),
        secret_key: "key".to_string(),
    };

    let result = request.validate();
    assert!(result.is_err());
}
```

---

## 9. 코드 품질

### 9.1 포맷팅

```bash
# 코드 포맷팅 (자동 수정)
cargo fmt

# 포맷 검사만 (CI용)
cargo fmt -- --check
```

### 9.2 린트

```bash
# clippy 실행
cargo clippy

# 경고를 에러로 처리 (CI용)
cargo clippy -- -D warnings
```

### 9.3 커밋 전 체크리스트

```bash
# 1. 테스트 통과
cargo test

# 2. 포맷팅
cargo fmt

# 3. 린트 통과
cargo clippy -- -D warnings
```

---

## 10. 성능 개선

### 10.1 기대 효과

| 항목 | Java | Rust | 개선 |
|------|------|------|------|
| Cold Start | ~3초 | ~50ms | **60배 향상** |
| 메모리 (유휴) | ~300MB | ~10MB | **30배 감소** |
| 바이너리 크기 | ~50MB (JAR) | ~10MB | **5배 감소** |
| 동시 요청 처리 | Thread Pool | async/await | 더 효율적 |

### 10.2 릴리즈 빌드 최적화

```toml
# Cargo.toml
[profile.release]
lto = true          # Link Time Optimization
codegen-units = 1   # 단일 코드 생성 단위
strip = true        # 디버그 심볼 제거
```

---

## 11. 트러블슈팅

### Q1: `cargo build` 실패 - OpenSSL 관련 에러

```bash
# macOS
brew install openssl

# Ubuntu
sudo apt-get install pkg-config libssl-dev
```

### Q2: 환경 변수를 인식하지 못함

```bash
# .env 파일이 rust/ 디렉토리에 있는지 확인
ls rust/.env

# 또는 직접 환경 변수 설정
export OPENAI_API_KEY=sk-xxx
export AI_SECRET_KEY=xxx
```

### Q3: 401 에러 (Invalid Secret Key)

- `.env` 파일의 `AI_SECRET_KEY` 값 확인
- 요청 JSON의 `secretKey` 필드 확인
- 공백 문자 주의

### Q4: 테스트에서 OpenAI API 호출 실패

테스트는 실제 OpenAI API를 호출하지 않습니다. Secret Key 검증 테스트만 수행합니다.

---

## 12. FAQ

### Q: Java 코드는 어디에 있나요?

`jvm/` 디렉토리에 레거시 코드로 보관되어 있습니다. 참조용으로 남겨두었습니다.

### Q: API 스펙이 변경되었나요?

아니요. 엔드포인트, 요청/응답 포맷, 에러 코드 모두 100% 동일합니다. 프론트엔드 변경 없이 사용 가능합니다.

### Q: Rust를 몰라도 개발할 수 있나요?

기본적인 Rust 문법을 이해하면 됩니다. 핵심 패턴:
- `Result<T, E>`: 성공/실패를 명시적으로 처리
- `?` 연산자: 에러 전파
- `async/await`: 비동기 처리
- `struct` + `impl`: 클래스와 유사한 개념

### Q: 새 API를 추가하려면?

1. `dto.rs`에 Request/Response 구조체 정의
2. `service.rs`에 비즈니스 로직 구현
3. `handler.rs`에 핸들러 함수 추가
4. `mod.rs`에 라우트 등록

---

## 13. 참고 자료

- [CLAUDE.md](../CLAUDE.md) - 프로젝트 개발 가이드
- [docs/api/](api/) - API 상세 스펙
- [docs/migration/](migration/) - 마이그레이션 단계별 문서
- [Axum 공식 문서](https://docs.rs/axum)
- [async-openai 문서](https://docs.rs/async-openai)
- [Rust Book (한국어)](https://rinthel.github.io/rust-lang-book-ko/)
