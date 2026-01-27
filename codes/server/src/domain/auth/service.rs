use chrono::{Duration, Utc};
use reqwest::Client;
use sea_orm::*;

use super::dto::{
    EmailLoginRequest, LoginRequest, LoginResponse, LogoutRequest, TokenRefreshRequest,
    TokenRefreshResponse,
};
use crate::domain::member::entity::member::{
    self, Entity as Member, Model as MemberModel, SocialType,
};
use crate::domain::member::entity::refresh_token::{self, Entity as RefreshToken};
use crate::state::AppState;
use crate::utils::error::AppError;
use crate::utils::jwt::{decode_refresh_token, encode_access_token, encode_refresh_token};

pub struct AuthService;

#[derive(Debug)]
struct SocialUserInfo {
    email: String,
    // social_id: String, // 필요한 경우 사용
}

impl AuthService {
    pub async fn login_by_email(
        state: AppState,
        req: EmailLoginRequest,
    ) -> Result<LoginResponse, AppError> {
        // DB에서 유저 조회 (이메일 기반, 일반 타입 우선 혹은 전체 검색)
        let member = Member::find()
            .filter(member::Column::Email.eq(&req.email))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("DB Error: {}", e)))?;

        let member =
            member.ok_or_else(|| AppError::Unauthorized("존재하지 않는 사용자입니다.".into()))?;

        // Access Token 발급
        let access_token = encode_access_token(
            member.member_id.to_string(),
            &state.config.jwt_secret,
            state.config.jwt_expiration,
        )?;

        // Refresh Token 발급
        let refresh_token_str = encode_refresh_token(
            member.member_id.to_string(),
            &state.config.jwt_secret,
            state.config.refresh_token_expiration,
        )?;

        // Refresh Token DB 저장
        Self::store_refresh_token(&state.db, member.member_id, &refresh_token_str, state.config.refresh_token_expiration).await?;

        Ok(LoginResponse {
            access_token,
            refresh_token: refresh_token_str,
            token_type: "Bearer".to_string(),
            expires_in: state.config.jwt_expiration,
        })
    }

    pub async fn login(state: AppState, req: LoginRequest) -> Result<LoginResponse, AppError> {
        // 1. 소셜 제공자로부터 유저 정보 가져오기
        let social_info = match req.social_type {
            SocialType::Kakao => Self::fetch_kakao_user_info(&req.token).await?,
            SocialType::Google => Self::fetch_google_user_info(&req.token).await?,
        };

        // 2. DB에서 유저 조회 (이메일 + 소셜 타입)
        let member = Member::find()
            .filter(member::Column::Email.eq(&social_info.email))
            .filter(member::Column::SocialType.eq(req.social_type.clone()))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("DB Error: {}", e)))?;

        let member = match member {
            Some(m) => m,
            None => {
                // 3. 없으면 회원가입
                Self::register(&state.db, social_info.email, req.social_type).await?
            }
        };

        // 4. Access Token 발급
        let access_token = encode_access_token(
            member.member_id.to_string(),
            &state.config.jwt_secret,
            state.config.jwt_expiration,
        )?;

        // 5. Refresh Token 발급
        let refresh_token_str = encode_refresh_token(
            member.member_id.to_string(),
            &state.config.jwt_secret,
            state.config.refresh_token_expiration,
        )?;

        // 6. Refresh Token DB 저장
        Self::store_refresh_token(&state.db, member.member_id, &refresh_token_str, state.config.refresh_token_expiration).await?;

        Ok(LoginResponse {
            access_token,
            refresh_token: refresh_token_str,
            token_type: "Bearer".to_string(),
            expires_in: state.config.jwt_expiration,
        })
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
            return Err(AppError::Unauthorized("Kakao 로그인 실패".into()));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::JsonParseFailed(e.to_string()))?;

        // Kakao 응답 구조: { "kakao_account": { "email": "...", ... }, ... }
        let email = json["kakao_account"]["email"]
            .as_str()
            .ok_or(AppError::ValidationError("Kakao 이메일 정보 없음".into()))?
            .to_string();

        Ok(SocialUserInfo { email })
    }

    async fn fetch_google_user_info(token: &str) -> Result<SocialUserInfo, AppError> {
        let client = Client::new();
        // Google UserInfo API
        let response = client
            .get("https://www.googleapis.com/oauth2/v2/userinfo")
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("Google API req failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::Unauthorized("Google 로그인 실패".into()));
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

    /// Refresh Token을 DB에 저장
    async fn store_refresh_token(
        db: &DatabaseConnection,
        member_id: i64,
        token: &str,
        expiration_seconds: i64,
    ) -> Result<(), AppError> {
        let expires_at = Utc::now()
            .checked_add_signed(Duration::seconds(expiration_seconds))
            .expect("valid timestamp")
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

    /// 로그아웃 - Refresh Token 삭제
    pub async fn logout(state: AppState, req: LogoutRequest) -> Result<(), AppError> {
        // 1. Refresh Token 검증
        let _claims = decode_refresh_token(&req.refresh_token, &state.config.jwt_secret)?;

        // 2. DB에서 해당 Refresh Token 삭제
        let delete_result = RefreshToken::delete_many()
            .filter(refresh_token::Column::Token.eq(&req.refresh_token))
            .exec(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("Refresh Token 삭제 실패: {}", e)))?;

        if delete_result.rows_affected == 0 {
            return Err(AppError::Unauthorized(
                "유효하지 않은 리프레시 토큰입니다.".into(),
            ));
        }

        Ok(())
    }

    /// 토큰 갱신 - Refresh Token으로 새 Access Token 발급
    pub async fn refresh(
        state: AppState,
        req: TokenRefreshRequest,
    ) -> Result<TokenRefreshResponse, AppError> {
        // 1. Refresh Token JWT 검증
        let claims = decode_refresh_token(&req.refresh_token, &state.config.jwt_secret)?;

        // 2. DB에서 해당 Refresh Token 존재 확인
        let stored_token = RefreshToken::find()
            .filter(refresh_token::Column::Token.eq(&req.refresh_token))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("DB Error: {}", e)))?;

        let stored_token = stored_token.ok_or_else(|| {
            AppError::Unauthorized("유효하지 않은 리프레시 토큰입니다.".into())
        })?;

        // 3. 만료 시간 확인
        if stored_token.expires_at < Utc::now().naive_utc() {
            // 만료된 토큰 삭제
            RefreshToken::delete_by_id(stored_token.refresh_token_id)
                .exec(&state.db)
                .await
                .ok();
            return Err(AppError::Unauthorized("리프레시 토큰이 만료되었습니다.".into()));
        }

        // 4. 기존 Refresh Token 삭제
        RefreshToken::delete_by_id(stored_token.refresh_token_id)
            .exec(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("Refresh Token 삭제 실패: {}", e)))?;

        // 5. 새 Access Token 발급
        let new_access_token = encode_access_token(
            claims.sub.clone(),
            &state.config.jwt_secret,
            state.config.jwt_expiration,
        )?;

        // 6. 새 Refresh Token 발급 (Rotation)
        let new_refresh_token = encode_refresh_token(
            claims.sub,
            &state.config.jwt_secret,
            state.config.refresh_token_expiration,
        )?;

        // 7. 새 Refresh Token DB 저장
        Self::store_refresh_token(
            &state.db,
            stored_token.member_id,
            &new_refresh_token,
            state.config.refresh_token_expiration,
        )
        .await?;

        Ok(TokenRefreshResponse {
            access_token: new_access_token,
            refresh_token: new_refresh_token,
        })
    }

    async fn register(
        db: &DatabaseConnection,
        email: String,
        social_type: SocialType,
    ) -> Result<MemberModel, AppError> {
        let active_model = member::ActiveModel {
            email: Set(email),
            social_type: Set(social_type),
            insight_count: Set(0),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
            ..Default::default()
        };

        active_model
            .insert(db)
            .await
            .map_err(|e| AppError::InternalError(format!("회원가입 실패: {}", e)))
    }
}
