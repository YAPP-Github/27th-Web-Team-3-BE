use axum::{
    extract::State,
    http::{header, StatusCode},
    Json,
};
use validator::Validate;

use crate::common::{error::AppError, response::ApiResponse};
use super::{
    jwt::{create_jwt, create_refresh_token},
    models::{AuthResponse, LoginRequest, RegisterRequest},
    service::AuthService,
};

/// 회원가입 핸들러
pub async fn register_handler(
    State(service): State<AuthService>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, AppError> {
    // 유효성 검사
    request
        .validate()
        .map_err(|e| AppError::ValidationError(format!("입력값 검증 실패: {}", e)))?;

    // 회원가입 처리
    let user = service.register(request)?;

    // JWT 토큰 생성
    let access_token = create_jwt(user.id.clone(), user.email.clone())?;
    let refresh_token = create_refresh_token(user.id.clone(), user.email.clone())?;

    let response = AuthResponse {
        access_token,
        refresh_token,
        user: user.to_dto(),
    };

    Ok(Json(ApiResponse::success_with_message(
        response,
        "회원가입이 완료되었습니다",
    )))
}

/// 로그인 핸들러
pub async fn login_handler(
    State(service): State<AuthService>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, AppError> {
    // 유효성 검사
    request
        .validate()
        .map_err(|e| AppError::ValidationError(format!("입력값 검증 실패: {}", e)))?;

    // 로그인 처리
    let user = service.login(request)?;

    // JWT 토큰 생성
    let access_token = create_jwt(user.id.clone(), user.email.clone())?;
    let refresh_token = create_refresh_token(user.id.clone(), user.email.clone())?;

    let response = AuthResponse {
        access_token,
        refresh_token,
        user: user.to_dto(),
    };

    Ok(Json(ApiResponse::success_with_message(
        response,
        "로그인 성공",
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::service::UserRepository;
    use axum::extract::State;

    #[tokio::test]
    async fn test_register_handler_success() {
        unsafe {
            std::env::set_var("SECRET_KEY", "test_secret_key_123");
        }

        let repo = UserRepository::new();
        let service = AuthService::new(repo);

        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            name: "Test User".to_string(),
        };

        let result = register_handler(State(service), Json(request)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_register_handler_invalid_email() {
        unsafe {
            std::env::set_var("SECRET_KEY", "test_secret_key_123");
        }

        let repo = UserRepository::new();
        let service = AuthService::new(repo);

        let request = RegisterRequest {
            email: "invalid-email".to_string(),
            password: "password123".to_string(),
            name: "Test User".to_string(),
        };

        let result = register_handler(State(service), Json(request)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_login_handler_success() {
        unsafe {
            std::env::set_var("SECRET_KEY", "test_secret_key_123");
        }

        let repo = UserRepository::new();
        let service = AuthService::new(repo);

        // 먼저 회원가입
        let register_req = RegisterRequest {
            email: "login@example.com".to_string(),
            password: "password123".to_string(),
            name: "Login User".to_string(),
        };
        service.register(register_req).unwrap();

        // 로그인
        let login_req = LoginRequest {
            email: "login@example.com".to_string(),
            password: "password123".to_string(),
        };

        let result = login_handler(State(service), Json(login_req)).await;
        assert!(result.is_ok());
    }
}

