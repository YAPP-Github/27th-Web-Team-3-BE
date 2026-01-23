use sea_orm::DatabaseConnection;
use crate::domain::ai::AiService;

#[derive(Clone)]
pub struct AppState {
    pub ai_service: AiService,
    pub db: DatabaseConnection,
}
