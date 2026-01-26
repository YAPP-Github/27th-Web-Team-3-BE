use super::dto::{
    JoinRetroRoomRequest, JoinRetroRoomResponse, RetroRoomCreateRequest, RetroRoomCreateResponse,
};
use crate::domain::member::entity::member_retro_room::{self, Entity as MemberRetroRoom, RoomRole};
use crate::domain::retrospect::entity::retro_room::{self, Entity as RetroRoom};
use crate::state::AppState;
use crate::utils::error::AppError;
use chrono::Utc;
use sea_orm::*;

pub struct RetrospectService;

impl RetrospectService {
    pub async fn create_retro_room(
        state: AppState,
        member_id: i64,
        req: RetroRoomCreateRequest,
    ) -> Result<RetroRoomCreateResponse, AppError> {
        // 1. 회고 룸 이름 중복 체크
        let existing_room = RetroRoom::find()
            .filter(retro_room::Column::Title.eq(&req.title))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("DB Error: {}", e)))?;

        if existing_room.is_some() {
            return Err(AppError::RetroRoomNameDuplicate(
                "이미 사용 중인 회고 룸 이름입니다.".into(),
            ));
        }

        // 2. 초대 코드 생성 (형식: INV-XXXX-XXXX)
        let invite_code = Self::generate_invite_code();

        // 3. retro_room 생성
        let now = Utc::now().naive_utc();
        let retro_room_active = retro_room::ActiveModel {
            title: Set(req.title.clone()),
            description: Set(req.description),
            invition_url: Set(invite_code.clone()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let result = retro_room_active
            .insert(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("회고 룸 생성 실패: {}", e)))?;

        // 4. member_retro_room 생성 (Owner 권한 부여)
        let member_retro_room_active = member_retro_room::ActiveModel {
            member_id: Set(member_id),
            retrospect_room_id: Set(result.retrospect_room_id),
            role: Set(RoomRole::Owner),
            ..Default::default()
        };

        member_retro_room_active
            .insert(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("회고 룸 멤버 등록 실패: {}", e)))?;

        Ok(RetroRoomCreateResponse {
            retro_room_id: result.retrospect_room_id,
            title: result.title,
            invite_code: result.invition_url,
        })
    }

    pub async fn join_retro_room(
        state: AppState,
        member_id: i64,
        req: JoinRetroRoomRequest,
    ) -> Result<JoinRetroRoomResponse, AppError> {
        // 1. 초대 코드 추출 (path segment 또는 query parameter 지원)
        let invite_code = Self::extract_invite_code(&req.invite_url)?;

        // 2. 초대 코드로 룸 조회
        let room = RetroRoom::find()
            .filter(retro_room::Column::InvitionUrl.eq(invite_code))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("DB Error: {}", e)))?;

        let room = room.ok_or_else(|| AppError::NotFound("존재하지 않는 회고 룸입니다.".into()))?;

        // 3. 만료 체크 (7일)
        let now = Utc::now().naive_utc();
        let diff = now.signed_duration_since(room.created_at);
        if diff.num_days() >= 7 {
            return Err(AppError::ExpiredInviteLink(
                "만료된 초대 링크입니다. 룸 관리자에게 새로운 초대 링크를 요청해주세요.".into(),
            ));
        }

        // 4. 이미 참여 중인지 체크
        let existing_member = MemberRetroRoom::find()
            .filter(member_retro_room::Column::MemberId.eq(member_id))
            .filter(member_retro_room::Column::RetrospectRoomId.eq(room.retrospect_room_id))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("DB Error: {}", e)))?;

        if existing_member.is_some() {
            return Err(AppError::AlreadyMember(
                "이미 해당 회고 룸의 멤버입니다.".into(),
            ));
        }

        // 5. 멤버 추가
        let member_retro_room_active = member_retro_room::ActiveModel {
            member_id: Set(member_id),
            retrospect_room_id: Set(room.retrospect_room_id),
            role: Set(RoomRole::Member),
            ..Default::default()
        };

        member_retro_room_active
            .insert(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("멤버 추가 실패: {}", e)))?;

        Ok(JoinRetroRoomResponse {
            retro_room_id: room.retrospect_room_id,
            title: room.title,
            joined_at: Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
        })
    }

    fn generate_invite_code() -> String {
        // Simple random-ish code generator
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

    /// 초대 URL에서 초대 코드를 추출합니다.
    ///
    /// 지원 형식:
    /// - Path segment: `https://service.com/invite/INV-A1B2-C3D4`
    /// - Query parameter: `https://service.com/join?code=INV-A1B2-C3D4`
    fn extract_invite_code(invite_url: &str) -> Result<String, AppError> {
        // 1. 쿼리 파라미터 형식 확인 (?code=...)
        if let Some(query_start) = invite_url.find('?') {
            let query_string = &invite_url[query_start + 1..];
            for param in query_string.split('&') {
                if let Some((key, value)) = param.split_once('=') {
                    if key == "code" && value.starts_with("INV-") {
                        return Ok(value.to_string());
                    }
                }
            }
        }

        // 2. Path segment 형식 확인 (/invite/INV-...)
        // 쿼리 파라미터 제거 후 마지막 경로 세그먼트 추출
        let path = invite_url.split('?').next().unwrap_or(invite_url);
        if let Some(last_segment) = path.split('/').next_back() {
            if last_segment.starts_with("INV-") {
                return Ok(last_segment.to_string());
            }
        }

        Err(AppError::InvalidInviteLink(
            "유효하지 않은 초대 링크입니다.".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_extract_invite_code_from_path_segment() {
        // Arrange
        let url = "https://service.com/invite/INV-A1B2-C3D4";

        // Act
        let result = RetrospectService::extract_invite_code(url);

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "INV-A1B2-C3D4");
    }

    #[test]
    fn should_extract_invite_code_from_query_parameter() {
        // Arrange
        let url = "https://service.com/join?code=INV-A1B2-C3D4";

        // Act
        let result = RetrospectService::extract_invite_code(url);

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "INV-A1B2-C3D4");
    }

    #[test]
    fn should_extract_invite_code_from_query_with_multiple_params() {
        // Arrange
        let url = "https://service.com/join?ref=abc&code=INV-TEST-1234&foo=bar";

        // Act
        let result = RetrospectService::extract_invite_code(url);

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "INV-TEST-1234");
    }

    #[test]
    fn should_return_error_for_invalid_url() {
        // Arrange
        let url = "https://service.com/invalid/path";

        // Act
        let result = RetrospectService::extract_invite_code(url);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn should_return_error_for_empty_code() {
        // Arrange
        let url = "https://service.com/join?code=";

        // Act
        let result = RetrospectService::extract_invite_code(url);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn should_generate_valid_invite_code() {
        // Act
        let code = RetrospectService::generate_invite_code();

        // Assert
        assert!(code.starts_with("INV-"));
        assert!(code.len() > 4);
    }
}
