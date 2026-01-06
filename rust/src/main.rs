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

use config::AppConfig;
use domain::ai::controller;

#[derive(OpenApi)]
#[openapi(
    paths(
        controller::provide_guide,
        controller::refine_retrospective,
    ),
    components(
        schemas(
            models::request::GuideRequest,
            models::request::RefineRequest,
            models::request::ToneStyle,
            models::response::GuideResponse,
            models::response::RefineResponse,
            models::response::BaseResponse<models::response::GuideResponse>,
            models::response::BaseResponse<models::response::RefineResponse>,
            error::ErrorResponse,
        )
    ),
    tags(
        (name = "AI", description = "AI 질문 API")
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

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .wrap(actix_web::middleware::Logger::default())
            .app_data(web::Data::new(config.clone()))
            .service(
                web::scope("/api/ai")
                    .configure(controller::configure)
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

