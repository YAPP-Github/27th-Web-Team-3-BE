pub mod domain;
pub mod utils;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        domain::ai::handler::guide_handler,
    ),
    components(
        schemas(
            domain::ai::dto::RetrospectiveGuideRequest,
            domain::ai::dto::RetrospectiveGuideResult,
            domain::ai::dto::RetrospectiveGuideResponse,
            utils::response::ErrorResponse,
        )
    ),
    tags(
        (name = "AI", description = "AI 관련 API")
    )
)]
pub struct ApiDoc;

pub fn app() -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/health", get(|| async { "OK" }))
        .route(
            "/api/ai/retrospective/guide",
            post(domain::ai::handler::guide_handler),
        )
        .layer(TraceLayer::new_for_http())
}
