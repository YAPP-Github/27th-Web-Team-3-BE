use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use dotenv::dotenv;
use std::env;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod config;
mod domain;
mod error;
mod models;
mod rate_limiter;

use config::AppConfig;
use domain::ai::controller;
use domain::auth::controller as auth_controller;
use domain::test::controller as test_controller;
use rate_limiter::RateLimiter;

#[derive(OpenApi)]
#[openapi(
    paths(
        controller::provide_guide,
        controller::provide_retrospective_guide,
        controller::refine_retrospective,
        auth_controller::sign_up,
        test_controller::test_rate_limit,
    ),
    components(
        schemas(
            models::request::GuideRequest,
            models::request::RetrospectiveGuideRequest,
            models::request::RefineRequest,
            models::request::ToneStyle,
            models::request::SignUpRequest,
            models::response::GuideResponse,
            models::response::RetrospectiveGuideResponse,
            models::response::RefineResponse,
            models::response::SignUpResponse,
            models::response::BaseResponse<models::response::GuideResponse>,
            models::response::BaseResponse<models::response::RetrospectiveGuideResponse>,
            models::response::BaseResponse<models::response::RefineResponse>,
            models::response::BaseResponse<models::response::SignUpResponse>,
            domain::test::controller::TestRequest,
            domain::test::controller::TestResponse,
            models::response::BaseResponse<domain::test::controller::TestResponse>,
            error::ErrorResponse,
        )
    ),
    tags(
        (name = "AI", description = "AI 질문 API"),
        (name = "Auth", description = "인증 API"),
        (name = "Test", description = "테스트 API")
    )
)]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = AppConfig::from_env();

    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);

    log::info!("Starting server at {}:{}", host, port);

    let openapi = ApiDoc::openapi();

    // RateLimiter 초기화 (10 requests per 60 seconds)
    let rate_limiter = web::Data::new(RateLimiter::new(10, 60));

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .wrap(actix_web::middleware::Logger::default())
            .app_data(web::Data::new(config.clone()))
            .app_data(rate_limiter.clone())
            .service(
                web::scope("/api/ai")
                    .configure(controller::configure)
            )
            .service(
                web::scope("/api/auth")
                    .configure(auth_controller::configure)
            )
            .service(
                web::scope("/api/test")
                    .configure(test_controller::configure)
            )
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", openapi.clone())
            )
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}

