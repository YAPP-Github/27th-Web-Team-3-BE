use axum::{extract::State, Json};
use utoipa;
use validator::Validate;

#[allow(unused_imports)]
use super::dto::{
    EmailLoginRequest, LoginRequest, LoginResponse, LogoutRequest, SuccessLoginResponse,
    SuccessLogoutResponse, SuccessTokenRefreshResponse, TokenRefreshRequest, TokenRefreshResponse,
};
use super::service::AuthService;
use crate::state::AppState;
use crate::utils::auth::AuthUser;
use crate::utils::error::AppError;
use crate::utils::BaseResponse;

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
pub async fn auth_test(user: AuthUser) -> Json<BaseResponse<String>> {
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

/// 로그아웃
///
/// Refresh Token을 무효화하여 로그아웃 처리합니다.
#[utoipa::path(
    post,
    path = "/api/auth/logout",
    request_body = LogoutRequest,
    responses(
        (status = 200, description = "로그아웃 성공", body = SuccessLogoutResponse),
        (status = 401, description = "유효하지 않은 토큰", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn logout(
    State(state): State<AppState>,
    Json(req): Json<LogoutRequest>,
) -> Result<Json<BaseResponse<()>>, AppError> {
    req.validate()?;

    AuthService::logout(state, req).await?;

    Ok(Json(BaseResponse::success(())))
}

/// 토큰 갱신
///
/// Refresh Token을 사용하여 새로운 Access Token과 Refresh Token을 발급받습니다.
/// Refresh Token Rotation이 적용되어 기존 Refresh Token은 무효화됩니다.
#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    request_body = TokenRefreshRequest,
    responses(
        (status = 200, description = "토큰 갱신 성공", body = SuccessTokenRefreshResponse),
        (status = 401, description = "유효하지 않은 토큰", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<TokenRefreshRequest>,
) -> Result<Json<BaseResponse<TokenRefreshResponse>>, AppError> {
    req.validate()?;

    let result = AuthService::refresh(state, req).await?;

    Ok(Json(BaseResponse::success(result)))
}
