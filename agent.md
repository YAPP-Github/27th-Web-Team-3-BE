# Engineering Conventions for Web-3 Backend

## 작업 디렉토리

모든 Rust 작업은 `rust/` 디렉토리에서 수행합니다.

```bash
cd rust
```

## Feature Flags

- 내부 실험용 기능 플래그 사용 금지
- 실험적 코드는 머지 전 제거
- feature가 필요하면 `Cargo.toml`에 명시적으로 정의

## Performance Work

- 벤치마크로 검증: `cargo bench`
- 프로파일링: `cargo flamegraph`
- 핫 패스에서 불필요한 할당 피하기
- `clone()` 사용 최소화

## Coding Style

- 변경은 명시된 목표에 최소화하고 집중
- 한 PR에 하나의 논리적 변경만
- 과도한 추상화 피하기

## Testing

- 테스트 프레임워크: `cargo test` (built-in)
- 비동기 테스트: `#[tokio::test]`
- 커버리지 목표: 핵심 비즈니스 로직 80% 이상

```bash
# 테스트 실행
cargo test

# 특정 테스트만
cargo test test_name

# 출력 표시
cargo test -- --nocapture
```

## Hygiene Before Commit

커밋 전 필수:

```bash
# 1. 포맷팅
cargo fmt

# 2. 린팅 (경고 = 에러)
cargo clippy -- -D warnings

# 3. 테스트
cargo test

# 4. (선택) 빌드 확인
cargo build
```

- [ ] 리팩토링으로 생긴 죽은 코드 제거
- [ ] 사용하지 않는 import 제거
- [ ] `TODO` 주석 정리 또는 이슈 생성

## Build & Run

```bash
# 개발 빌드
cargo build

# 릴리즈 빌드
cargo build --release

# 실행
cargo run

# 로그 레벨 지정
RUST_LOG=debug cargo run
```

## Error Handling

- `unwrap()` / `expect()` 프로덕션 코드에서 사용 금지
- `Result<T, E>` + `?` 연산자 사용
- `thiserror`로 에러 타입 정의
- 에러 메시지는 사용자 친화적으로

```rust
// 좋은 예
fn process(input: &str) -> Result<Output, AppError> {
    let parsed = input.parse().map_err(|_| AppError::InvalidInput)?;
    Ok(parsed)
}

// 나쁜 예
fn process(input: &str) -> Output {
    input.parse().unwrap() // 패닉 위험!
}
```

## Async/Await

- 모든 IO 작업은 async
- `tokio::spawn` 사용 시 에러 처리 주의
- blocking 작업은 `tokio::task::spawn_blocking` 사용

## Dependencies

- 새 의존성 추가 전 팀 논의
- 보안 취약점 정기 검사: `cargo audit`
- 버전은 명시적으로 고정 (Cargo.lock 커밋)

## API Design

- RESTful 원칙 준수
- JSON 필드명: camelCase (`#[serde(rename_all = "camelCase")]`)
- 응답 포맷: `BaseResponse<T>` 사용
- 에러 코드: 기존 코드 체계 유지 (AI_001, COMMON400 등)

## Project-Specific Rules

### AI 도메인
- OpenAI API 호출은 `AiService`를 통해서만
- 프롬프트 템플릿은 `prompt.rs`에 상수로 정의
- Secret Key 검증은 모든 AI 엔드포인트에 필수

### 새 도메인 추가 시
```
src/domain/[domain_name]/
├── mod.rs
├── handler.rs    # API 핸들러
├── service.rs    # 비즈니스 로직
├── dto.rs        # Request/Response
└── repository.rs # DB 접근 (필요시)
```
