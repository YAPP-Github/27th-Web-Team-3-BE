mod common;
mod auth;

use axum::{Router, routing::get};
use dotenv::dotenv;
use std::env;
use tower_http::cors::CorsLayer;
use tracing_subscriber;

use auth::service::{AuthService, UserRepository};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 환경 변수 로드
    dotenv().ok();

    // 로깅 초기화
    tracing_subscriber::fmt()
        .with_env_filter(env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()))
        .init();

    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("{}:{}", host, port);

    // AuthService 초기화
    let user_repository = UserRepository::new();
    let auth_service = AuthService::new(user_repository);

    // 라우터 설정
    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/api/auth", auth::routes::auth_routes())
        .with_state(auth_service)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("🚀 Server running on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

/// 헬스 체크 엔드포인트
async fn health_check() -> &'static str {
    "OK"
}
