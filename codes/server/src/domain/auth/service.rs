use sea_orm::*;
use reqwest::Client;
use chrono::Utc;

use crate::state::AppState;
use crate::utils::error::AppError;
use crate::utils::jwt::encode_token;
use crate::domain::member::entity::member::{self, Entity as Member, Model as MemberModel, SocialType};
use super::dto::{LoginRequest, LoginResponse, EmailLoginRequest};

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

        let member = member.ok_or_else(|| {
            AppError::Unauthorized("존재하지 않는 사용자입니다.".into())
        })?;

        // JWT 발급
        let token = encode_token(
            member.member_id.to_string(),
            &state.config.jwt_secret,
            state.config.jwt_expiration,
        )?;

        Ok(LoginResponse {
            access_token: token,
            token_type: "Bearer".to_string(),
            expires_in: state.config.jwt_expiration,
        })
    }

    pub async fn login(
        state: AppState,
        req: LoginRequest,
    ) -> Result<LoginResponse, AppError> {
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

        // 4. JWT 발급
        let token = encode_token(
            member.member_id.to_string(),
            &state.config.jwt_secret,
            state.config.jwt_expiration,
        )?;

        Ok(LoginResponse {
            access_token: token,
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

        active_model.insert(db).await.map_err(|e| {
            AppError::InternalError(format!("회원가입 실패: {}", e))
        })
    }
}
