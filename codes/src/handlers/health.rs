use axum::Json;
use serde::Serialize;

use crate::response::ApiResponse;

/// 루트 핸들러
pub async fn root() -> Json<ApiResponse<ServiceInfo>> {
    Json(ApiResponse::success(ServiceInfo {
        service: "Rust Server".to_string(),
        version: "0.1.0".to_string(),
    }))
}

/// 헬스 체크 핸들러
pub async fn health_check() -> Json<ApiResponse<HealthStatus>> {
    Json(ApiResponse::success(HealthStatus {
        status: "healthy".to_string(),
    }))
}

#[derive(Serialize)]
pub struct ServiceInfo {
    service: String,
    version: String,
}

#[derive(Serialize)]
pub struct HealthStatus {
    status: String,
}

