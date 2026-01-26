use sea_orm::*;
use chrono::Utc;
use crate::state::AppState;
use crate::utils::error::AppError;
use crate::domain::retrospect::entity::retro_room::{self, Entity as RetroRoom};
use crate::domain::member::entity::member_retro_room::{self, RoomRole};
use super::dto::{TeamCreateRequest, TeamCreateResponse};

pub struct RetrospectService;

impl RetrospectService {
    pub async fn create_team(
        state: AppState,
        member_id: i64,
        req: TeamCreateRequest,
    ) -> Result<TeamCreateResponse, AppError> {
        // 1. 팀 이름 중복 체크
        let existing_room = RetroRoom::find()
            .filter(retro_room::Column::Title.eq(&req.team_name))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("DB Error: {}", e)))?;

        if existing_room.is_some() {
            return Err(AppError::TeamNameDuplicate("이미 사용 중인 팀 이름입니다.".into()));
        }

        // 2. 초대 코드 생성 (형식: INV-XXXX-XXXX)
        let invite_code = Self::generate_invite_code();

        // 3. retro_room 생성
        let now = Utc::now().naive_utc();
        let retro_room_active = retro_room::ActiveModel {
            title: Set(req.team_name.clone()),
            description: Set(req.description),
            invition_url: Set(invite_code.clone()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let result = retro_room_active.insert(&state.db).await.map_err(|e| {
            AppError::InternalError(format!("팀 생성 실패: {}", e))
        })?;

        // 4. member_retro_room 생성 (Owner 권한 부여)
        let member_retro_room_active = member_retro_room::ActiveModel {
            member_id: Set(member_id),
            retrospect_room_id: Set(result.retrospect_room_id),
            role: Set(RoomRole::Owner),
            ..Default::default()
        };

        member_retro_room_active.insert(&state.db).await.map_err(|e| {
            AppError::InternalError(format!("팀 멤버 등록 실패: {}", e))
        })?;

        Ok(TeamCreateResponse {
            team_id: result.retrospect_room_id,
            team_name: result.title,
            invite_code: result.invition_url,
        })
    }

    fn generate_invite_code() -> String {
        // Simple random-ish code generator for now as rand/nanoid is not available
        let now = Utc::now().timestamp_nanos_opt().unwrap_or(0);
        let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut code = String::from("INV-");
        
        let mut n = now as u64;
        for i in 0..8 {
            if i == 4 {
                code.push('-');
            }
            let idx = (n % chars.len() as u64) as usize;
            code.push(chars.chars().nth(idx).unwrap());
            n /= chars.len() as u64;
        }
        code
    }
}
