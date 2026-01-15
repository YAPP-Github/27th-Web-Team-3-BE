# TDD & Tidy First Skill (Rust Edition)

Kent Beck의 Test-Driven Development(TDD)와 Tidy First 원칙을 따르는 Rust 개발 가이드입니다.

## 언제 이 스킬을 사용하나요?

- 새로운 기능 구현 시
- 버그 수정 시
- 리팩토링 작업 시
- 코드 품질 향상 작업 시

## 핵심 개발 원칙

### TDD 사이클: Red -> Green -> Refactor

1. **Red**: 실패하는 테스트를 먼저 작성
2. **Green**: 테스트를 통과하는 최소한의 코드 구현
3. **Refactor**: 테스트 통과 후 코드 개선

## Rust TDD 방법론 가이드

### 테스트 작성 원칙

- 작은 기능 증분을 정의하는 실패 테스트부터 시작
- 행동을 설명하는 의미 있는 테스트 이름 사용 (예: `should_sum_two_positive_numbers`)
- `#[test]` 속성과 `assert!`, `assert_eq!`, `assert_ne!` 매크로 활용
- 테스트를 통과시키기 위한 최소한의 코드만 작성
- 테스트 통과 후 리팩토링 필요성 검토

### Rust 테스트 구조

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_guide_message_for_valid_input() {
        // Arrange
        let service = AiService::new(mock_client());
        let input = "오늘 프로젝트를 진행하면서...";

        // Act
        let result = service.provide_guide(input);

        // Assert
        assert!(result.is_ok());
        assert!(!result.unwrap().guide_message.is_empty());
    }

    #[test]
    fn should_return_error_for_invalid_secret_key() {
        // Arrange
        let validator = SecretKeyValidator::new("correct_key");

        // Act
        let result = validator.validate("wrong_key");

        // Assert
        assert!(result.is_err());
        assert!(matches!(result, Err(AppError::InvalidSecretKey)));
    }
}
```

### 비동기 테스트 (Tokio)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_call_openai_api_successfully() {
        let client = AsyncOpenAI::new();
        let response = client.chat().create(request).await;

        assert!(response.is_ok());
    }
}
```

### 버그 수정 시 TDD

버그 수정 시에는:
1. 먼저 실패하는 테스트 작성 (버그 재현)
2. 문제를 재현하는 가장 작은 테스트 작성
3. 두 테스트 모두 통과하도록 수정

## Tidy First 접근법

**모든 변경을 두 가지 유형으로 분리:**

### 1. 구조적 변경 (Structural Changes)
행동 변경 없이 코드 재배치:
- 이름 변경 (`rename`)
- 함수 추출 (`extract function`)
- 모듈 분리
- `impl` 블록 재구성

### 2. 행동적 변경 (Behavioral Changes)
실제 기능 추가 또는 수정

### 핵심 규칙

- **구조적 변경과 행동적 변경을 같은 커밋에 섞지 않는다**
- 둘 다 필요할 때는 **항상 구조적 변경을 먼저** 수행
- 구조적 변경 전후로 테스트를 실행하여 행동 변화 없음을 검증

## 커밋 규율

커밋은 다음 조건을 모두 만족할 때만:

1. **`cargo test` 모든 테스트 통과**
2. **`cargo clippy` 모든 경고 해결**
3. **`cargo fmt` 포맷팅 완료**
4. **단일 논리 단위의 작업**
5. **커밋 메시지에 구조적/행동적 변경 여부 명시**

작고 빈번한 커밋을 크고 드문 커밋보다 선호합니다.

## Rust 코드 품질 표준

- **중복 제거**: `DRY` 원칙, trait/generic 활용
- **의도 표현**: 명확한 타입과 함수명
- **에러 처리**: `Result<T, E>` 사용, `?` 연산자 활용
- **소유권**: 명확한 소유권 경계
- **상태 최소화**: immutable by default

## 체크리스트

### 기능 구현 전
- [ ] 실패하는 테스트 작성됨
- [ ] 테스트 이름이 행동을 설명함

### 구현 중
- [ ] 테스트 통과를 위한 최소 코드만 작성
- [ ] 모든 테스트가 Green 상태

### 커밋 전
- [ ] `cargo test` 통과
- [ ] `cargo clippy` 경고 없음
- [ ] `cargo fmt` 적용
- [ ] 구조적/행동적 변경 분리됨
- [ ] 커밋 메시지가 변경 유형 명시

**항상 한 번에 하나의 테스트만 작성하고, 실행하고, 구조를 개선하세요. 매번 모든 테스트를 실행하세요.**
