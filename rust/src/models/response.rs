use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BaseResponse<T> {
    #[serde(rename = "isSuccess")]
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: T,
}

impl<T> BaseResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            is_success: true,
            code: "COMMON200".to_string(),
            message: "성공입니다.".to_string(),
            result: data,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GuideResponse {
    #[schema(example = "오늘 프로젝트를 진행하면서...")]
    #[serde(rename = "currentContent")]
    pub current_content: String,

    #[schema(example = "좋은 시작이에요! 구체적으로 어떤 점이 어려웠는지 작성해보면 어떨까요?")]
    #[serde(rename = "guideMessage")]
    pub guide_message: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RefineResponse {
    #[schema(example = "오늘 일 존나 힘들었음 ㅋㅋ 근데 배운게 많았어")]
    #[serde(rename = "originalContent")]
    pub original_content: String,

    #[schema(example = "오늘 업무가 힘들었지만, 그만큼 많은 것을 배울 수 있었어요.")]
    #[serde(rename = "refinedContent")]
    pub refined_content: String,

    #[schema(example = "KIND")]
    #[serde(rename = "toneStyle")]
    pub tone_style: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SignUpResponse {
    #[serde(rename = "userId")]
    pub user_id: u64,

    #[schema(example = "user@example.com")]
    pub email: String,

    #[schema(example = "홍길동")]
    pub username: String,

    #[schema(example = "2024-01-08T12:34:56Z")]
    #[serde(rename = "createdAt")]
    pub created_at: String,
}
