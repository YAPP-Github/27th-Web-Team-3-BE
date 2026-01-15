---
globs: rust/src/domain/**/handler.rs, rust/src/domain/**/dto.rs
---

# API 설계 규칙

API 핸들러와 DTO에 적용되는 규칙입니다.

## DTO 구조

### Request
```rust
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MyRequest {
    #[validate(length(min = 1, message = "필수 입력입니다"))]
    pub content: String,

    #[validate(length(min = 1))]
    pub secret_key: String,
}
```

### Response
```rust
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MyResponse {
    pub result_field: String,
}
```

## 응답 형식

모든 API 응답은 다음 형식을 따릅니다:

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공입니다.",
  "result": { ... }
}
```

## 핸들러 패턴

```rust
#[utoipa::path(
    post,
    path = "/api/endpoint",
    request_body = MyRequest,
    responses(
        (status = 200, body = BaseResponse<MyResponse>),
        (status = 400, body = ErrorResponse)
    )
)]
pub async fn my_handler(
    State(state): State<AppState>,
    Json(req): Json<MyRequest>,
) -> Result<Json<BaseResponse<MyResponse>>, AppError> {
    req.validate()?;
    let result = state.service.process(&req).await?;
    Ok(Json(BaseResponse::success(result)))
}
```

## 에러 코드

| 코드 | HTTP | 용도 |
|------|------|------|
| AI_001 | 401 | 인증 실패 |
| AI_002 | 400 | 유효하지 않은 입력 |
| COMMON400 | 400 | 일반 요청 오류 |
| COMMON500 | 500 | 서버 오류 |
