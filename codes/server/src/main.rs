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
    EmailLoginRequest, LoginRequest, LoginResponse, SuccessLoginResponse,
};
use crate::domain::member::entity::member_retro::RetrospectStatus;
use crate::domain::retrospect::dto::{
    AnalysisResponse, CreateParticipantResponse, CreateRetrospectRequest, CreateRetrospectResponse,
    DraftItem, DraftSaveRequest, DraftSaveResponse, EmotionRankItem, MissionItem,
    PersonalMissionItem, ReferenceItem, ResponseCategory, ResponseListItem, ResponsesListResponse,
    RetrospectDetailResponse, RetrospectMemberItem, RetrospectQuestionItem, SearchRetrospectItem,
    StorageRangeFilter, StorageResponse, StorageRetrospectItem, StorageYearGroup, SubmitAnswerItem,
    SubmitRetrospectRequest, SubmitRetrospectResponse, SuccessAnalysisResponse,
    SuccessCreateParticipantResponse, SuccessCreateRetrospectResponse,
    SuccessDeleteRetrospectResponse, SuccessDraftSaveResponse, SuccessReferencesListResponse,
    SuccessResponsesListResponse, SuccessRetrospectDetailResponse, SuccessSearchResponse,
    SuccessStorageResponse, SuccessSubmitRetrospectResponse, SuccessTeamRetrospectListResponse,
    TeamRetrospectListItem,
};
use crate::domain::retrospect::entity::retrospect::RetrospectMethod;
use crate::state::AppState;
use crate::utils::{BaseResponse, ErrorResponse};

/// OpenAPI 문서 정의
#[derive(OpenApi)]
#[openapi(
    paths(
        health_check,
        domain::auth::handler::login,
        domain::auth::handler::login_by_email,
        domain::auth::handler::auth_test,
        domain::retrospect::handler::create_retrospect,
        domain::retrospect::handler::list_team_retrospects,
        domain::retrospect::handler::create_participant,
        domain::retrospect::handler::list_references,
        domain::retrospect::handler::save_draft,
        domain::retrospect::handler::get_retrospect_detail,
        domain::retrospect::handler::submit_retrospect,
        domain::retrospect::handler::get_storage,
        domain::retrospect::handler::analyze_retrospective_handler,
        domain::retrospect::handler::search_retrospects,
        domain::retrospect::handler::list_responses,
        domain::retrospect::handler::export_retrospect,
        domain::retrospect::handler::delete_retrospect
    ),
    components(
        schemas(
            ErrorResponse,
            HealthResponse,
            SuccessHealthResponse,
            LoginRequest,
            LoginResponse,
            EmailLoginRequest,
            SuccessLoginResponse,
            CreateRetrospectRequest,
            CreateRetrospectResponse,
            SuccessCreateRetrospectResponse,
            TeamRetrospectListItem,
            SuccessTeamRetrospectListResponse,
            RetrospectMethod,
            CreateParticipantResponse,
            SuccessCreateParticipantResponse,
            ReferenceItem,
            SuccessReferencesListResponse,
            DraftSaveRequest,
            DraftItem,
            DraftSaveResponse,
            SuccessDraftSaveResponse,
            SubmitRetrospectRequest,
            SubmitRetrospectResponse,
            SubmitAnswerItem,
            SuccessSubmitRetrospectResponse,
            RetrospectStatus,
            StorageRangeFilter,
            StorageRetrospectItem,
            StorageYearGroup,
            StorageResponse,
            SuccessStorageResponse,
            RetrospectDetailResponse,
            RetrospectMemberItem,
            RetrospectQuestionItem,
            SuccessRetrospectDetailResponse,
            AnalysisResponse,
            EmotionRankItem,
            MissionItem,
            PersonalMissionItem,
            SuccessAnalysisResponse,
            SearchRetrospectItem,
            SuccessSearchResponse,
            SuccessDeleteRetrospectResponse,
            ResponseCategory,
            ResponseListItem,
            ResponsesListResponse,
            SuccessResponsesListResponse
        )
    ),
    tags(
        (name = "Health", description = "헬스 체크 API"),
        (name = "Auth", description = "인증 API"),
        (name = "Retrospect", description = "회고 API")
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

    // AI 서비스 초기화
    let ai_service = domain::ai::service::AiService::new(&config);

    // 애플리케이션 상태 생성
    let app_state = AppState {
        db,
        config: config.clone(),
        ai_service,
    };

    // CORS 설정
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 라우터 구성
    let app = Router::new()
        .route("/health", get(health_check))
        .route(
            "/api/auth/login",
            axum::routing::post(domain::auth::handler::login),
        )
        .route(
            "/api/auth/login/email",
            axum::routing::post(domain::auth::handler::login_by_email),
        )
        .route(
            "/api/auth/test",
            axum::routing::get(domain::auth::handler::auth_test),
        )
        .route(
            "/api/v1/retrospects",
            axum::routing::post(domain::retrospect::handler::create_retrospect),
        )
        .route(
            "/api/v1/teams/:team_id/retrospects",
            axum::routing::get(domain::retrospect::handler::list_team_retrospects),
        )
        .route(
            "/api/v1/retrospects/:retrospect_id/participants",
            axum::routing::post(domain::retrospect::handler::create_participant),
        )
        .route(
            "/api/v1/retrospects/:retrospect_id/references",
            axum::routing::get(domain::retrospect::handler::list_references),
        )
        .route(
            "/api/v1/retrospects/search",
            axum::routing::get(domain::retrospect::handler::search_retrospects),
        )
        .route(
            "/api/v1/retrospects/storage",
            axum::routing::get(domain::retrospect::handler::get_storage),
        )
        .route(
            "/api/v1/retrospects/:retrospect_id",
            axum::routing::get(domain::retrospect::handler::get_retrospect_detail)
                .delete(domain::retrospect::handler::delete_retrospect),
        )
        .route(
            "/api/v1/retrospects/:retrospect_id/drafts",
            axum::routing::put(domain::retrospect::handler::save_draft),
        )
        .route(
            "/api/v1/retrospects/:retrospect_id/submit",
            axum::routing::post(domain::retrospect::handler::submit_retrospect),
        )
        .route(
            "/api/v1/retrospects/:retrospect_id/analysis",
            axum::routing::post(domain::retrospect::handler::analyze_retrospective_handler),
        )
        .route(
            "/api/v1/retrospects/:retrospect_id/responses",
            axum::routing::get(domain::retrospect::handler::list_responses),
        )
        .route(
            "/api/v1/retrospects/:retrospect_id/export",
            axum::routing::get(domain::retrospect::handler::export_retrospect),
        )
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
