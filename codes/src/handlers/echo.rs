use axum::Json;
use serde::{Deserialize, Serialize};

use crate::util::{AppError, ApiResponse};

/// Echo 요청 DTO
#[derive(Deserialize)]
pub struct EchoRequest {
    pub message: String,
}

/// Echo 응답 DTO
#[derive(Serialize)]
pub struct EchoResponse {
    pub echo: String,
}

/// Echo 핸들러
pub async fn echo(
    Json(payload): Json<EchoRequest>,
) -> Result<Json<ApiResponse<EchoResponse>>, AppError> {
    // 메시지 유효성 검사
    if payload.message.trim().is_empty() {
        return Err(AppError::validation_error("Message cannot be empty"));
    }

    let response = EchoResponse {
        echo: payload.message,
    };

    Ok(Json(ApiResponse::success(response)))
}

