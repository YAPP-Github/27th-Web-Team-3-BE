use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// OG 메타데이터 조회 쿼리 파라미터
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OgMetadataQuery {
    #[validate(length(
        min = 1,
        max = 2048,
        message = "URL은 1자 이상 2,048자 이하여야 합니다"
    ))]
    pub url: String,
}

/// OG 메타데이터 응답
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OgMetadataResponse {
    /// 요청한 원본 URL
    pub url: String,
    /// og:title 값
    pub title: Option<String>,
    /// og:description 값
    pub description: Option<String>,
    /// og:image 값
    pub image: Option<String>,
}

/// Swagger용 성공 응답
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessOgMetadataResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: OgMetadataResponse,
}
