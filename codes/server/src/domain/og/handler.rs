use axum::extract::Query;
use axum::Json;
use validator::Validate;

use crate::domain::og::dto::{OgMetadataQuery, OgMetadataResponse};
use crate::domain::og::service::OgService;
use crate::utils::auth::AuthUser;
use crate::utils::error::AppError;
use crate::utils::response::BaseResponse;

/// Open Graph 메타데이터 조회
#[utoipa::path(
    get,
    path = "/api/v1/og",
    params(
        ("url" = String, Query, description = "OG 태그를 파싱할 대상 URL")
    ),
    responses(
        (status = 200, description = "OG 메타데이터 조회 성공", body = SuccessOgMetadataResponse),
        (status = 400, description = "잘못된 요청", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "OG"
)]
pub async fn get_og_metadata(
    _user: AuthUser,
    Query(query): Query<OgMetadataQuery>,
) -> Result<Json<BaseResponse<OgMetadataResponse>>, AppError> {
    query.validate()?;
    let result = OgService::fetch_metadata(&query.url).await?;
    Ok(Json(BaseResponse::success(result)))
}
