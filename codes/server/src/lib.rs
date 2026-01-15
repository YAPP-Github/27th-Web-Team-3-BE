pub mod config;
pub mod domain;
pub mod error;
pub mod global;
pub mod response;

use axum::{routing::get, Router};

pub use config::Config;
pub use domain::ai::client::{AiClient, AiClientTrait};
pub use domain::ai::service::AiService;
pub use global::validator::SecretKeyValidator;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub ai_service: AiService,
}

/// 테스트용 AppState 생성
pub fn create_test_app_state(secret_key: &str) -> AppState {
    // 테스트용 시작 시간 초기화
    domain::health::init_start_time();

    let config = Config {
        openai_api_key: "test-api-key".to_string(),
        ai_secret_key: secret_key.to_string(),
        server_port: 8080,
    };

    let secret_key_validator = SecretKeyValidator::new(config.ai_secret_key.clone());
    let ai_service = AiService::new(&config.openai_api_key, secret_key_validator);

    AppState { config, ai_service }
}

/// Mock 클라이언트로 테스트용 AppState 생성
pub fn create_test_app_state_with_mock<C: AiClientTrait + 'static>(
    secret_key: &str,
    mock_client: C,
) -> AppState {
    // 테스트용 시작 시간 초기화
    domain::health::init_start_time();

    let config = Config {
        openai_api_key: "test-api-key".to_string(),
        ai_secret_key: secret_key.to_string(),
        server_port: 8080,
    };

    let secret_key_validator = SecretKeyValidator::new(config.ai_secret_key.clone());
    let ai_service = AiService::with_client(mock_client, secret_key_validator);

    AppState { config, ai_service }
}

/// 테스트용 라우터 생성 (Rate Limiting 미포함)
///
/// 테스트 환경에서는 Rate Limiting이 제외됩니다.
/// IP 추출이 불가능한 테스트 요청에서의 panic을 방지합니다.
pub fn create_test_router(secret_key: &str) -> Router {
    let state = create_test_app_state(secret_key);

    Router::new()
        .route("/health", get(domain::health::health_check))
        .merge(domain::ai::router_without_rate_limit())
        .with_state(state)
}

/// Mock 클라이언트로 테스트용 라우터 생성 (Rate Limiting 미포함)
pub fn create_test_router_with_mock<C: AiClientTrait + 'static>(
    secret_key: &str,
    mock_client: C,
) -> Router {
    let state = create_test_app_state_with_mock(secret_key, mock_client);

    Router::new()
        .route("/health", get(domain::health::health_check))
        .merge(domain::ai::router_without_rate_limit())
        .with_state(state)
}
