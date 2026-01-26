use serde::{Deserialize, Serialize};
use validator::Validate;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TeamCreateRequest {
    #[validate(length(max = 20, message = "팀 이름은 20자를 초과할 수 없습니다."))]
    pub team_name: String,
    
    #[validate(length(max = 50, message = "팀 한 줄 소개는 50자를 초과할 수 없습니다."))]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TeamCreateResponse {
    pub team_id: i64,
    pub team_name: String,
    pub invite_code: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessTeamCreateResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: TeamCreateResponse,
}
