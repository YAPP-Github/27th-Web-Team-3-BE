---
globs: codes/server/src/**/*test*.rs, codes/server/tests/**/*.rs
---

# Rust 테스트 규칙

테스트 파일에 적용되는 규칙입니다.

## 테스트 구조

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // 테스트 이름은 행동을 설명
    #[test]
    fn should_return_error_for_empty_input() {
        // Arrange - 준비
        let input = "";

        // Act - 실행
        let result = validate(input);

        // Assert - 검증
        assert!(result.is_err());
        assert!(matches!(result, Err(AppError::ValidationError(_))));
    }
}
```

## 비동기 테스트

```rust
#[tokio::test]
async fn should_call_api_successfully() {
    // ...
}
```

## 테스트 커버리지

- 모든 public 함수에 최소 1개 테스트
- 정상 케이스 + 에러 케이스 모두 테스트
- 엣지 케이스 (빈 값, 최대값, 특수문자)

## 허용 사항

테스트 코드에서는 다음이 허용됩니다:
- `unwrap()` / `expect()` 사용 가능
- 하드코딩된 테스트 데이터
- 임시 파일/디렉토리 생성
