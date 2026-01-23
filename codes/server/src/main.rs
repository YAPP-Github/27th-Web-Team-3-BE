mod config;
mod domain;
mod utils;
mod state;

use axum::{
    routing::get,
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::config::AppConfig;
use crate::utils::{BaseResponse, ErrorResponse};
use crate::state::AppState;

/// OpenAPI 문서 정의
#[derive(OpenApi)]
#[openapi(
    paths(
    ),
    components(
        schemas(
            ErrorResponse,
            HealthResponse
        )
    ),
    tags(
        (name = "Health", description = "헬스 체크 API")
    ),
    info(
        title = "회고록 서비스 API",
        version = "1.0.0",
        description = "회고록 서비스의 API 문서입니다."
    )
)]
struct ApiDoc;

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

    // DB 연결 및 테이블 생성 (Auto-Schema)
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = crate::config::establish_connection(&database_url).await?;

    // 애플리케이션 상태 생성
    let app_state = AppState { 
        db,
        config: config.clone(),
    };

    // CORS 설정
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 라우터 구성
    let app = Router::new()
        .route("/health", get(health_check))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
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

/// 헬스 체크 응답 DTO
#[derive(serde::Serialize, utoipa::ToSchema)]
struct HealthResponse {
    /// 서버 상태
    status: String,
}