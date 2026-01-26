use std::collections::{BTreeMap, HashSet};

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
    TransactionTrait,
};
use tracing::info;

use crate::domain::member::entity::member_response;
use crate::domain::member::entity::member_retro;
use crate::domain::member::entity::member_retro::RetrospectStatus;
use crate::domain::retrospect::entity::response;
use crate::domain::retrospect::entity::retrospect;
use crate::state::AppState;
use crate::utils::error::AppError;

use super::dto::{
    StorageQueryParams, StorageResponse, StorageRetrospectItem, StorageYearGroup, SubmitAnswerItem,
    SubmitRetrospectRequest, SubmitRetrospectResponse,
};

pub struct RetrospectService;

impl RetrospectService {
    /// 회고 최종 제출 (API-017)
    pub async fn submit_retrospect(
        state: AppState,
        user_id: i64,
        retrospect_id: i64,
        req: SubmitRetrospectRequest,
    ) -> Result<SubmitRetrospectResponse, AppError> {
        // 1. 답변 비즈니스 검증 (트랜잭션 전에 수행)
        Self::validate_answers(&req.answers)?;

        // 2. 회고 존재 여부 확인
        let _retrospect_model = retrospect::Entity::find_by_id(retrospect_id)
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .ok_or_else(|| AppError::RetrospectNotFound("존재하지 않는 회고입니다.".to_string()))?;

        // 3. 트랜잭션 시작 (동시 제출 경쟁 조건 방지)
        let txn = state
            .db
            .begin()
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // 4. 참석자(member_retro) 확인 - 행 잠금으로 동시 제출 방지
        let member_retro_model = member_retro::Entity::find()
            .filter(member_retro::Column::MemberId.eq(user_id))
            .filter(member_retro::Column::RetrospectId.eq(retrospect_id))
            .lock_exclusive()
            .one(&txn)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .ok_or_else(|| {
                AppError::RetrospectNotFound(
                    "존재하지 않는 회고이거나 접근 권한이 없습니다.".to_string(),
                )
            })?;

        // 5. 이미 제출 완료 여부 확인 (행 잠금 후 검사로 경쟁 조건 방지)
        if member_retro_model.status == RetrospectStatus::Submitted
            || member_retro_model.status == RetrospectStatus::Analyzed
        {
            return Err(AppError::RetroAlreadySubmitted(
                "이미 제출이 완료된 회고입니다.".to_string(),
            ));
        }

        // 6. member_response를 통해 해당 멤버의 응답(response) ID 조회
        let member_response_ids: Vec<i64> = member_response::Entity::find()
            .filter(member_response::Column::MemberId.eq(user_id))
            .all(&txn)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .iter()
            .map(|mr| mr.response_id)
            .collect();

        // 7. 해당 멤버의 질문(response) 목록 조회 (response_id 오름차순)
        let responses = response::Entity::find()
            .filter(response::Column::RetrospectId.eq(retrospect_id))
            .filter(response::Column::ResponseId.is_in(member_response_ids))
            .order_by_asc(response::Column::ResponseId)
            .all(&txn)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        if responses.len() != 5 {
            return Err(AppError::InternalError(
                "회고의 질문 수가 올바르지 않습니다.".to_string(),
            ));
        }

        // 8. 답변 업데이트 (questionNumber 순서에 맞게)
        let now = Utc::now().naive_utc();
        for answer in &req.answers {
            let idx = (answer.question_number - 1) as usize;
            let response_model = &responses[idx];

            let mut active: response::ActiveModel = response_model.clone().into();
            active.content = Set(answer.content.clone());
            active.updated_at = Set(now);
            active
                .update(&txn)
                .await
                .map_err(|e| AppError::InternalError(e.to_string()))?;
        }

        // 9. member_retro 상태를 SUBMITTED으로 업데이트 (UTC로 저장)
        let mut member_retro_active: member_retro::ActiveModel = member_retro_model.clone().into();
        member_retro_active.status = Set(RetrospectStatus::Submitted);
        member_retro_active.submitted_at = Set(Some(now));
        member_retro_active
            .update(&txn)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // 10. 트랜잭션 커밋
        txn.commit()
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // 응답 생성 (KST 변환은 응답에서만 수행)
        let kst_display = (now + chrono::Duration::hours(9))
            .format("%Y-%m-%d")
            .to_string();

        Ok(SubmitRetrospectResponse {
            retrospect_id,
            submitted_at: kst_display,
            status: RetrospectStatus::Submitted,
        })
    }

    /// 보관함 조회 (API-019)
    pub async fn get_storage(
        state: AppState,
        user_id: i64,
        params: StorageQueryParams,
    ) -> Result<StorageResponse, AppError> {
        let range_filter = params.range.unwrap_or_default();

        info!(
            user_id = user_id,
            range = %range_filter,
            "보관함 조회 요청"
        );

        // 1. 사용자가 참여한 회고 중 제출 완료/분석 완료 상태만 조회
        let mut member_retro_query = member_retro::Entity::find()
            .filter(member_retro::Column::MemberId.eq(user_id))
            .filter(
                member_retro::Column::Status
                    .is_in([RetrospectStatus::Submitted, RetrospectStatus::Analyzed]),
            );

        // 2. 기간 필터 적용
        if let Some(days) = range_filter.days() {
            let cutoff = Utc::now().naive_utc() - chrono::Duration::days(days);
            member_retro_query =
                member_retro_query.filter(member_retro::Column::SubmittedAt.gte(cutoff));
        }

        let member_retros = member_retro_query
            .all(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        if member_retros.is_empty() {
            return Ok(StorageResponse { years: vec![] });
        }

        // 3. 관련 회고 ID 추출
        let retrospect_ids: Vec<i64> = member_retros.iter().map(|mr| mr.retrospect_id).collect();

        // 4. 회고 정보 조회
        let retrospects = retrospect::Entity::find()
            .filter(retrospect::Column::RetrospectId.is_in(retrospect_ids.clone()))
            .all(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // 5. 각 회고의 참여자 수 조회
        let mut member_counts: std::collections::HashMap<i64, i64> =
            std::collections::HashMap::new();
        for retro_id in &retrospect_ids {
            let count = member_retro::Entity::find()
                .filter(member_retro::Column::RetrospectId.eq(*retro_id))
                .all(&state.db)
                .await
                .map_err(|e| AppError::InternalError(e.to_string()))?
                .len() as i64;
            member_counts.insert(*retro_id, count);
        }

        // 6. 연도별 그룹핑 (BTreeMap으로 정렬)
        let mut year_groups: BTreeMap<i32, Vec<StorageRetrospectItem>> = BTreeMap::new();

        // member_retro에서 submitted_at 기준으로 날짜 매핑
        let submitted_dates: std::collections::HashMap<i64, chrono::NaiveDateTime> = member_retros
            .iter()
            .filter_map(|mr| mr.submitted_at.map(|dt| (mr.retrospect_id, dt)))
            .collect();

        for retro in &retrospects {
            // UTC → KST 변환은 표시용에서만 수행
            let kst_offset = chrono::Duration::hours(9);

            let display_date = submitted_dates
                .get(&retro.retrospect_id)
                .map(|dt| (*dt + kst_offset).format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| {
                    (retro.created_at + kst_offset)
                        .format("%Y-%m-%d")
                        .to_string()
                });

            let year = submitted_dates
                .get(&retro.retrospect_id)
                .map(|dt| (*dt + kst_offset).format("%Y").to_string())
                .unwrap_or_else(|| (retro.created_at + kst_offset).format("%Y").to_string())
                .parse::<i32>()
                .unwrap_or(0);

            let item = StorageRetrospectItem {
                retrospect_id: retro.retrospect_id,
                display_date,
                title: retro.title.clone(),
                retro_category: retro.retro_category.clone(),
                member_count: *member_counts.get(&retro.retrospect_id).unwrap_or(&0),
            };

            year_groups.entry(year).or_default().push(item);
        }

        // 7. 연도별 내림차순 정렬 + 각 그룹 내 최신순 정렬
        let mut years: Vec<StorageYearGroup> = year_groups
            .into_iter()
            .rev()
            .map(|(year, mut items)| {
                items.sort_by(|a, b| b.display_date.cmp(&a.display_date));
                StorageYearGroup {
                    year_label: format!("{}년", year),
                    retrospects: items,
                }
            })
            .collect();

        // BTreeMap의 rev()는 이미 내림차순이므로 추가 정렬 불필요
        // 하지만 안전을 위해 정렬 보장
        years.sort_by(|a, b| b.year_label.cmp(&a.year_label));

        Ok(StorageResponse { years })
    }

    /// 답변 비즈니스 검증
    fn validate_answers(answers: &[SubmitAnswerItem]) -> Result<(), AppError> {
        // 1. 정확히 5개 답변 확인
        if answers.len() != 5 {
            return Err(AppError::RetroAnswersMissing(
                "모든 질문에 대한 답변이 필요합니다.".to_string(),
            ));
        }

        // 2. questionNumber 1~5 모두 존재하는지 확인
        let question_numbers: HashSet<i32> = answers.iter().map(|a| a.question_number).collect();
        let expected: HashSet<i32> = (1..=5).collect();
        if question_numbers != expected {
            return Err(AppError::RetroAnswersMissing(
                "모든 질문에 대한 답변이 필요합니다.".to_string(),
            ));
        }

        // 3. 각 답변 내용 검증
        for answer in answers {
            // 공백만으로 구성된 답변 체크
            if answer.content.trim().is_empty() {
                return Err(AppError::RetroAnswerWhitespaceOnly(
                    "답변 내용은 공백만으로 구성될 수 없습니다.".to_string(),
                ));
            }

            // 최대 1,000자 제한
            if answer.content.chars().count() > 1000 {
                return Err(AppError::RetroAnswerTooLong(
                    "답변은 1,000자를 초과할 수 없습니다.".to_string(),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== 답변 검증 테스트 =====

    fn create_valid_answers() -> Vec<SubmitAnswerItem> {
        (1..=5)
            .map(|i| SubmitAnswerItem {
                question_number: i,
                content: format!("질문 {}에 대한 답변입니다.", i),
            })
            .collect()
    }

    #[test]
    fn should_pass_valid_answers() {
        // Arrange
        let answers = create_valid_answers();

        // Act
        let result = RetrospectService::validate_answers(&answers);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_fail_when_answers_count_is_not_5() {
        // Arrange
        let answers: Vec<SubmitAnswerItem> = (1..=4)
            .map(|i| SubmitAnswerItem {
                question_number: i,
                content: format!("답변 {}", i),
            })
            .collect();

        // Act
        let result = RetrospectService::validate_answers(&answers);

        // Assert
        assert!(result.is_err());
        if let Err(AppError::RetroAnswersMissing(msg)) = result {
            assert!(msg.contains("모든 질문"));
        } else {
            panic!("Expected RetroAnswersMissing error");
        }
    }

    #[test]
    fn should_fail_when_question_number_missing() {
        // Arrange - questionNumber 3 대신 6을 사용
        let mut answers = create_valid_answers();
        answers[2].question_number = 6;

        // Act
        let result = RetrospectService::validate_answers(&answers);

        // Assert
        assert!(result.is_err());
        assert!(matches!(result, Err(AppError::RetroAnswersMissing(_))));
    }

    #[test]
    fn should_fail_when_duplicate_question_numbers() {
        // Arrange - questionNumber 1이 두 개
        let mut answers = create_valid_answers();
        answers[4].question_number = 1; // 5번 대신 1번 중복

        // Act
        let result = RetrospectService::validate_answers(&answers);

        // Assert
        assert!(result.is_err());
        assert!(matches!(result, Err(AppError::RetroAnswersMissing(_))));
    }

    #[test]
    fn should_fail_when_content_is_whitespace_only() {
        // Arrange
        let mut answers = create_valid_answers();
        answers[0].content = "   \t\n  ".to_string();

        // Act
        let result = RetrospectService::validate_answers(&answers);

        // Assert
        assert!(result.is_err());
        if let Err(AppError::RetroAnswerWhitespaceOnly(msg)) = result {
            assert!(msg.contains("공백만"));
        } else {
            panic!("Expected RetroAnswerWhitespaceOnly error");
        }
    }

    #[test]
    fn should_fail_when_content_is_empty() {
        // Arrange
        let mut answers = create_valid_answers();
        answers[0].content = String::new();

        // Act
        let result = RetrospectService::validate_answers(&answers);

        // Assert
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(AppError::RetroAnswerWhitespaceOnly(_))
        ));
    }

    #[test]
    fn should_fail_when_content_exceeds_1000_chars() {
        // Arrange
        let mut answers = create_valid_answers();
        answers[0].content = "가".repeat(1001);

        // Act
        let result = RetrospectService::validate_answers(&answers);

        // Assert
        assert!(result.is_err());
        if let Err(AppError::RetroAnswerTooLong(msg)) = result {
            assert!(msg.contains("1,000자"));
        } else {
            panic!("Expected RetroAnswerTooLong error");
        }
    }

    #[test]
    fn should_pass_when_content_is_exactly_1000_chars() {
        // Arrange
        let mut answers = create_valid_answers();
        answers[0].content = "가".repeat(1000);

        // Act
        let result = RetrospectService::validate_answers(&answers);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_pass_when_content_has_leading_trailing_whitespace() {
        // Arrange - 앞뒤 공백이 있지만 실제 내용이 있는 경우
        let mut answers = create_valid_answers();
        answers[0].content = "  유효한 답변  ".to_string();

        // Act
        let result = RetrospectService::validate_answers(&answers);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_fail_when_answers_is_empty() {
        // Arrange
        let answers: Vec<SubmitAnswerItem> = vec![];

        // Act
        let result = RetrospectService::validate_answers(&answers);

        // Assert
        assert!(result.is_err());
        assert!(matches!(result, Err(AppError::RetroAnswersMissing(_))));
    }
}
