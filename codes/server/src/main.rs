use axum::{
    extract::DefaultBodyLimit,
    http::{header, HeaderValue},
    middleware,
    routing::get,
    Router,
};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use std::time::Duration;
use tower_http::{set_header::SetResponseHeaderLayer, timeout::TimeoutLayer};
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod shutdown;

use web3_server::{
    domain::{
        self,
        ai::dto::{GuideRequest, GuideResponse, RefineRequest, RefineResponse, ToneStyle},
        health::dto::{CheckResult, HealthChecks, HealthState, HealthStatus},
    },
    global::middleware::request_tracing,
    response::{BaseResponse, ErrorResponse},
    AiService, AppState, Config, SecretKeyValidator,
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Web3 회고록 AI API",
        version = "0.1.0",
        description = "회고록 작성을 도와주는 AI 서비스 API",
        license(name = "MIT")
    ),
    paths(
        domain::ai::handler::provide_guide,
        domain::ai::handler::refine_retrospective,
        domain::health::handler::health_check,
    ),
    components(schemas(
        GuideRequest,
        GuideResponse,
        RefineRequest,
        RefineResponse,
        ToneStyle,
        HealthStatus,
        HealthState,
        HealthChecks,
        CheckResult,
        BaseResponse<GuideResponse>,
        BaseResponse<RefineResponse>,
        ErrorResponse,
    )),
    tags(
        (name = "AI", description = "AI 기반 회고 작성 지원 API"),
        (name = "Health", description = "서버 상태 확인 API")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    // 로깅 초기화 (환경에 따라 JSON 또는 pretty 포맷)
    setup_tracing();

    // Prometheus 메트릭 레코더 설치
    let prometheus_handle = PrometheusBuilder::new()
        .install_recorder()
        .expect("Failed to install Prometheus recorder");

    // 서버 시작 시간 초기화 (헬스체크 uptime 계산용)
    domain::health::init_start_time();

    // 설정 로드
    let config = Config::from_env().expect("Failed to load configuration");
    tracing::info!("Configuration loaded successfully");

    // AppState 생성
    let secret_key_validator = SecretKeyValidator::new(config.ai_secret_key.clone());
    let ai_service = AiService::new(&config.openai_api_key, secret_key_validator);

    let state = AppState {
        config: config.clone(),
        ai_service,
    };

    // Rate Limiting 설정: 초당 10 요청, 버스트 20
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(10)
            .burst_size(20)
            .finish()
            .expect("GovernorConfig should be valid"),
    );

    tracing::info!(
        per_second = 10,
        burst_size = 20,
        "Rate limiting configured"
    );

    // 라우터 설정
    let app = Router::new()
        .route("/health", get(domain::health::health_check))
        .route(
            "/metrics",
            get(move || {
                let handle = prometheus_handle.clone();
                async move { handle.render() }
            }),
        )
        .merge(domain::ai::router())
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(GovernorLayer {
            config: governor_conf,
        })
        // Request Body 크기 제한: 1MB
        .layer(DefaultBodyLimit::max(1024 * 1024))
        // 보안 헤더
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'self'"),
        ))
        // HTTP 요청 타임아웃: 30초
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(middleware::from_fn(request_tracing))
        .with_state(state);

    // 서버 시작
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!(port = config.server_port, "Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind TCP listener on {}", addr));

    if let Err(e) = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown::shutdown_signal())
        .await
    {
        tracing::error!(error = %e, "Server error occurred");
        std::process::exit(1);
    }

    tracing::info!("Server shutdown complete");
}

/// Setup tracing with environment-aware formatting
fn setup_tracing() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    // Check if we're in production (RUST_ENV=production)
    let is_production = std::env::var("RUST_ENV")
        .map(|v| v == "production")
        .unwrap_or(false);

    if is_production {
        // Production: JSON format for log aggregation
        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_span_events(FmtSpan::CLOSE),
            )
            .init();
    } else {
        // Development: Pretty format for readability
        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_span_events(FmtSpan::CLOSE)
                    .with_target(true),
            )
            .init();
    }
}
