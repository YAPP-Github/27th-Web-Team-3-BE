use axum::{extract::State, Json};
use utoipa;
use validator::Validate;

use crate::state::AppState;
use crate::utils::error::AppError;
use crate::utils::BaseResponse;
use crate::utils::auth::AuthUser;
use super::dto::{LoginRequest, LoginResponse, EmailLoginRequest, SuccessLoginResponse};
use super::service::AuthService;

/// 인증 테스트 API
///
/// JWT 토큰이 유효한지 테스트합니다.
/// Authorization 헤더에 Bearer 토큰이 필요합니다.
#[utoipa::path(
    get,
    path = "/api/auth/test",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "인증 성공 (User ID 반환)", body = BaseResponse<String>),
        (status = 401, description = "인증 실패", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn auth_test(
    user: AuthUser,
) -> Json<BaseResponse<String>> {
    Json(BaseResponse::success(format!("User ID: {}", user.0.sub)))
}

/// 이메일 기반 로그인 (테스트/일반)
///
/// 비밀번호 없이 이메일만으로 로그인을 진행합니다. (존재하는 유저만 가능)
#[utoipa::path(
    post,
    path = "/api/auth/login/email",
    request_body = EmailLoginRequest,
    responses(
        (status = 200, description = "로그인 성공", body = SuccessLoginResponse),
        (status = 401, description = "존재하지 않는 사용자", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn login_by_email(
    State(state): State<AppState>,
    Json(req): Json<EmailLoginRequest>,
) -> Result<Json<BaseResponse<LoginResponse>>, AppError> {
    req.validate()?;
    
    let result = AuthService::login_by_email(state, req).await?;
    
    Ok(Json(BaseResponse::success(result)))
}

/// 소셜 로그인 및 회원가입
///
/// 카카오/구글 액세스 토큰을 받아 로그인 검증 후 JWT 토큰을 발급합니다.
/// 미가입 유저의 경우 자동으로 회원가입 처리됩니다.
#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "로그인 성공", body = SuccessLoginResponse),
        (status = 400, description = "잘못된 요청", body = ErrorResponse),
        (status = 401, description = "인증 실패 (유효하지 않은 토큰)", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<BaseResponse<LoginResponse>>, AppError> {
    req.validate()?;
    
    let result = AuthService::login(state, req).await?;
    
    Ok(Json(BaseResponse::success(result)))
}
