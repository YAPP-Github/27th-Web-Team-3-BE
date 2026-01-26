use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetroRoomCreateRequest {
    #[validate(length(max = 20, message = "회고 룸 이름은 20자를 초과할 수 없습니다."))]
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
