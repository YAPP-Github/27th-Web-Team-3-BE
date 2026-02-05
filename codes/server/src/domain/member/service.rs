use chrono::{TimeZone, Utc};
use sea_orm::EntityTrait;
use tracing::info;

use super::dto::MemberProfileResponse;
use crate::domain::member::entity::member;
use crate::state::AppState;
use crate::utils::error::AppError;

pub struct MemberService;

impl MemberService {
    /// 회원 프로필 조회
    pub async fn get_profile(
        state: &AppState,
        member_id: i64,
    ) -> Result<MemberProfileResponse, AppError> {
        let member = member::Entity::find_by_id(member_id)
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .ok_or_else(|| AppError::MemberNotFound("존재하지 않는 사용자입니다.".to_string()))?;

        Ok(MemberProfileResponse {
            member_id: member.member_id,
            email: member.email,
            nickname: member.nickname,
            insight_count: member.insight_count,
            social_type: member.social_type,
            created_at: Utc.from_utc_datetime(&member.created_at),
        })
    }

    /// 회원 탈퇴 처리
    ///
    /// 사용자 계정 정보를 삭제하고, 연관 데이터(답변, 회고 등)는 유지합니다.
    /// - member: 삭제 (계정 정보, refresh_token 포함)
    /// - member_response, member_retro, member_retro_room: FK가 NULL로 설정됨 (ON DELETE SET NULL)
    pub async fn withdraw(state: AppState, member_id: i64) -> Result<(), AppError> {
        // 사용자 존재 여부 확인
        let member = member::Entity::find_by_id(member_id)
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        if member.is_none() {
            return Err(AppError::MemberNotFound(
                "존재하지 않는 사용자입니다.".to_string(),
            ));
        }

        // member 삭제 (연관 테이블의 member_id는 ON DELETE SET NULL로 자동 NULL 처리)
        member::Entity::delete_by_id(member_id)
            .exec(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        info!("Member {} has been withdrawn successfully", member_id);

        Ok(())
    }
}
