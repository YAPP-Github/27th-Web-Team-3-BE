use std::collections::HashSet;

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set, TransactionTrait,
};
use tracing::info;

use crate::domain::member::entity::member_response;
use crate::domain::member::entity::member_retro;
use crate::domain::retrospect::entity::response;
use crate::domain::retrospect::entity::retrospect;
use crate::state::AppState;
use crate::utils::error::AppError;

use super::dto::{DraftItem, DraftSaveRequest, DraftSaveResponse};

/// 회고당 질문 수 (고정)
const QUESTIONS_PER_RETROSPECT: usize = 5;

pub struct RetrospectService;

impl RetrospectService {
    /// 회고 답변 임시 저장 (API-016)
    pub async fn save_draft(
        state: AppState,
        user_id: i64,
        retrospect_id: i64,
        req: DraftSaveRequest,
    ) -> Result<DraftSaveResponse, AppError> {
        // 1. 답변 비즈니스 검증 (트랜잭션 전에 수행)
        Self::validate_drafts(&req.drafts)?;

        info!(
            user_id = user_id,
            retrospect_id = retrospect_id,
            draft_count = req.drafts.len(),
            "회고 답변 임시 저장 요청"
        );

        // 2. 회고 존재 여부 확인
        let _retrospect_model = retrospect::Entity::find_by_id(retrospect_id)
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .ok_or_else(|| AppError::RetrospectNotFound("존재하지 않는 회고입니다.".to_string()))?;

        // 3. 참석자(member_retro) 확인 - 해당 회고에 대한 작성 권한 검증
        let _member_retro_model = member_retro::Entity::find()
            .filter(member_retro::Column::MemberId.eq(user_id))
            .filter(member_retro::Column::RetrospectId.eq(retrospect_id))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .ok_or_else(|| {
                AppError::TeamAccessDenied("해당 회고에 작성 권한이 없습니다.".to_string())
            })?;

        // 4. member_response를 통해 해당 멤버의 응답(response) ID 조회
        let member_response_ids: Vec<i64> = member_response::Entity::find()
            .filter(member_response::Column::MemberId.eq(user_id))
            .all(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .iter()
            .map(|mr| mr.response_id)
            .collect();

        // 4-1. 응답이 없는 경우 사전 방어 (member_response가 없으면 권한 문제)
        if member_response_ids.is_empty() {
            return Err(AppError::TeamAccessDenied(
                "해당 회고에 대한 응답 데이터가 존재하지 않습니다.".to_string(),
            ));
        }

        // 5. 해당 멤버의 질문(response) 목록 조회 (response_id 오름차순)
        let responses = response::Entity::find()
            .filter(response::Column::RetrospectId.eq(retrospect_id))
            .filter(response::Column::ResponseId.is_in(member_response_ids))
            .order_by_asc(response::Column::ResponseId)
            .all(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // 5-1. 질문 수 불일치 검증 (response_id 순서 매핑이 안전한지 확인)
        if responses.len() != QUESTIONS_PER_RETROSPECT {
            return Err(AppError::InternalError(format!(
                "질문-응답 매핑 불일치: 예상 {}개, 실제 {}개",
                QUESTIONS_PER_RETROSPECT,
                responses.len()
            )));
        }

        // 6. 답변 업데이트 (트랜잭션으로 원자적 처리)
        let now = Utc::now().naive_utc();
        let txn = state
            .db
            .begin()
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        for draft in &req.drafts {
            let idx = (draft.question_number - 1) as usize;
            // validate_drafts에서 1~5 범위를 이미 검증했으므로 idx는 0~4 이내
            let response_model = &responses[idx];

            let mut active: response::ActiveModel = response_model.clone().into();
            // content가 None이면 빈 문자열로 저장 (기존 내용 삭제)
            active.content = Set(draft.content.clone().unwrap_or_default());
            active.updated_at = Set(now);
            active
                .update(&txn)
                .await
                .map_err(|e| AppError::InternalError(e.to_string()))?;
        }

        txn.commit()
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // 7. 응답 생성 (KST 변환은 응답에서만 수행)
        let kst_display = (now + chrono::Duration::hours(9))
            .format("%Y-%m-%d")
            .to_string();

        info!(
            retrospect_id = retrospect_id,
            updated_at = %kst_display,
            "회고 답변 임시 저장 완료"
        );

        Ok(DraftSaveResponse {
            retrospect_id,
            updated_at: kst_display,
        })
    }

    /// 임시 저장 답변 비즈니스 검증
    fn validate_drafts(drafts: &[DraftItem]) -> Result<(), AppError> {
        // 1. 빈 배열 확인 (최소 1개)
        if drafts.is_empty() {
            return Err(AppError::BadRequest(
                "저장할 답변이 최소 1개 이상 필요합니다.".to_string(),
            ));
        }

        // 2. 최대 5개 제한
        if drafts.len() > 5 {
            return Err(AppError::BadRequest(
                "저장할 답변은 최대 5개까지 가능합니다.".to_string(),
            ));
        }

        // 3. 중복 questionNumber 확인
        let mut seen = HashSet::new();
        for draft in drafts {
            if !seen.insert(draft.question_number) {
                return Err(AppError::BadRequest(
                    "중복된 질문 번호가 포함되어 있습니다.".to_string(),
                ));
            }
        }

        // 4. questionNumber 범위 검증 (1~5)
        for draft in drafts {
            if draft.question_number < 1 || draft.question_number > 5 {
                return Err(AppError::BadRequest(
                    "올바르지 않은 질문 번호입니다.".to_string(),
                ));
            }
        }

        // 5. content 길이 검증 (최대 1,000자)
        for draft in drafts {
            if let Some(content) = &draft.content {
                if content.chars().count() > 1000 {
                    return Err(AppError::RetroAnswerTooLong(
                        "답변은 1,000자를 초과할 수 없습니다.".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== 임시 저장 답변 검증 테스트 =====

    fn create_draft(question_number: i32, content: Option<&str>) -> DraftItem {
        DraftItem {
            question_number,
            content: content.map(|c| c.to_string()),
        }
    }

    #[test]
    fn should_pass_valid_single_draft() {
        // Arrange
        let drafts = vec![create_draft(1, Some("첫 번째 답변"))];

        // Act
        let result = RetrospectService::validate_drafts(&drafts);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_pass_valid_multiple_drafts() {
        // Arrange
        let drafts = vec![
            create_draft(1, Some("첫 번째 답변")),
            create_draft(3, Some("세 번째 답변")),
            create_draft(5, Some("다섯 번째 답변")),
        ];

        // Act
        let result = RetrospectService::validate_drafts(&drafts);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_pass_all_five_drafts() {
        // Arrange
        let drafts: Vec<DraftItem> = (1..=5)
            .map(|i| create_draft(i, Some(&format!("답변 {}", i))))
            .collect();

        // Act
        let result = RetrospectService::validate_drafts(&drafts);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_pass_draft_with_null_content() {
        // Arrange
        let drafts = vec![create_draft(2, None)];

        // Act
        let result = RetrospectService::validate_drafts(&drafts);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_pass_draft_with_empty_content() {
        // Arrange
        let drafts = vec![create_draft(1, Some(""))];

        // Act
        let result = RetrospectService::validate_drafts(&drafts);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_pass_draft_with_exactly_1000_chars() {
        // Arrange
        let content = "가".repeat(1000);
        let drafts = vec![create_draft(1, Some(&content))];

        // Act
        let result = RetrospectService::validate_drafts(&drafts);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_fail_when_drafts_is_empty() {
        // Arrange
        let drafts: Vec<DraftItem> = vec![];

        // Act
        let result = RetrospectService::validate_drafts(&drafts);

        // Assert
        assert!(result.is_err());
        if let Err(AppError::BadRequest(msg)) = result {
            assert!(msg.contains("최소 1개"));
        } else {
            panic!("Expected BadRequest error");
        }
    }

    #[test]
    fn should_fail_when_drafts_exceeds_5() {
        // Arrange
        let drafts: Vec<DraftItem> = (1..=6)
            .map(|i| create_draft(i, Some(&format!("답변 {}", i))))
            .collect();

        // Act
        let result = RetrospectService::validate_drafts(&drafts);

        // Assert
        assert!(result.is_err());
        if let Err(AppError::BadRequest(msg)) = result {
            assert!(msg.contains("최대 5개"));
        } else {
            panic!("Expected BadRequest error");
        }
    }

    #[test]
    fn should_fail_when_duplicate_question_numbers() {
        // Arrange
        let drafts = vec![
            create_draft(1, Some("답변 1")),
            create_draft(1, Some("답변 1 중복")),
        ];

        // Act
        let result = RetrospectService::validate_drafts(&drafts);

        // Assert
        assert!(result.is_err());
        if let Err(AppError::BadRequest(msg)) = result {
            assert!(msg.contains("중복된 질문 번호"));
        } else {
            panic!("Expected BadRequest error");
        }
    }

    #[test]
    fn should_fail_when_question_number_is_0() {
        // Arrange
        let drafts = vec![create_draft(0, Some("답변"))];

        // Act
        let result = RetrospectService::validate_drafts(&drafts);

        // Assert
        assert!(result.is_err());
        if let Err(AppError::BadRequest(msg)) = result {
            assert!(msg.contains("올바르지 않은 질문 번호"));
        } else {
            panic!("Expected BadRequest error");
        }
    }

    #[test]
    fn should_fail_when_question_number_is_6() {
        // Arrange
        let drafts = vec![create_draft(6, Some("답변"))];

        // Act
        let result = RetrospectService::validate_drafts(&drafts);

        // Assert
        assert!(result.is_err());
        if let Err(AppError::BadRequest(msg)) = result {
            assert!(msg.contains("올바르지 않은 질문 번호"));
        } else {
            panic!("Expected BadRequest error");
        }
    }

    #[test]
    fn should_fail_when_question_number_is_negative() {
        // Arrange
        let drafts = vec![create_draft(-1, Some("답변"))];

        // Act
        let result = RetrospectService::validate_drafts(&drafts);

        // Assert
        assert!(result.is_err());
        assert!(matches!(result, Err(AppError::BadRequest(_))));
    }

    #[test]
    fn should_fail_when_content_exceeds_1000_chars() {
        // Arrange
        let content = "가".repeat(1001);
        let drafts = vec![create_draft(1, Some(&content))];

        // Act
        let result = RetrospectService::validate_drafts(&drafts);

        // Assert
        assert!(result.is_err());
        if let Err(AppError::RetroAnswerTooLong(msg)) = result {
            assert!(msg.contains("1,000자"));
        } else {
            panic!("Expected RetroAnswerTooLong error");
        }
    }

    #[test]
    fn should_pass_mixed_null_and_content_drafts() {
        // Arrange
        let drafts = vec![
            create_draft(1, Some("답변 있음")),
            create_draft(2, None),
            create_draft(3, Some("")),
        ];

        // Act
        let result = RetrospectService::validate_drafts(&drafts);

        // Assert
        assert!(result.is_ok());
    }
}
