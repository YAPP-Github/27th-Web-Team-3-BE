# Code Quality Guard Skill (Rust Edition)

프로덕션 Rust 코드 품질을 보장하기 위한 가드레일과 체크리스트입니다.

## 언제 이 스킬을 사용하나요?

- 프로덕션 코드 작성 시
- 코드 리뷰 시
- PR 생성 전 최종 점검 시
- 품질 감사 시

## 절대 하지 말아야 할 것들 (NEVER)

프로덕션 코드에 다음을 포함하지 마세요:

### 1. 무분별한 `unwrap()` / `expect()`

```rust
// 나쁜 예: 정상 작동 경로에서 패닉 가능
fn get_user_id(headers: &HeaderMap) -> String {
    headers.get("X-User-Id").unwrap().to_str().unwrap().to_string()
}

// 좋은 예: Result 반환
fn get_user_id(headers: &HeaderMap) -> Result<String, AppError> {
    headers
        .get("X-User-Id")
        .ok_or(AppError::MissingHeader("X-User-Id"))?
        .to_str()
        .map_err(|_| AppError::InvalidHeader("X-User-Id"))?
        .to_string()
        .pipe(Ok)
}

// 또는 Option 반환
fn get_user_id(headers: &HeaderMap) -> Option<String> {
    headers.get("X-User-Id")?.to_str().ok()?.to_string().into()
}
```

### 2. 메모리 누수

```rust
// 나쁜 예: 순환 참조 가능
struct Node {
    children: Vec<Rc<RefCell<Node>>>,
    parent: Option<Rc<RefCell<Node>>>, // 순환 참조!
}

// 좋은 예: Weak 사용
struct Node {
    children: Vec<Rc<RefCell<Node>>>,
    parent: Option<Weak<RefCell<Node>>>, // 순환 방지
}
```

### 3. 데이터 레이스

```rust
// 나쁜 예: 안전하지 않은 공유 상태
static mut COUNTER: u32 = 0; // unsafe 필요!

// 좋은 예: 동기화 프리미티브 사용
use std::sync::atomic::{AtomicU32, Ordering};
static COUNTER: AtomicU32 = AtomicU32::new(0);
```

### 4. 불필요한 `unsafe`

- `unsafe` 사용 시 반드시 주석으로 안전성 근거 설명
- 가능하면 safe한 대안 사용

## 항상 해야 할 것들 (ALWAYS)

### 1. 포괄적인 테스트 작성

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_case() {
        let result = process_request(valid_input());
        assert!(result.is_ok());
    }

    #[test]
    fn test_edge_cases() {
        // 빈 문자열
        assert!(process_request("").is_err());

        // 매우 긴 입력
        let long_input = "a".repeat(10000);
        assert!(process_request(&long_input).is_ok());

        // 유니코드
        assert!(process_request("한글 테스트 ").is_ok());
    }

    #[test]
    fn test_error_conditions() {
        assert!(matches!(
            process_request(invalid_input()),
            Err(AppError::ValidationError(_))
        ));
    }
}
```

### 2. 입력 검증

```rust
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct RefineRequest {
    #[validate(length(min = 1, message = "내용은 필수입니다"))]
    pub content: String,

    #[validate(custom = "validate_tone_style")]
    pub tone_style: String,

    #[validate(length(min = 1, message = "비밀 키는 필수입니다"))]
    pub secret_key: String,
}

fn validate_tone_style(tone: &str) -> Result<(), ValidationError> {
    match tone {
        "KIND" | "POLITE" => Ok(()),
        _ => Err(ValidationError::new("invalid_tone_style")),
    }
}
```

### 3. 정적 분석 도구 사용

```bash
# 커밋 전 필수 실행
cargo fmt --check          # 포맷팅 검사
cargo clippy -- -D warnings # 린트 검사 (경고를 에러로)
cargo test                  # 테스트
cargo audit                 # 보안 취약점 검사
```

### 4. 적절한 로깅

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(secret_key))] // 민감 정보 로깅 방지
pub async fn provide_guide(
    &self,
    content: &str,
    secret_key: &str,
) -> Result<GuideResponse, AppError> {
    info!(content_length = content.len(), "가이드 생성 요청");

    let result = self.call_openai(content).await;

    match &result {
        Ok(response) => info!(guide_length = response.guide_message.len(), "가이드 생성 성공"),
        Err(e) => error!(error = %e, "가이드 생성 실패"),
    }

    result
}
```

## 개발 프로세스 가드

### 테스트 요구사항

- [ ] 실패 테스트 먼저 작성, 그 다음 구현 (TDD)
- [ ] `#[ignore]`로 테스트 건너뛰지 않기 - 문제를 수정
- [ ] 비동기 코드는 `#[tokio::test]` 사용
- [ ] 모든 public 함수에 테스트
- [ ] 엣지 케이스와 에러 조건 검증

### 아키텍처 요구사항

- [ ] 명시적 에러 처리 - `Result<T, E>` 사용
- [ ] 메모리 안전성 - `unsafe` 최소화
- [ ] 비동기 안전성 - `Send + Sync` bound 확인
- [ ] API 일관성 - 기존 API 스펙 준수

## Rust Idiom 패턴

### Option/Result 체이닝

```rust
// 좋은 예: 체이닝
fn process(input: Option<&str>) -> Result<String, AppError> {
    input
        .filter(|s| !s.is_empty())
        .map(|s| s.trim())
        .ok_or(AppError::EmptyInput)?
        .parse()
        .map_err(AppError::ParseError)
}

// 피해야 할 패턴: 중첩 if-let
fn process(input: Option<&str>) -> Result<String, AppError> {
    if let Some(s) = input {
        if !s.is_empty() {
            if let Ok(parsed) = s.trim().parse() {
                return Ok(parsed);
            }
        }
    }
    Err(AppError::EmptyInput)
}
```

### Builder 패턴

```rust
#[derive(Default)]
pub struct RequestBuilder {
    content: Option<String>,
    tone_style: Option<ToneStyle>,
}

impl RequestBuilder {
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    pub fn tone_style(mut self, style: ToneStyle) -> Self {
        self.tone_style = Some(style);
        self
    }

    pub fn build(self) -> Result<RefineRequest, AppError> {
        Ok(RefineRequest {
            content: self.content.ok_or(AppError::MissingField("content"))?,
            tone_style: self.tone_style.ok_or(AppError::MissingField("tone_style"))?,
        })
    }
}
```

## 리뷰 체크포인트

코드를 완료로 표시하기 전 확인:

1. [ ] `cargo build` 경고 없음
2. [ ] `cargo test` 모든 테스트 통과
3. [ ] `cargo clippy` 경고 없음
4. [ ] `cargo fmt` 포맷팅 완료
5. [ ] 에러 처리가 포괄적이고 일관적
6. [ ] 로깅이 적절히 적용됨
7. [ ] 문서 주석이 public API에 있음
8. [ ] 기존 API 스펙과 호환됨
