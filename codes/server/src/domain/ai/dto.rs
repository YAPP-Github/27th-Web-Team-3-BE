use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetrospectiveGuideRequest {
    #[validate(length(min = 1, message = "회고 내용은 필수입니다."))]
    pub current_content: String,

    #[validate(length(min = 1, message = "Secret Key는 필수입니다."))]
    pub secret_key: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetrospectiveGuideResult {
    pub current_content: String,
    pub guide_message: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetrospectiveGuideResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: Option<RetrospectiveGuideResult>,
}

impl RetrospectiveGuideResponse {
    pub fn success(data: RetrospectiveGuideResult) -> Self {
        Self {
            is_success: true,
            code: "COMMON200".to_string(),
            message: "성공입니다.".to_string(),
            result: Some(data),
        }
    }
}

