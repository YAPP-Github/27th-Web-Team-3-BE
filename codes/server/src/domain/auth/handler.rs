use axum::{extract::State, http::HeaderMap, Json};
use utoipa;
use validator::Validate;

use super::dto::{
    EmailLoginRequest, EmailLoginResponse, SignupRequest, SignupResponse, SocialLoginRequest,
    SocialLoginResponse,
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

/// 이메일 기반 로그인 (테스트/개발용)
///
/// 비밀번호 없이 이메일만으로 로그인을 진행합니다. (존재하는 유저만 가능)
#[utoipa::path(
    post,
    path = "/api/auth/login/email",
    request_body = EmailLoginRequest,
    responses(
        (status = 200, description = "로그인 성공", body = SuccessEmailLoginResponse),
        (status = 401, description = "존재하지 않는 사용자", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn login_by_email(
    State(state): State<AppState>,
    Json(req): Json<EmailLoginRequest>,
) -> Result<Json<BaseResponse<EmailLoginResponse>>, AppError> {
    req.validate()?;

    let result = AuthService::login_by_email(state, req).await?;

    Ok(Json(BaseResponse::success(result)))
}

/// [API-001] 소셜 로그인
///
/// 카카오/구글 액세스 토큰을 받아 로그인 검증 후 JWT 토큰을 발급합니다.
/// - 기존 회원: accessToken, refreshToken 발급
/// - 신규 회원: signupToken 발급 (회원가입 필요)
#[utoipa::path(
    post,
    path = "/api/v1/auth/social-login",
    request_body = SocialLoginRequest,
    responses(
        (status = 200, description = "로그인 성공 또는 신규 회원", body = SuccessSocialLoginResponse),
        (status = 400, description = "필수 파라미터 누락", body = ErrorResponse),
        (status = 401, description = "유효하지 않은 소셜 토큰", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn social_login(
    State(state): State<AppState>,
    Json(req): Json<SocialLoginRequest>,
) -> Result<Json<BaseResponse<SocialLoginResponse>>, AppError> {
    req.validate()?;

    let result = AuthService::social_login(state, req).await?;

    let (code, message) = if result.is_new_member {
        ("AUTH2001", "신규 회원입니다. 가입 절차를 진행해 주세요.")
    } else {
        ("COMMON200", "로그인에 성공하였습니다.")
    };

    Ok(Json(BaseResponse {
        is_success: true,
        code: code.to_string(),
        message: message.to_string(),
        result: Some(result),
    }))
}

/// [API-002] 회원가입
///
/// 소셜 로그인에서 발급받은 signupToken으로 회원가입을 완료합니다.
/// Authorization 헤더에 Bearer {signupToken}이 필요합니다.
#[utoipa::path(
    post,
    path = "/api/v1/auth/signup",
    request_body = SignupRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "회원가입 성공", body = SuccessSignupResponse),
        (status = 400, description = "유효성 검증 실패", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 409, description = "닉네임 중복", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn signup(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<SignupRequest>,
) -> Result<Json<BaseResponse<SignupResponse>>, AppError> {
    req.validate()?;

    // Authorization 헤더에서 signupToken 추출
    let auth_header = headers
        .get("Authorization")
        .ok_or_else(|| AppError::Unauthorized("인증 정보가 유효하지 않습니다.".into()))?
        .to_str()
        .map_err(|_| AppError::Unauthorized("인증 정보가 유효하지 않습니다.".into()))?;

    let signup_token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("인증 정보가 유효하지 않습니다.".into()))?;

    let result = AuthService::signup(state, req, signup_token).await?;

    Ok(Json(BaseResponse {
        is_success: true,
        code: "COMMON200".to_string(),
        message: "회원가입이 성공적으로 완료되었습니다.".to_string(),
        result: Some(result),
    }))
}

// 하위 호환성을 위한 별칭
#[allow(deprecated)]
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<SocialLoginRequest>,
) -> Result<Json<BaseResponse<SocialLoginResponse>>, AppError> {
    social_login(State(state), Json(req)).await
}
