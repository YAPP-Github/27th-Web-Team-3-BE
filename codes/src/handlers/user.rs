use axum::extract::Path;
use axum::Json;
use serde::Serialize;
use std::collections::HashMap;

use crate::util::{AppError, ApiResponse};

/// 사용자 정보 DTO
#[derive(Serialize, Clone)]
pub struct UserInfo {
    pub id: u32,
    pub name: String,
    pub email: String,
    pub role: String,
}

/// Mock 사용자 데이터 생성
fn get_mock_users() -> HashMap<u32, UserInfo> {
    let mut users = HashMap::new();

    users.insert(1, UserInfo {
        id: 1,
        name: "홍길동".to_string(),
        email: "hong@example.com".to_string(),
        role: "admin".to_string(),
    });

    users.insert(2, UserInfo {
        id: 2,
        name: "김철수".to_string(),
        email: "kim@example.com".to_string(),
        role: "user".to_string(),
    });

    users.insert(3, UserInfo {
        id: 3,
        name: "이영희".to_string(),
        email: "lee@example.com".to_string(),
        role: "user".to_string(),
    });

    users
}

/// 사용자 정보 조회 핸들러
pub async fn get_user_info(
    Path(user_id): Path<u32>,
) -> Result<Json<ApiResponse<UserInfo>>, AppError> {
    let users = get_mock_users();

    match users.get(&user_id) {
        Some(user) => Ok(Json(ApiResponse::success(user.clone()))),
        None => Err(AppError::not_found(format!("User not found: id={}", user_id))),
    }
}

