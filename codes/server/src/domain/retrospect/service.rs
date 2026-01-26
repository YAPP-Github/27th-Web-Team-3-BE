use super::dto::{
    DeleteRetroRoomResponse, JoinRetroRoomRequest, JoinRetroRoomResponse, RetroRoomCreateRequest,
    RetroRoomCreateResponse, RetroRoomListItem, RetrospectListItem, UpdateRetroRoomNameRequest,
    UpdateRetroRoomNameResponse, UpdateRetroRoomOrderRequest,
};
use crate::domain::member::entity::member_retro_room::{self, Entity as MemberRetroRoom, RoomRole};
use crate::domain::retrospect::entity::retro_room::{self, Entity as RetroRoom};
use crate::domain::retrospect::entity::retrospect::{self, Entity as Retrospect};
use crate::state::AppState;
use crate::utils::error::AppError;
use chrono::Utc;
use sea_orm::*;
use std::collections::HashSet;

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

    /// API-006: 사용자가 참여 중인 레트로룸 목록 조회
    pub async fn list_retro_rooms(
        state: AppState,
        member_id: i64,
    ) -> Result<Vec<RetroRoomListItem>, AppError> {
        // member_retro_room에서 사용자가 참여 중인 룸 목록 조회
        let member_rooms = MemberRetroRoom::find()
            .filter(member_retro_room::Column::MemberId.eq(member_id))
            .order_by_asc(member_retro_room::Column::OrderIndex)
            .all(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("DB Error: {}", e)))?;

        let mut result = Vec::new();
        for member_room in member_rooms {
            let room = RetroRoom::find_by_id(member_room.retrospect_room_id)
                .one(&state.db)
                .await
                .map_err(|e| AppError::InternalError(format!("DB Error: {}", e)))?;

            if let Some(room) = room {
                result.push(RetroRoomListItem {
                    retro_room_id: room.retrospect_room_id,
                    retro_room_name: room.title,
                    order_index: member_room.order_index,
                });
            }
        }

        Ok(result)
    }

    /// API-007: 레트로룸 순서 변경
    pub async fn update_retro_room_order(
        state: AppState,
        member_id: i64,
        req: UpdateRetroRoomOrderRequest,
    ) -> Result<(), AppError> {
        // 1. orderIndex 중복 체크
        let order_indices: Vec<i32> = req
            .retro_room_orders
            .iter()
            .map(|o| o.order_index)
            .collect();
        let unique_indices: HashSet<i32> = order_indices.iter().cloned().collect();
        if order_indices.len() != unique_indices.len() {
            return Err(AppError::InvalidOrderData(
                "잘못된 순서 데이터입니다.".into(),
            ));
        }

        // 2. 각 룸에 대해 권한 체크 및 업데이트
        for order_item in &req.retro_room_orders {
            // 사용자가 해당 룸의 멤버인지 확인
            let member_room = MemberRetroRoom::find()
                .filter(member_retro_room::Column::MemberId.eq(member_id))
                .filter(member_retro_room::Column::RetrospectRoomId.eq(order_item.retro_room_id))
                .one(&state.db)
                .await
                .map_err(|e| AppError::InternalError(format!("DB Error: {}", e)))?;

            let member_room = member_room
                .ok_or_else(|| AppError::NoPermission("순서를 변경할 권한이 없습니다.".into()))?;

            // 순서 업데이트
            let mut active_model: member_retro_room::ActiveModel = member_room.into();
            active_model.order_index = Set(order_item.order_index);
            active_model
                .update(&state.db)
                .await
                .map_err(|e| AppError::InternalError(format!("순서 업데이트 실패: {}", e)))?;
        }

        Ok(())
    }

    /// API-008: 레트로룸 이름 변경
    pub async fn update_retro_room_name(
        state: AppState,
        member_id: i64,
        retro_room_id: i64,
        req: UpdateRetroRoomNameRequest,
    ) -> Result<UpdateRetroRoomNameResponse, AppError> {
        // 1. 룸 존재 확인
        let room = RetroRoom::find_by_id(retro_room_id)
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("DB Error: {}", e)))?;

        let room =
            room.ok_or_else(|| AppError::NotFound("존재하지 않는 레트로룸입니다.".into()))?;

        // 2. Owner 권한 확인
        let member_room = MemberRetroRoom::find()
            .filter(member_retro_room::Column::MemberId.eq(member_id))
            .filter(member_retro_room::Column::RetrospectRoomId.eq(retro_room_id))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("DB Error: {}", e)))?;

        let member_room = member_room.ok_or_else(|| {
            AppError::NoRoomPermission("레트로룸 이름을 변경할 권한이 없습니다.".into())
        })?;

        if member_room.role != RoomRole::Owner {
            return Err(AppError::NoRoomPermission(
                "레트로룸 이름을 변경할 권한이 없습니다.".into(),
            ));
        }

        // 3. 이름 중복 체크
        let existing = RetroRoom::find()
            .filter(retro_room::Column::Title.eq(&req.name))
            .filter(retro_room::Column::RetrospectRoomId.ne(retro_room_id))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("DB Error: {}", e)))?;

        if existing.is_some() {
            return Err(AppError::RetroRoomNameDuplicate(
                "이미 사용 중인 레트로룸 이름입니다.".into(),
            ));
        }

        // 4. 이름 업데이트
        let now = Utc::now().naive_utc();
        let mut active_model: retro_room::ActiveModel = room.into();
        active_model.title = Set(req.name.clone());
        active_model.updated_at = Set(now);
        active_model
            .update(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("이름 변경 실패: {}", e)))?;

        Ok(UpdateRetroRoomNameResponse {
            retro_room_id,
            retro_room_name: req.name,
            updated_at: Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
        })
    }

    /// API-009: 레트로룸 삭제
    pub async fn delete_retro_room(
        state: AppState,
        member_id: i64,
        retro_room_id: i64,
    ) -> Result<DeleteRetroRoomResponse, AppError> {
        // 1. 룸 존재 확인
        let room = RetroRoom::find_by_id(retro_room_id)
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("DB Error: {}", e)))?;

        let room =
            room.ok_or_else(|| AppError::NotFound("존재하지 않는 레트로룸입니다.".into()))?;

        // 2. Owner 권한 확인
        let member_room = MemberRetroRoom::find()
            .filter(member_retro_room::Column::MemberId.eq(member_id))
            .filter(member_retro_room::Column::RetrospectRoomId.eq(retro_room_id))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("DB Error: {}", e)))?;

        let member_room = member_room
            .ok_or_else(|| AppError::NoPermission("레트로룸을 삭제할 권한이 없습니다.".into()))?;

        if member_room.role != RoomRole::Owner {
            return Err(AppError::NoPermission(
                "레트로룸을 삭제할 권한이 없습니다.".into(),
            ));
        }

        // 3. 관련 데이터 삭제 (member_retro_room 먼저)
        MemberRetroRoom::delete_many()
            .filter(member_retro_room::Column::RetrospectRoomId.eq(retro_room_id))
            .exec(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("멤버 삭제 실패: {}", e)))?;

        // 4. 레트로룸 삭제
        let active_model: retro_room::ActiveModel = room.into();
        active_model
            .delete(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("레트로룸 삭제 실패: {}", e)))?;

        Ok(DeleteRetroRoomResponse {
            retro_room_id,
            deleted_at: Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
        })
    }

    /// API-010: 레트로룸 내 회고 목록 조회
    pub async fn list_retrospects(
        state: AppState,
        member_id: i64,
        retro_room_id: i64,
    ) -> Result<Vec<RetrospectListItem>, AppError> {
        // 1. 룸 존재 확인
        let room = RetroRoom::find_by_id(retro_room_id)
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("DB Error: {}", e)))?;

        if room.is_none() {
            return Err(AppError::NotFound("존재하지 않는 레트로룸입니다.".into()));
        }

        // 2. 멤버 권한 확인
        let member_room = MemberRetroRoom::find()
            .filter(member_retro_room::Column::MemberId.eq(member_id))
            .filter(member_retro_room::Column::RetrospectRoomId.eq(retro_room_id))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("DB Error: {}", e)))?;

        if member_room.is_none() {
            return Err(AppError::NoPermission(
                "해당 레트로룸에 접근 권한이 없습니다.".into(),
            ));
        }

        // 3. 회고 목록 조회 (최신순 정렬)
        let retrospects = Retrospect::find()
            .filter(retrospect::Column::RetrospectRoomId.eq(retro_room_id))
            .order_by_desc(retrospect::Column::StartTime)
            .all(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("DB Error: {}", e)))?;

        let result = retrospects
            .into_iter()
            .map(|r| RetrospectListItem {
                retrospect_id: r.retrospect_id,
                project_name: r.title,
                retrospect_method: format!("{:?}", r.retro_category).to_uppercase(),
                retrospect_date: r.start_time.format("%Y-%m-%d").to_string(),
                retrospect_time: r.start_time.format("%H:%M").to_string(),
            })
            .collect();

        Ok(result)
    }

    /// 초대 코드 생성 (형식: INV-XXXX-XXXX)
    pub fn generate_invite_code() -> String {
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
    pub fn extract_invite_code(invite_url: &str) -> Result<String, AppError> {
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
