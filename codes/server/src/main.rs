mod config;
mod domain;
mod state;
mod utils;

use axum::{routing::get, Router};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::config::AppConfig;
use crate::domain::auth::dto::{
    EmailLoginRequest, EmailLoginResponse, LogoutRequest, SignupRequest, SignupResponse,
    SocialLoginRequest, SocialLoginResponse, SuccessEmailLoginResponse, SuccessLogoutResponse,
    SuccessSignupResponse, SuccessSocialLoginResponse, SuccessTokenRefreshResponse,
    TokenRefreshRequest, TokenRefreshResponse,
};
use crate::state::AppState;
use crate::utils::{BaseResponse, ErrorResponse};

/// OpenAPI 문서 정의
#[derive(OpenApi)]
#[openapi(
    paths(
        health_check,
        domain::auth::handler::social_login,
        domain::auth::handler::signup,
        domain::auth::handler::refresh_token,
        domain::auth::handler::logout,
        domain::auth::handler::login_by_email,
        domain::auth::handler::auth_test
    ),
    components(
        schemas(
            ErrorResponse,
            HealthResponse,
            SuccessHealthResponse,
            SocialLoginRequest,
            SocialLoginResponse,
            SuccessSocialLoginResponse,
            SignupRequest,
            SignupResponse,
            SuccessSignupResponse,
            TokenRefreshRequest,
            TokenRefreshResponse,
            SuccessTokenRefreshResponse,
            LogoutRequest,
            SuccessLogoutResponse,
            EmailLoginRequest,
            EmailLoginResponse,
            SuccessEmailLoginResponse
        )
    ),
    tags(
        (name = "Health", description = "헬스 체크 API"),
        (name = "Auth", description = "인증 API")
    ),
    modifiers(&SecurityAddon),
    info(
        title = "회고록 서비스 API",
        version = "1.0.0",
        description = "회고록 서비스의 API 문서입니다."
    )
)]
struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
            )
        }
    }
}

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
        // [API-001] 소셜 로그인
        .route(
            "/api/v1/auth/social-login",
            axum::routing::post(domain::auth::handler::social_login),
        )
        // [API-002] 회원가입
        .route(
            "/api/v1/auth/signup",
            axum::routing::post(domain::auth::handler::signup),
        )
        // [API-003] 토큰 갱신
        .route(
            "/api/v1/auth/token/refresh",
            axum::routing::post(domain::auth::handler::refresh_token),
        )
        // [API-004] 로그아웃
        .route(
            "/api/v1/auth/logout",
            axum::routing::post(domain::auth::handler::logout),
        )
        // 테스트/개발용 API
        .route(
            "/api/auth/login/email",
            axum::routing::post(domain::auth::handler::login_by_email),
        )
        .route(
            "/api/auth/test",
            axum::routing::get(domain::auth::handler::auth_test),
        )
        // 하위 호환성을 위한 구 엔드포인트 (deprecated)
        .route("/api/auth/login", {
            #[allow(deprecated)]
            axum::routing::post(domain::auth::handler::login)
        })
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
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "서버 상태 정상", body = SuccessHealthResponse)
    ),
    tag = "Health"
)]
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

#[derive(serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
struct SuccessHealthResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: HealthResponse,
}
