use axum::{
    extract::State,
    http::{header::COOKIE, HeaderMap},
    response::IntoResponse,
    Json,
};
use utoipa;
use validator::Validate;

use super::dto::{
    EmailLoginRequest, EmailLoginResponse, LogoutRequest, SignupRequest, SignupResponse,
    SocialLoginRequest, TokenRefreshRequest, TokenRefreshResponse,
};
use super::service::AuthService;
use crate::state::AppState;
use crate::utils::auth::AuthUser;
use crate::utils::cookie::{
    clear_access_token_cookie, clear_refresh_token_cookie, clear_signup_token_cookie,
    create_access_token_cookie, create_refresh_token_cookie, create_signup_token_cookie,
    set_cookie_header, REFRESH_TOKEN_COOKIE, SIGNUP_TOKEN_COOKIE,
};
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
/// 성공 시 accessToken, refreshToken을 쿠키로 설정합니다.
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
) -> Result<impl IntoResponse, AppError> {
    req.validate()?;

    let jwt_expiration = state.config.jwt_expiration;
    let refresh_token_expiration = state.config.refresh_token_expiration;
    let result = AuthService::login_by_email(state, req).await?;

    // 쿠키 설정
    let mut headers = HeaderMap::new();
    headers.insert(
        set_cookie_header(),
        create_access_token_cookie(&result.access_token, jwt_expiration),
    );
    headers.append(
        set_cookie_header(),
        create_refresh_token_cookie(&result.refresh_token, refresh_token_expiration),
    );

    // 응답 본문에는 토큰 제외
    let response_body = Json(BaseResponse::success(EmailLoginResponse {
        is_new_member: result.is_new_member,
    }));

    Ok((headers, response_body))
}

/// 소셜 로그인 API (API-001)
///
/// 카카오/구글 액세스 토큰을 받아 로그인 검증 후 JWT 토큰을 발급합니다.
/// - 기존 회원: accessToken, refreshToken 쿠키로 발급
/// - 신규 회원: signupToken 쿠키로 발급 (회원가입 필요)
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
) -> Result<impl IntoResponse, AppError> {
    req.validate()?;

    let jwt_expiration = state.config.jwt_expiration;
    let refresh_token_expiration = state.config.refresh_token_expiration;
    let signup_token_expiration = state.config.signup_token_expiration;
    let result = AuthService::social_login(state, req).await?;

    let (code, message) = if result.is_new_member {
        ("AUTH2001", "신규 회원입니다. 가입 절차를 진행해 주세요.")
    } else {
        ("COMMON200", "로그인에 성공하였습니다.")
    };

    let mut headers = HeaderMap::new();

    if result.is_new_member {
        // 신규 회원: signup_token 쿠키 설정
        if let Some(signup_token) = &result.signup_token {
            headers.insert(
                set_cookie_header(),
                create_signup_token_cookie(signup_token, signup_token_expiration),
            );
        }
        // 응답 본문에서 토큰 제거
        let response_body = Json(BaseResponse {
            is_success: true,
            code: code.to_string(),
            message: message.to_string(),
            result: Some(super::dto::SocialLoginResponse {
                is_new_member: true,
                access_token: None,
                refresh_token: None,
                email: result.email.clone(),
                signup_token: None, // 쿠키로 전달하므로 본문에서 제거
            }),
        });
        Ok((headers, response_body))
    } else {
        // 기존 회원: access_token, refresh_token 쿠키 설정
        if let (Some(access_token), Some(refresh_token)) =
            (&result.access_token, &result.refresh_token)
        {
            headers.insert(
                set_cookie_header(),
                create_access_token_cookie(access_token, jwt_expiration),
            );
            headers.append(
                set_cookie_header(),
                create_refresh_token_cookie(refresh_token, refresh_token_expiration),
            );
        }
        // 응답 본문에서 토큰 제거
        let response_body = Json(BaseResponse {
            is_success: true,
            code: code.to_string(),
            message: message.to_string(),
            result: Some(super::dto::SocialLoginResponse {
                is_new_member: false,
                access_token: None,  // 쿠키로 전달하므로 본문에서 제거
                refresh_token: None, // 쿠키로 전달하므로 본문에서 제거
                email: None,
                signup_token: None,
            }),
        });
        Ok((headers, response_body))
    }
}

/// 회원가입 API (API-002)
///
/// 소셜 로그인에서 발급받은 signupToken으로 회원가입을 완료합니다.
/// signupToken은 쿠키 또는 Authorization 헤더에서 읽습니다.
/// 성공 시 accessToken, refreshToken을 쿠키로 설정합니다.
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
) -> Result<impl IntoResponse, AppError> {
    req.validate()?;

    // 쿠키 또는 Authorization 헤더에서 signupToken 추출
    let signup_token = extract_signup_token_from_cookie_or_header(&headers)?;

    let jwt_expiration = state.config.jwt_expiration;
    let refresh_token_expiration = state.config.refresh_token_expiration;
    let result = AuthService::signup(state, req, &signup_token).await?;

    // 쿠키 설정: access_token, refresh_token 추가, signup_token 삭제
    let mut response_headers = HeaderMap::new();
    response_headers.insert(
        set_cookie_header(),
        create_access_token_cookie(&result.access_token, jwt_expiration),
    );
    response_headers.append(
        set_cookie_header(),
        create_refresh_token_cookie(&result.refresh_token, refresh_token_expiration),
    );
    response_headers.append(set_cookie_header(), clear_signup_token_cookie());

    // 응답 본문에는 토큰 제외
    let response_body = Json(BaseResponse {
        is_success: true,
        code: "COMMON200".to_string(),
        message: "회원가입이 성공적으로 완료되었습니다.".to_string(),
        result: Some(SignupResponse {
            member_id: result.member_id,
            nickname: result.nickname,
        }),
    });

    Ok((response_headers, response_body))
}

/// 토큰 갱신 API (API-003)
///
/// 만료된 Access Token을 Refresh Token을 이용하여 재발급합니다.
/// Refresh Token은 쿠키 또는 요청 본문에서 읽습니다.
/// Refresh Token Rotation 정책에 따라 새로운 Refresh Token도 함께 발급됩니다.
#[utoipa::path(
    post,
    path = "/api/v1/auth/token/refresh",
    request_body = TokenRefreshRequest,
    responses(
        (status = 200, description = "토큰 갱신 성공", body = SuccessTokenRefreshResponse),
        (status = 400, description = "필수 파라미터 누락", body = ErrorResponse),
        (status = 401, description = "유효하지 않거나 만료된 Refresh Token", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn refresh_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Option<Json<TokenRefreshRequest>>,
) -> Result<impl IntoResponse, AppError> {
    // 쿠키 또는 요청 본문에서 refresh_token 추출
    let refresh_token = extract_refresh_token_from_cookie_or_body(&headers, body)?;

    let req = TokenRefreshRequest { refresh_token };
    req.validate()?;

    let jwt_expiration = state.config.jwt_expiration;
    let refresh_token_expiration = state.config.refresh_token_expiration;
    let result = AuthService::refresh_token(state, req).await?;

    // 쿠키 설정
    let mut response_headers = HeaderMap::new();
    response_headers.insert(
        set_cookie_header(),
        create_access_token_cookie(&result.access_token, jwt_expiration),
    );
    response_headers.append(
        set_cookie_header(),
        create_refresh_token_cookie(&result.refresh_token, refresh_token_expiration),
    );

    // 응답 본문에는 토큰 제외
    let response_body = Json(BaseResponse {
        is_success: true,
        code: "COMMON200".to_string(),
        message: "토큰이 성공적으로 갱신되었습니다.".to_string(),
        result: Some(TokenRefreshResponse {}),
    });

    Ok((response_headers, response_body))
}

/// 로그아웃 API (API-029)
///
/// 현재 사용자의 로그아웃을 처리합니다.
/// 서버에 저장된 Refresh Token을 무효화하고 쿠키를 삭제합니다.
/// Refresh Token은 쿠키 또는 요청 본문에서 읽습니다.
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    request_body = LogoutRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "로그아웃 성공", body = SuccessLogoutResponse),
        (status = 400, description = "유효하지 않은 토큰", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn logout(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    body: Option<Json<LogoutRequest>>,
) -> Result<impl IntoResponse, AppError> {
    // 쿠키 또는 요청 본문에서 refresh_token 추출
    let refresh_token = extract_refresh_token_from_cookie_or_body(&headers, body)?;

    let req = LogoutRequest { refresh_token };
    req.validate()?;

    let user_id: i64 = user
        .0
        .sub
        .parse()
        .map_err(|_| AppError::Unauthorized("잘못된 인증 정보입니다.".into()))?;

    AuthService::logout(state, req, user_id).await?;

    // 쿠키 삭제
    let mut response_headers = HeaderMap::new();
    response_headers.insert(set_cookie_header(), clear_access_token_cookie());
    response_headers.append(set_cookie_header(), clear_refresh_token_cookie());

    let response_body = Json(BaseResponse {
        is_success: true,
        code: "COMMON200".to_string(),
        message: "로그아웃이 성공적으로 처리되었습니다.".to_string(),
        result: None::<()>,
    });

    Ok((response_headers, response_body))
}

/// 쿠키 또는 요청 본문에서 refresh_token 추출
fn extract_refresh_token_from_cookie_or_body<T: HasRefreshToken>(
    headers: &HeaderMap,
    body: Option<Json<T>>,
) -> Result<String, AppError> {
    // 1. 요청 본문에서 먼저 시도
    if let Some(Json(req)) = body {
        let token = req.get_refresh_token();
        if !token.is_empty() {
            return Ok(token);
        }
    }

    // 2. 쿠키에서 시도
    if let Some(cookie_header) = headers.get(COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if let Some(value) = cookie.strip_prefix(&format!("{}=", REFRESH_TOKEN_COOKIE)) {
                    if !value.is_empty() {
                        return Ok(value.to_string());
                    }
                }
            }
        }
    }

    Err(AppError::InvalidToken(
        "Refresh Token이 필요합니다.".to_string(),
    ))
}

/// refresh_token을 가진 타입을 위한 트레이트
trait HasRefreshToken {
    fn get_refresh_token(&self) -> String;
}

impl HasRefreshToken for TokenRefreshRequest {
    fn get_refresh_token(&self) -> String {
        self.refresh_token.clone()
    }
}

impl HasRefreshToken for LogoutRequest {
    fn get_refresh_token(&self) -> String {
        self.refresh_token.clone()
    }
}

/// 쿠키 또는 Authorization 헤더에서 signup_token 추출
fn extract_signup_token_from_cookie_or_header(headers: &HeaderMap) -> Result<String, AppError> {
    // 1. Authorization 헤더에서 먼저 시도
    if let Some(auth_header) = headers.get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if !token.is_empty() {
                    return Ok(token.to_string());
                }
            }
        }
    }

    // 2. 쿠키에서 시도
    if let Some(cookie_header) = headers.get(COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if let Some(value) = cookie.strip_prefix(&format!("{}=", SIGNUP_TOKEN_COOKIE)) {
                    if !value.is_empty() {
                        return Ok(value.to_string());
                    }
                }
            }
        }
    }

    Err(AppError::Unauthorized(
        "인증 정보가 유효하지 않습니다.".to_string(),
    ))
}

// 하위 호환성을 위한 별칭
#[deprecated(note = "Use social_login instead")]
#[allow(dead_code)]
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<SocialLoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    social_login(State(state), Json(req)).await
}
