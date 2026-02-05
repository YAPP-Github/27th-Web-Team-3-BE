use crate::config::AppConfig;
use crate::domain::ai::service::AiService;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: AppConfig,
    pub ai_service: AiService,
}
