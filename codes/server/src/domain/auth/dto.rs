use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

use crate::domain::member::entity::member::SocialType;

/// 닉네임 유효성 검증 (특수문자 제외)
/// 한글, 영문, 숫자만 허용
fn validate_nickname(nickname: &str) -> Result<(), ValidationError> {
    for c in nickname.chars() {
        if !c.is_alphanumeric() && !is_korean(c) {
            return Err(ValidationError::new("nickname_invalid_chars"));
        }
    }
    Ok(())
}

/// 한글 문자 여부 확인 (가-힣, ㄱ-ㅎ, ㅏ-ㅣ)
fn is_korean(c: char) -> bool {
    matches!(c, '\u{AC00}'..='\u{D7A3}' | '\u{3131}'..='\u{314E}' | '\u{314F}'..='\u{3163}')
}

/// [API-001] 소셜 로그인 요청 DTO
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SocialLoginRequest {
    /// 소셜 서비스 구분 (GOOGLE, KAKAO)
    pub provider: SocialType,

    /// 소셜 서비스에서 발급받은 Access Token
    #[validate(length(min = 1, message = "accessToken은 필수입니다"))]
    pub access_token: String,
}

/// [API-001] 소셜 로그인 응답 DTO
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SocialLoginResponse {
    /// 신규 회원 여부
    pub is_new_member: bool,
    /// 서비스 Access Token (기존 회원인 경우)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    /// 서비스 Refresh Token (기존 회원인 경우)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// 소셜 계정 이메일 (신규 회원인 경우)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// 회원가입용 임시 토큰 (신규 회원인 경우)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signup_token: Option<String>,
}

/// [API-002] 회원가입 요청 DTO
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SignupRequest {
    /// 소셜 로그인에서 반환받은 이메일
    #[validate(email(message = "이메일 형식이 올바르지 않습니다"))]
    pub email: String,

    /// 사용자 닉네임 (1~20자, 특수문자 제외)
    #[validate(
        length(min = 1, max = 20, message = "닉네임은 1~20자 이내로 입력해야 합니다"),
        custom(
            function = "validate_nickname",
            message = "닉네임에 특수문자를 사용할 수 없습니다"
        )
    )]
    pub nickname: String,
}

/// [API-002] 회원가입 응답 DTO
/// 토큰은 쿠키로 전달됩니다.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SignupResponse {
    /// 생성된 회원 ID
    pub member_id: i64,
    /// 설정된 닉네임
    pub nickname: String,
}

// --- [API-003] 토큰 갱신 ---

/// [API-003] 토큰 갱신 요청 DTO
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TokenRefreshRequest {
    /// 로그인 또는 회원가입 시 발급받은 Refresh Token
    #[validate(length(min = 1, message = "refreshToken은 필수입니다"))]
    pub refresh_token: String,
}

/// [API-003] 토큰 갱신 응답 DTO
/// 토큰은 쿠키로 전달됩니다.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TokenRefreshResponse {
    // 토큰은 쿠키로 전달되므로 본문은 비어있음
}

// --- [API-004] 로그아웃 ---

/// [API-004] 로그아웃 요청 DTO
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LogoutRequest {
    /// 서버에서 무효화 처리할 Refresh Token
    #[validate(length(min = 1, message = "refreshToken은 필수입니다"))]
    pub refresh_token: String,
}

// --- 이메일 로그인 (테스트용) ---

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EmailLoginRequest {
    #[validate(email(message = "이메일 형식이 올바르지 않습니다"))]
    pub email: String,
}

/// 이메일 로그인 응답 DTO (테스트용)
/// 토큰은 쿠키로 전달됩니다.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EmailLoginResponse {
    pub is_new_member: bool,
}

// --- Swagger용 래핑 DTO ---

/// 소셜 로그인 성공 응답 (Swagger용)
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessSocialLoginResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: SocialLoginResponse,
}

/// 회원가입 성공 응답 (Swagger용)
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessSignupResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: SignupResponse,
}

/// 이메일 로그인 성공 응답 (Swagger용)
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessEmailLoginResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: EmailLoginResponse,
}

/// 토큰 갱신 성공 응답 (Swagger용)
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessTokenRefreshResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: TokenRefreshResponse,
}

/// 로그아웃 성공 응답 (Swagger용)
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessLogoutResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: Option<()>,
}

// --- 하위 호환성을 위한 별칭 ---

/// LoginRequest 별칭 (하위 호환성)
#[allow(dead_code)]
pub type LoginRequest = SocialLoginRequest;

/// LoginResponse 별칭 (하위 호환성)
#[allow(dead_code)]
pub type LoginResponse = SocialLoginResponse;

/// SuccessLoginResponse 별칭 (하위 호환성)
#[allow(dead_code)]
pub type SuccessLoginResponse = SuccessSocialLoginResponse;
