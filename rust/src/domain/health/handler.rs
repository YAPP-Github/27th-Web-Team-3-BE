use axum::{extract::State, Json};

use super::dto::HealthStatus;
use super::service::check_health;
use crate::AppState;

/// 헬스체크 API
///
/// 서버 상태, 버전, 가동 시간, 의존성 상태를 반환합니다.
#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses(
        (status = 200, description = "헬스체크 성공", body = HealthStatus)
    )
)]
pub async fn health_check(State(state): State<AppState>) -> Json<HealthStatus> {
    let status = check_health(&state.ai_service).await;
    Json(status)
}
