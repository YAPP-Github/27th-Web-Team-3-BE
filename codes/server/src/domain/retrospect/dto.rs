use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::member::entity::member_retro::RetrospectStatus;

// ============================================
// API-017: 회고 최종 제출 DTO
// ============================================

/// 회고 제출 요청 DTO
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubmitRetrospectRequest {
    /// 제출할 답변 리스트 (정확히 5개, 서비스 레이어에서 검증)
    pub answers: Vec<SubmitAnswerItem>,
}

/// 제출 답변 아이템
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubmitAnswerItem {
    /// 질문 번호 (1~5)
    pub question_number: i32,
    /// 답변 내용 (1~1,000자)
    pub content: String,
}

/// 회고 제출 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubmitRetrospectResponse {
    /// 제출된 회고의 고유 ID
    pub retrospect_id: i64,
    /// 최종 제출 날짜 (YYYY-MM-DD)
    pub submitted_at: String,
    /// 현재 회고 상태
    pub status: RetrospectStatus,
}

/// Swagger용 회고 제출 성공 응답 타입
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessSubmitRetrospectResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: SubmitRetrospectResponse,
}
