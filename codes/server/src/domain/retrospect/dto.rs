use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetroRoomCreateRequest {
    #[validate(length(min = 1, max = 20, message = "회고 룸 이름은 1~20자여야 합니다."))]
    pub title: String,

    #[validate(length(max = 50, message = "회고 룸 한 줄 소개는 50자를 초과할 수 없습니다."))]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetroRoomCreateResponse {
    pub retro_room_id: i64,
    pub title: String,
    pub invite_code: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessRetroRoomCreateResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: RetroRoomCreateResponse,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JoinRetroRoomRequest {
    #[validate(url(message = "유효한 URL 형식이 아닙니다."))]
    pub invite_url: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JoinRetroRoomResponse {
    pub retro_room_id: i64,
    pub title: String,
    pub joined_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessJoinRetroRoomResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: JoinRetroRoomResponse,
}

// ============== API-006: 레트로룸 목록 조회 ==============

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetroRoomListItem {
    pub retro_room_id: i64,
    pub retro_room_name: String,
    pub order_index: i32,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessRetroRoomListResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: Vec<RetroRoomListItem>,
}

// ============== API-007: 레트로룸 순서 변경 ==============

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetroRoomOrderItem {
    pub retro_room_id: i64,
    #[validate(range(min = 1, message = "orderIndex는 1 이상이어야 합니다."))]
    pub order_index: i32,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRetroRoomOrderRequest {
    #[validate(length(min = 1, message = "최소 1개 이상의 순서 정보가 필요합니다."))]
    #[validate(nested)]
    pub retro_room_orders: Vec<RetroRoomOrderItem>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessEmptyResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: Option<()>,
}

// ============== API-008: 레트로룸 이름 변경 ==============

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRetroRoomNameRequest {
    #[validate(length(min = 1, max = 20, message = "팀 이름은 1~20자여야 합니다."))]
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRetroRoomNameResponse {
    pub retro_room_id: i64,
    pub retro_room_name: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessUpdateRetroRoomNameResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: UpdateRetroRoomNameResponse,
}

// ============== API-009: 레트로룸 삭제 ==============

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeleteRetroRoomResponse {
    pub retro_room_id: i64,
    pub deleted_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessDeleteRetroRoomResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: DeleteRetroRoomResponse,
}

// ============== API-010: 레트로룸 내 회고 목록 조회 ==============

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetrospectListItem {
    pub retrospect_id: i64,
    pub project_name: String,
    pub retrospect_method: String,
    pub retrospect_date: String,
    pub retrospect_time: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessRetrospectListResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: Vec<RetrospectListItem>,
}
