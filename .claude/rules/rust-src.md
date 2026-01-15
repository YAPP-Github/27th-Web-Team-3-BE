---
globs: codes/server/src/**/*.rs
---

# Rust 소스 코드 규칙

이 규칙은 `codes/server/src/` 하위의 모든 `.rs` 파일에 적용됩니다.

## 필수 사항

### 에러 처리
- `unwrap()` / `expect()` 사용 금지 (테스트 제외)
- 모든 에러는 `Result<T, AppError>` 반환
- `?` 연산자로 에러 전파

### 직렬화
- `#[serde(rename_all = "camelCase")]` 필수 (DTO)
- API 응답은 `BaseResponse` 래핑

### 로깅
- `tracing` 매크로 사용 (`info!`, `warn!`, `error!`)
- 민감 정보는 `#[instrument(skip(secret_key))]`로 제외

## 코드 스타일

```rust
// 좋은 예
pub async fn handler(
    State(state): State<AppState>,
    Json(req): Json<Request>,
) -> Result<Json<Response>, AppError> {
    let result = state.service.process(&req).await?;
    Ok(Json(result))
}

// 나쁜 예
pub async fn handler(state: State<AppState>, req: Json<Request>) -> Json<Response> {
    Json(state.service.process(&req.0).await.unwrap())
}
```

## 파일별 역할

- `handler.rs` - HTTP 핸들러만 (비즈니스 로직 없음)
- `service.rs` - 비즈니스 로직 (테스트 가능하게)
- `dto.rs` - Request/Response 구조체
- `error.rs` - 에러 타입 정의
