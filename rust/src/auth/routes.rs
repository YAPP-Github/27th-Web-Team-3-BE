use axum::{
    routing::{get, post},
    Router, Json,
};

use crate::common::response::ApiResponse;
use super::{
    handlers::{login_handler, register_handler},
    middleware::AuthUser,
    models::UserDto,
    service::AuthService,
};

/// 인증 라우트 설정
pub fn auth_routes() -> Router<AuthService> {
    Router::new()
        .route("/register", post(register_handler))
        .route("/login", post(login_handler))
        .route("/me", get(get_current_user))
}

/// 현재 로그인한 사용자 정보 조회 (인증 필요)
async fn get_current_user(
    auth_user: AuthUser,
) -> Json<ApiResponse<UserDto>> {
    let user_dto = UserDto {
        id: auth_user.user_id,
        email: auth_user.email,
        name: "".to_string(), // 실제로는 DB에서 조회해야 함
    };

    Json(ApiResponse::success(user_dto))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_routes_creation() {
        // 라우터 생성 테스트 - 컴파일되면 성공
        let _routes = auth_routes();
        assert!(true);
    }
}

