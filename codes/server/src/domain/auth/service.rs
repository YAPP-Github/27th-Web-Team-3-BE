use chrono::{Duration, Utc};
use reqwest::Client;
use sea_orm::{DbErr, RuntimeErr, *};

use super::dto::{
    EmailLoginRequest, EmailLoginResponse, LogoutRequest, SignupRequest, SignupResponse,
    SocialLoginRequest, SocialLoginResponse, TokenRefreshRequest, TokenRefreshResponse,
};
use crate::domain::member::entity::member::{self, Entity as Member, SocialType};
use crate::domain::member::entity::refresh_token::{self, Entity as RefreshToken};
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

                let refresh_token_str = encode_refresh_token(
                    member.member_id.to_string(),
                    &state.config.jwt_secret,
                    state.config.refresh_token_expiration,
                )?;

                // Refresh Token DB 저장
                Self::store_refresh_token(
                    &state.db,
                    member.member_id,
                    &refresh_token_str,
                    state.config.refresh_token_expiration,
                )
                .await?;

                Ok(SocialLoginResponse {
                    is_new_member: false,
                    access_token: Some(access_token),
                    refresh_token: Some(refresh_token_str),
                    email: None,
                    signup_token: None,
                })
            }
            None => {
                // 신규 회원: Signup Token 발급 (provider 정보 포함)
                let provider_str = match req.provider {
                    SocialType::Kakao => "KAKAO",
                    SocialType::Google => "GOOGLE",
                };
                let signup_token = encode_signup_token(
                    social_info.email.clone(),
                    provider_str.to_string(),
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

        // 4. provider 정보 추출
        let social_type = match claims.provider.as_deref() {
            Some("KAKAO") => SocialType::Kakao,
            Some("GOOGLE") => SocialType::Google,
            _ => {
                return Err(AppError::Unauthorized(
                    "토큰에 유효한 provider 정보가 없습니다.".into(),
                ))
            }
        };

        // 5. 닉네임 중복 확인
        let existing_nickname = Member::find()
            .filter(member::Column::Nickname.eq(&req.nickname))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("DB Error: {}", e)))?;

        if existing_nickname.is_some() {
            return Err(AppError::Conflict("이미 사용 중인 닉네임입니다.".into()));
        }

        // 6. 회원 생성
        let active_model = member::ActiveModel {
            email: Set(req.email),
            nickname: Set(Some(req.nickname.clone())),
            social_type: Set(social_type),
            insight_count: Set(0),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
            ..Default::default()
        };

        let new_member = active_model.insert(&state.db).await.map_err(|e| {
            // DB unique constraint 위반 시 Conflict 에러 반환 (TOCTOU 경쟁 조건 방어)
            if let DbErr::Query(RuntimeErr::SqlxError(sqlx_err)) = &e {
                let err_str = sqlx_err.to_string();
                if err_str.contains("UNIQUE") || err_str.contains("duplicate") {
                    return AppError::Conflict("이미 사용 중인 닉네임입니다.".into());
                }
            }
            AppError::InternalError(format!("회원가입 실패: {}", e))
        })?;

        // 7. JWT 토큰 발급
        let access_token = encode_token(
            new_member.member_id.to_string(),
            &state.config.jwt_secret,
            state.config.jwt_expiration,
        )?;

        let refresh_token_str = encode_refresh_token(
            new_member.member_id.to_string(),
            &state.config.jwt_secret,
            state.config.refresh_token_expiration,
        )?;

        // 8. Refresh Token DB 저장
        Self::store_refresh_token(
            &state.db,
            new_member.member_id,
            &refresh_token_str,
            state.config.refresh_token_expiration,
        )
        .await?;

        Ok(SignupResponse {
            member_id: new_member.member_id,
            nickname: req.nickname,
            access_token,
            refresh_token: refresh_token_str,
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

        let refresh_token_str = encode_refresh_token(
            member.member_id.to_string(),
            &state.config.jwt_secret,
            state.config.refresh_token_expiration,
        )?;

        // Refresh Token DB 저장
        Self::store_refresh_token(
            &state.db,
            member.member_id,
            &refresh_token_str,
            state.config.refresh_token_expiration,
        )
        .await?;

        Ok(EmailLoginResponse {
            is_new_member: false,
            access_token: Some(access_token),
            refresh_token: Some(refresh_token_str),
        })
    }

    /// [API-003] 토큰 갱신
    pub async fn refresh_token(
        state: AppState,
        req: TokenRefreshRequest,
    ) -> Result<TokenRefreshResponse, AppError> {
        // 1. Refresh Token JWT 검증
        let claims = decode_token(&req.refresh_token, &state.config.jwt_secret).map_err(|_| {
            AppError::InvalidRefreshToken("유효하지 않거나 만료된 Refresh Token입니다.".into())
        })?;

        // 2. 토큰 타입 확인
        if claims.token_type.as_deref() != Some("refresh") {
            return Err(AppError::InvalidRefreshToken(
                "유효하지 않거나 만료된 Refresh Token입니다.".into(),
            ));
        }

        // 3. DB에서 해당 Refresh Token 존재 확인
        let stored_token = RefreshToken::find()
            .filter(refresh_token::Column::Token.eq(&req.refresh_token))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("DB Error: {}", e)))?;

        let stored_token = stored_token.ok_or_else(|| {
            AppError::InvalidRefreshToken("유효하지 않거나 만료된 Refresh Token입니다.".into())
        })?;

        // 4. 만료 시간 확인
        if stored_token.expires_at < Utc::now().naive_utc() {
            // 만료된 토큰 삭제
            RefreshToken::delete_by_id(stored_token.refresh_token_id)
                .exec(&state.db)
                .await
                .ok();
            return Err(AppError::InvalidRefreshToken(
                "유효하지 않거나 만료된 Refresh Token입니다.".into(),
            ));
        }

        let member_id = stored_token.member_id;

        // 5. 기존 Refresh Token 삭제 (Rotation)
        RefreshToken::delete_by_id(stored_token.refresh_token_id)
            .exec(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("Refresh Token 삭제 실패: {}", e)))?;

        // 6. 새 토큰 발급
        let new_access_token = encode_token(
            member_id.to_string(),
            &state.config.jwt_secret,
            state.config.jwt_expiration,
        )?;

        let new_refresh_token = encode_refresh_token(
            member_id.to_string(),
            &state.config.jwt_secret,
            state.config.refresh_token_expiration,
        )?;

        // 7. 새 Refresh Token DB 저장
        Self::store_refresh_token(
            &state.db,
            member_id,
            &new_refresh_token,
            state.config.refresh_token_expiration,
        )
        .await?;

        Ok(TokenRefreshResponse {
            access_token: new_access_token,
            refresh_token: new_refresh_token,
        })
    }

    /// [API-004] 로그아웃
    pub async fn logout(state: AppState, req: LogoutRequest, user_id: i64) -> Result<(), AppError> {
        // 1. Refresh Token JWT 검증
        let claims = decode_token(&req.refresh_token, &state.config.jwt_secret).map_err(|_| {
            AppError::InvalidToken("이미 로그아웃되었거나 유효하지 않은 토큰입니다.".into())
        })?;

        // 2. 토큰 타입 확인
        if claims.token_type.as_deref() != Some("refresh") {
            return Err(AppError::InvalidToken(
                "이미 로그아웃되었거나 유효하지 않은 토큰입니다.".into(),
            ));
        }

        // 3. Access Token의 user_id와 Refresh Token의 sub 일치 확인
        // 타인의 Refresh Token을 무효화하는 공격 방지
        if claims.sub != user_id.to_string() {
            return Err(AppError::InvalidToken(
                "토큰 소유자가 일치하지 않습니다.".into(),
            ));
        }

        // 4. DB에서 해당 Refresh Token 삭제
        let delete_result = RefreshToken::delete_many()
            .filter(refresh_token::Column::Token.eq(&req.refresh_token))
            .exec(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("Refresh Token 삭제 실패: {}", e)))?;

        // 토큰이 DB에 없어도 이미 로그아웃된 것으로 간주하고 성공 반환
        if delete_result.rows_affected == 0 {
            tracing::info!("Refresh token not found in DB, possibly already logged out");
        }

        Ok(())
    }

    /// Refresh Token을 DB에 저장
    async fn store_refresh_token(
        db: &DatabaseConnection,
        member_id: i64,
        token: &str,
        expiration_seconds: i64,
    ) -> Result<(), AppError> {
        let expires_at = Utc::now()
            .checked_add_signed(Duration::seconds(expiration_seconds))
            .ok_or_else(|| AppError::InternalError("유효하지 않은 토큰 만료 시간입니다.".into()))?
            .naive_utc();

        let active_model = refresh_token::ActiveModel {
            member_id: Set(member_id),
            token: Set(token.to_string()),
            expires_at: Set(expires_at),
            created_at: Set(Utc::now().naive_utc()),
            ..Default::default()
        };

        active_model
            .insert(db)
            .await
            .map_err(|e| AppError::InternalError(format!("Refresh Token 저장 실패: {}", e)))?;

        Ok(())
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
