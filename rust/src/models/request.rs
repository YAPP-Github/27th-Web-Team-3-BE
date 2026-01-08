use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct GuideRequest {
    #[validate(length(min = 1, message = "현재 작성 중인 내용은 필수입니다."))]
    #[schema(example = "오늘 프로젝트를 진행하면서...")]
    #[serde(rename = "currentContent")]
    pub current_content: String,

    #[validate(length(min = 1, message = "비밀 키는 필수입니다."))]
    #[schema(example = "mySecretKey123")]
    #[serde(rename = "secretKey")]
    pub secret_key: String,
}

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct RetrospectiveGuideRequest {
    #[validate(length(min = 1, message = "회고 내용은 필수입니다."))]
    #[schema(example = "오늘 프로젝트를 진행하면서 많은 것을 배웠다.")]
    pub content: String,

    #[validate(length(min = 1, message = "비밀 키는 필수입니다."))]
    #[schema(example = "mySecretKey123")]
    #[serde(rename = "secretKey")]
    pub secret_key: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ToneStyle {
    Kind,   // 상냥체
    Polite, // 정중체
}

impl ToneStyle {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_uppercase().as_str() {
            "KIND" => Ok(ToneStyle::Kind),
            "POLITE" => Ok(ToneStyle::Polite),
            _ => Err(s.to_string()),
        }
    }

    pub fn to_korean(&self) -> &str {
        match self {
            ToneStyle::Kind => "상냥체",
            ToneStyle::Polite => "정중체",
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct RefineRequest {
    #[validate(length(min = 1, message = "회고 내용은 필수입니다."))]
    #[schema(example = "오늘 일 존나 힘들었음 ㅋㅋ 근데 배운게 많았어")]
    pub content: String,

    #[schema(example = "KIND")]
    #[serde(rename = "toneStyle")]
    pub tone_style: ToneStyle,

    #[validate(length(min = 1, message = "비밀 키는 필수입니다."))]
    #[schema(example = "mySecretKey123")]
    #[serde(rename = "secretKey")]
    pub secret_key: String,
}

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct SignUpRequest {
    #[validate(email(message = "올바른 이메일 형식이 아닙니다."))]
    #[schema(example = "user@example.com")]
    pub email: String,

    #[validate(length(min = 2, max = 50, message = "사용자 이름은 2-50자 사이여야 합니다."))]
    #[schema(example = "홍길동")]
    pub username: String,

    #[validate(length(min = 8, message = "비밀번호는 최소 8자 이상이어야 합니다."))]
    #[schema(example = "password123!")]
    pub password: String,

    #[validate(must_match(other = "password", message = "비밀번호가 일치하지 않습니다."))]
    #[schema(example = "password123!")]
    #[serde(rename = "passwordConfirm")]
    pub password_confirm: String,
}
