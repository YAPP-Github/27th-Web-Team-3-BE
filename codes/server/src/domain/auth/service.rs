use chrono::Utc;
use reqwest::Client;
use sea_orm::*;

use super::dto::{
    EmailLoginRequest, EmailLoginResponse, SignupRequest, SignupResponse, SocialLoginRequest,
    SocialLoginResponse,
};
use crate::domain::member::entity::member::{self, Entity as Member, SocialType};
use crate::state::AppState;
use crate::utils::error::AppError;
use crate::utils::jwt::{decode_token, encode_refresh_token, encode_signup_token, encode_token};

pub struct AuthService;

#[derive(Debug)]
struct SocialUserInfo {
    email: String,
}

impl AuthService {
    /// [API-001] 소셜 로그인
    pub async fn social_login(
        state: AppState,
        req: SocialLoginRequest,
    ) -> Result<SocialLoginResponse, AppError> {
        // 1. 소셜 제공자로부터 유저 정보 가져오기
        let social_info = match req.provider {
            SocialType::Kakao => Self::fetch_kakao_user_info(&req.access_token).await?,
            SocialType::Google => Self::fetch_google_user_info(&req.access_token).await?,
        };

        // 2. DB에서 유저 조회 (이메일 + 소셜 타입)
        let member = Member::find()
            .filter(member::Column::Email.eq(&social_info.email))
            .filter(member::Column::SocialType.eq(req.provider.clone()))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("DB Error: {}", e)))?;

        match member {
            Some(member) => {
                // 기존 회원: Access/Refresh Token 발급
                let access_token = encode_token(
                    member.member_id.to_string(),
                    &state.config.jwt_secret,
                    state.config.jwt_expiration,
                )?;

                let refresh_token = encode_refresh_token(
                    member.member_id.to_string(),
                    &state.config.jwt_secret,
                    state.config.refresh_token_expiration,
                )?;

                Ok(SocialLoginResponse {
                    is_new_member: false,
                    access_token: Some(access_token),
                    refresh_token: Some(refresh_token),
                    email: None,
                    signup_token: None,
                })
            }
            None => {
                // 신규 회원: Signup Token 발급
                let signup_token = encode_signup_token(
                    social_info.email.clone(),
                    &state.config.jwt_secret,
                    state.config.signup_token_expiration,
                )?;

                Ok(SocialLoginResponse {
                    is_new_member: true,
                    access_token: None,
                    refresh_token: None,
                    email: Some(social_info.email),
                    signup_token: Some(signup_token),
                })
            }
        }
    }

    /// [API-002] 회원가입
    pub async fn signup(
        state: AppState,
        req: SignupRequest,
        signup_token: &str,
    ) -> Result<SignupResponse, AppError> {
        // 1. signupToken 검증
        let claims = decode_token(signup_token, &state.config.jwt_secret)?;

        // 2. 토큰 타입 확인
        if claims.token_type.as_deref() != Some("signup") {
            return Err(AppError::Unauthorized(
                "유효하지 않은 토큰 타입입니다.".into(),
            ));
        }

        // 3. 이메일 일치 여부 확인
        let token_email = claims
            .email
            .ok_or_else(|| AppError::Unauthorized("토큰에 이메일 정보가 없습니다.".into()))?;

        if token_email != req.email {
            return Err(AppError::Unauthorized(
                "이메일 정보가 일치하지 않습니다.".into(),
            ));
        }

        // 4. 닉네임 중복 확인
        let existing_nickname = Member::find()
            .filter(member::Column::Nickname.eq(&req.nickname))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("DB Error: {}", e)))?;

        if existing_nickname.is_some() {
            return Err(AppError::Conflict("이미 사용 중인 닉네임입니다.".into()));
        }

        // 5. 회원 생성
        let active_model = member::ActiveModel {
            email: Set(req.email),
            nickname: Set(Some(req.nickname.clone())),
            social_type: Set(SocialType::Kakao), // TODO: signupToken에 provider 정보 포함 필요
            insight_count: Set(0),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
            ..Default::default()
        };

        let new_member = active_model
            .insert(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("회원가입 실패: {}", e)))?;

        // 6. JWT 토큰 발급
        let access_token = encode_token(
            new_member.member_id.to_string(),
            &state.config.jwt_secret,
            state.config.jwt_expiration,
        )?;

        let refresh_token = encode_refresh_token(
            new_member.member_id.to_string(),
            &state.config.jwt_secret,
            state.config.refresh_token_expiration,
        )?;

        Ok(SignupResponse {
            member_id: new_member.member_id,
            nickname: req.nickname,
            access_token,
            refresh_token,
        })
    }

    /// 이메일 기반 로그인 (테스트/개발용)
    pub async fn login_by_email(
        state: AppState,
        req: EmailLoginRequest,
    ) -> Result<EmailLoginResponse, AppError> {
        // DB에서 유저 조회 (이메일 기반)
        let member = Member::find()
            .filter(member::Column::Email.eq(&req.email))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("DB Error: {}", e)))?;

        let member =
            member.ok_or_else(|| AppError::Unauthorized("존재하지 않는 사용자입니다.".into()))?;

        // JWT 발급
        let access_token = encode_token(
            member.member_id.to_string(),
            &state.config.jwt_secret,
            state.config.jwt_expiration,
        )?;

        let refresh_token = encode_refresh_token(
            member.member_id.to_string(),
            &state.config.jwt_secret,
            state.config.refresh_token_expiration,
        )?;

        Ok(EmailLoginResponse {
            is_new_member: false,
            access_token: Some(access_token),
            refresh_token: Some(refresh_token),
        })
    }

    /// 하위 호환성을 위한 login 메서드 (deprecated)
    #[deprecated(note = "Use social_login instead")]
    #[allow(dead_code)]
    pub async fn login(
        state: AppState,
        req: SocialLoginRequest,
    ) -> Result<SocialLoginResponse, AppError> {
        Self::social_login(state, req).await
    }

    async fn fetch_kakao_user_info(token: &str) -> Result<SocialUserInfo, AppError> {
        let client = Client::new();
        let response = client
            .get("https://kapi.kakao.com/v2/user/me")
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("Kakao API req failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::SocialAuthFailed(
                "유효하지 않은 소셜 토큰입니다.".into(),
            ));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::JsonParseFailed(e.to_string()))?;

        let email = json["kakao_account"]["email"]
            .as_str()
            .ok_or(AppError::ValidationError("Kakao 이메일 정보 없음".into()))?
            .to_string();

        Ok(SocialUserInfo { email })
    }

    async fn fetch_google_user_info(token: &str) -> Result<SocialUserInfo, AppError> {
        let client = Client::new();
        let response = client
            .get("https://www.googleapis.com/oauth2/v2/userinfo")
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("Google API req failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::SocialAuthFailed(
                "유효하지 않은 소셜 토큰입니다.".into(),
            ));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::JsonParseFailed(e.to_string()))?;

        let email = json["email"]
            .as_str()
            .ok_or(AppError::ValidationError("Google 이메일 정보 없음".into()))?
            .to_string();

        Ok(SocialUserInfo { email })
    }
}
