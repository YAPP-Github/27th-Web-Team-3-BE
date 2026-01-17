mod config;
mod domain;
mod utils;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::AppConfig;
use crate::domain::ai::{refine_retrospective, AiService, AppState};
use crate::utils::BaseResponse;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 환경 변수 로드
    dotenvy::dotenv().ok();

    // 로깅 초기화
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        ))
        .init();

    // 설정 로드
    let config = AppConfig::from_env()?;
    let port = config.server_port;

    // 애플리케이션 상태 생성
    let ai_service = AiService::new(&config);
    let app_state = AppState { ai_service };

    // CORS 설정
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 라우터 구성
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/ai/retrospective/refine", post(refine_retrospective))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // 서버 시작
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!("Server running on http://0.0.0.0:{}", port);

    axum::serve(listener, app).await?;

    Ok(())
}

/// 헬스 체크 엔드포인트
async fn health_check() -> axum::Json<BaseResponse<HealthResponse>> {
    axum::Json(BaseResponse::success(HealthResponse {
        status: "healthy".to_string(),
    }))
}

#[derive(serde::Serialize)]
struct HealthResponse {
    status: String,
}
