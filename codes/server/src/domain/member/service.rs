use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
use tracing::info;

use crate::domain::member::entity::member;
use crate::domain::member::entity::refresh_token;
use crate::state::AppState;
use crate::utils::error::AppError;

pub struct MemberService;

impl MemberService {
    /// 회원 탈퇴 처리
    ///
    /// 사용자 계정 정보만 삭제하고, 연관 데이터(답변, 회고 등)는 유지합니다.
    /// - refresh_token: 삭제 (세션 정보)
    /// - member: 삭제 (계정 정보)
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

        // 트랜잭션으로 refresh_token과 member 삭제를 원자적으로 처리
        let txn = state
            .db
            .begin()
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // 1. refresh_token 삭제
        refresh_token::Entity::delete_many()
            .filter(refresh_token::Column::MemberId.eq(member_id))
            .exec(&txn)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // 2. member 삭제 (연관 테이블의 member_id는 ON DELETE SET NULL로 자동 NULL 처리)
        member::Entity::delete_by_id(member_id)
            .exec(&txn)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        txn.commit()
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        info!("Member {} has been withdrawn successfully", member_id);

        Ok(())
    }
}
