use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use super::entity::retrospect::RetrospectMethod;

/// 회고 생성 요청 DTO
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateRetrospectRequest {
    /// 회고가 속한 팀의 고유 ID
    #[validate(range(min = 1, message = "팀 ID는 1 이상이어야 합니다"))]
    pub team_id: i64,

    /// 프로젝트 이름 (최소 1자, 최대 20자)
    #[validate(length(
        min = 1,
        max = 20,
        message = "프로젝트 이름은 1자 이상 20자 이하여야 합니다"
    ))]
    pub project_name: String,

    /// 회고 날짜 (ISO 8601 형식: YYYY-MM-DD)
    #[validate(length(
        min = 10,
        max = 10,
        message = "날짜 형식이 올바르지 않습니다. (YYYY-MM-DD 형식 필요)"
    ))]
    pub retrospect_date: String,

    /// 회고 방식
    pub retrospect_method: RetrospectMethod,

    /// 참고 자료 URL 리스트 (최대 10개)
    #[validate(length(max = 10, message = "참고 URL은 최대 10개까지 등록 가능합니다"))]
    #[serde(default)]
    pub reference_urls: Vec<String>,
}

/// 회고 생성 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateRetrospectResponse {
    /// 생성된 회고 고유 ID
    pub retrospect_id: i64,
    /// 회고가 속한 팀의 고유 ID
    pub team_id: i64,
    /// 저장된 프로젝트 이름
    pub project_name: String,
}

/// Swagger용 성공 응답 타입
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessCreateRetrospectResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: CreateRetrospectResponse,
}
