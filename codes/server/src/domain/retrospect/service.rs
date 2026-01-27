use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use genpdf::elements::{Break, Paragraph};
use genpdf::style;
use genpdf::Element;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set, TransactionTrait,
};
use tracing::{info, warn};

use crate::domain::member::entity::member;
use crate::domain::member::entity::member_response;
use crate::domain::member::entity::member_retro;
use crate::domain::member::entity::member_retro::RetrospectStatus;
use crate::domain::member::entity::member_retro_room;
use crate::domain::retrospect::entity::response;
use crate::domain::retrospect::entity::response_comment;
use crate::domain::retrospect::entity::response_like;
use crate::domain::retrospect::entity::retro_reference;
use crate::domain::retrospect::entity::retro_room;
use crate::domain::retrospect::entity::retrospect;
use crate::domain::team::entity::member_team;
use crate::domain::team::entity::team;
use crate::state::AppState;
use crate::utils::error::AppError;

use super::dto::{
    AnalysisResponse, CommentItem, CreateCommentRequest, CreateCommentResponse,
    CreateParticipantResponse, CreateRetrospectRequest, CreateRetrospectResponse, DraftItem,
    DraftSaveRequest, DraftSaveResponse, ListCommentsResponse, ReferenceItem, ResponseCategory,
    ResponseListItem, ResponsesListResponse, RetrospectDetailResponse, RetrospectMemberItem,
    RetrospectQuestionItem, SearchQueryParams, SearchRetrospectItem, StorageQueryParams,
    StorageResponse, StorageRetrospectItem, StorageYearGroup, SubmitAnswerItem,
    SubmitRetrospectRequest, SubmitRetrospectResponse, TeamRetrospectListItem,
    REFERENCE_URL_MAX_LENGTH,
};

/// 회고당 질문 수 (고정)
const QUESTIONS_PER_RETROSPECT: usize = 5;

pub struct RetrospectService;

impl RetrospectService {
    /// 회고 생성
    pub async fn create_retrospect(
        state: AppState,
        user_id: i64,
        req: CreateRetrospectRequest,
    ) -> Result<CreateRetrospectResponse, AppError> {
        // 1. 참고 URL 검증
        Self::validate_reference_urls(&req.reference_urls)?;

        // 2. 날짜 및 시간 형식 검증
        let retrospect_date = Self::validate_and_parse_date(&req.retrospect_date)?;
        let retrospect_time = Self::validate_and_parse_time(&req.retrospect_time)?;

        // 3. 미래 날짜/시간 검증
        Self::validate_future_datetime(retrospect_date, retrospect_time)?;

        // 4. 팀 존재 여부 확인
        let team_exists = team::Entity::find_by_id(req.team_id)
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        if team_exists.is_none() {
            return Err(AppError::TeamNotFound(
                "존재하지 않는 팀입니다.".to_string(),
            ));
        }

        // 5. 팀 멤버십 확인
        let is_member = member_team::Entity::find()
            .filter(member_team::Column::MemberId.eq(user_id))
            .filter(member_team::Column::TeamId.eq(req.team_id))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        if is_member.is_none() {
            return Err(AppError::TeamAccessDenied(
                "해당 팀의 멤버가 아닙니다.".to_string(),
            ));
        }

        // 6. 트랜잭션 시작
        let txn = state
            .db
            .begin()
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // 7. 회고방 생성
        let now = Utc::now().naive_utc();
        let base_url = std::env::var("INVITATION_BASE_URL")
            .unwrap_or_else(|_| "https://retro.example.com".to_string());
        let invitation_url = format!(
            "{}/room/{}",
            base_url.trim_end_matches('/'),
            uuid::Uuid::new_v4()
        );

        let retro_room_model = retro_room::ActiveModel {
            title: Set(req.project_name.clone()),
            invitation_url: Set(invitation_url),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let retro_room_result = retro_room_model
            .insert(&txn)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        let retrospect_room_id = retro_room_result.retrospect_room_id;

        // 8. 회고 생성
        let start_time = NaiveDateTime::new(retrospect_date, retrospect_time);

        let retrospect_model = retrospect::ActiveModel {
            title: Set(req.project_name.clone()),
            team_insight: Set(None),
            retrospect_method: Set(req.retrospect_method.clone()),
            created_at: Set(now),
            updated_at: Set(now),
            start_time: Set(start_time),
            retrospect_room_id: Set(retrospect_room_id),
            team_id: Set(req.team_id),
            ..Default::default()
        };

        let retrospect_result = retrospect_model
            .insert(&txn)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        let retrospect_id = retrospect_result.retrospect_id;

        // 9. 회고 방식에 따른 기본 질문 생성
        let questions = req.retrospect_method.default_questions();
        for question in questions {
            let response_model = response::ActiveModel {
                question: Set(question.to_string()),
                content: Set(String::new()),
                created_at: Set(now),
                updated_at: Set(now),
                retrospect_id: Set(retrospect_id),
                ..Default::default()
            };

            response_model
                .insert(&txn)
                .await
                .map_err(|e| AppError::InternalError(e.to_string()))?;
        }

        // 10. 참고 URL 저장
        for url in &req.reference_urls {
            let reference_model = retro_reference::ActiveModel {
                title: Set(url.clone()),
                url: Set(url.clone()),
                retrospect_id: Set(retrospect_id),
                ..Default::default()
            };

            reference_model
                .insert(&txn)
                .await
                .map_err(|e| AppError::InternalError(e.to_string()))?;
        }

        // 11. 트랜잭션 커밋
        txn.commit()
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        Ok(CreateRetrospectResponse {
            retrospect_id,
            team_id: req.team_id,
            project_name: req.project_name,
        })
    }

    /// 참고 URL 검증
    fn validate_reference_urls(urls: &[String]) -> Result<(), AppError> {
        // 중복 검증
        let unique_urls: HashSet<_> = urls.iter().collect();
        if unique_urls.len() != urls.len() {
            return Err(AppError::RetroUrlInvalid(
                "중복된 URL이 있습니다.".to_string(),
            ));
        }

        // 각 URL 형식 검증
        for url in urls {
            // 최대 길이 검증
            if url.len() > REFERENCE_URL_MAX_LENGTH {
                return Err(AppError::RetroUrlInvalid(format!(
                    "URL은 최대 {}자까지 허용됩니다.",
                    REFERENCE_URL_MAX_LENGTH
                )));
            }

            // URL 형식 검증 (http:// 또는 https://로 시작해야 함)
            let without_scheme = if let Some(stripped) = url.strip_prefix("https://") {
                stripped
            } else if let Some(stripped) = url.strip_prefix("http://") {
                stripped
            } else {
                return Err(AppError::RetroUrlInvalid(
                    "유효하지 않은 URL 형식입니다.".to_string(),
                ));
            };

            // 기본 URL 형식 검증 (스키마 이후에 호스트가 있어야 함)
            if without_scheme.is_empty() || !without_scheme.contains('.') {
                return Err(AppError::RetroUrlInvalid(
                    "유효하지 않은 URL 형식입니다.".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// 날짜 형식 및 미래 날짜 검증
    fn validate_and_parse_date(date_str: &str) -> Result<NaiveDate, AppError> {
        // YYYY-MM-DD 형식 파싱
        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|_| {
            AppError::BadRequest(
                "날짜 형식이 올바르지 않습니다. (YYYY-MM-DD 형식 필요)".to_string(),
            )
        })?;

        // 오늘 이후 날짜 검증 (오늘 포함)
        let today = Utc::now().date_naive();
        if date < today {
            return Err(AppError::BadRequest(
                "회고 날짜는 오늘 이후만 허용됩니다.".to_string(),
            ));
        }

        Ok(date)
    }

    /// 시간 형식 검증
    fn validate_and_parse_time(time_str: &str) -> Result<NaiveTime, AppError> {
        // HH:mm 형식 파싱
        NaiveTime::parse_from_str(time_str, "%H:%M").map_err(|_| {
            AppError::BadRequest("시간 형식이 올바르지 않습니다. (HH:mm 형식 필요)".to_string())
        })
    }

    /// 미래 날짜/시간 검증 (한국 시간 기준, UTC+9)
    fn validate_future_datetime(date: NaiveDate, time: NaiveTime) -> Result<(), AppError> {
        let input_datetime = NaiveDateTime::new(date, time);

        // 한국 시간 기준 현재 시각 (UTC + 9시간)
        let now_kst = Utc::now().naive_utc() + chrono::Duration::hours(9);

        if input_datetime <= now_kst {
            return Err(AppError::BadRequest(
                "회고 날짜와 시간은 현재보다 미래여야 합니다.".to_string(),
            ));
        }

        Ok(())
    }

    /// 팀 회고 목록 조회 (API-010)
    pub async fn list_team_retrospects(
        state: AppState,
        user_id: i64,
        team_id: i64,
    ) -> Result<Vec<TeamRetrospectListItem>, AppError> {
        // 1. 팀 존재 여부 확인
        let team_exists = team::Entity::find_by_id(team_id)
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        if team_exists.is_none() {
            return Err(AppError::TeamNotFound(
                "존재하지 않는 팀입니다.".to_string(),
            ));
        }

        // 2. 팀 멤버십 확인
        let is_member = member_team::Entity::find()
            .filter(member_team::Column::MemberId.eq(user_id))
            .filter(member_team::Column::TeamId.eq(team_id))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        if is_member.is_none() {
            return Err(AppError::TeamAccessDenied(
                "해당 팀에 접근 권한이 없습니다.".to_string(),
            ));
        }

        // 3. 팀에 속한 회고 목록 조회 (최신순 정렬, 동일 시간일 경우 ID 역순으로 안정 정렬)
        let retrospects = retrospect::Entity::find()
            .filter(retrospect::Column::TeamId.eq(team_id))
            .order_by_desc(retrospect::Column::StartTime)
            .order_by_desc(retrospect::Column::RetrospectId)
            .all(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // 4. DTO 변환
        let result: Vec<TeamRetrospectListItem> =
            retrospects.into_iter().map(|r| r.into()).collect();

        Ok(result)
    }

    /// 회고 조회 및 팀 멤버십 확인 헬퍼
    /// 비멤버에게 회고 존재 여부를 노출하지 않도록
    /// "존재하지 않음"과 "접근 권한 없음"을 동일한 404로 처리
    async fn find_retrospect_for_member(
        state: &AppState,
        user_id: i64,
        retrospect_id: i64,
    ) -> Result<retrospect::Model, AppError> {
        let retrospect_model = retrospect::Entity::find_by_id(retrospect_id)
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .ok_or_else(|| {
                AppError::RetrospectNotFound(
                    "존재하지 않는 회고이거나 접근 권한이 없습니다.".to_string(),
                )
            })?;

        let is_member = member_team::Entity::find()
            .filter(member_team::Column::MemberId.eq(user_id))
            .filter(member_team::Column::TeamId.eq(retrospect_model.team_id))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        if is_member.is_none() {
            return Err(AppError::RetrospectNotFound(
                "존재하지 않는 회고이거나 접근 권한이 없습니다.".to_string(),
            ));
        }

        Ok(retrospect_model)
    }

    /// 회고 참석자 등록 (API-014)
    pub async fn create_participant(
        state: AppState,
        user_id: i64,
        retrospect_id: i64,
    ) -> Result<CreateParticipantResponse, AppError> {
        // 1. 회고 조회 및 팀 멤버십 확인
        let retrospect_model =
            Self::find_retrospect_for_member(&state, user_id, retrospect_id).await?;

        // 2. 진행 예정인 회고인지 확인 (과거 회고에는 참석 불가)
        let now_kst = Utc::now().naive_utc() + chrono::Duration::hours(9);
        if retrospect_model.start_time <= now_kst {
            return Err(AppError::RetrospectAlreadyStarted(
                "이미 시작되었거나 종료된 회고에는 참석할 수 없습니다.".to_string(),
            ));
        }

        // 3. 이미 참석자로 등록되어 있는지 확인
        let existing_participant = member_retro::Entity::find()
            .filter(member_retro::Column::MemberId.eq(user_id))
            .filter(member_retro::Column::RetrospectId.eq(retrospect_id))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        if existing_participant.is_some() {
            return Err(AppError::ParticipantDuplicate(
                "이미 참석자로 등록되어 있습니다.".to_string(),
            ));
        }

        // 4. member 정보 조회하여 nickname 추출 (이메일에서 @ 앞부분 추출)
        let member_model = member::Entity::find_by_id(user_id)
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .ok_or_else(|| AppError::InternalError("회원 정보를 찾을 수 없습니다.".to_string()))?;

        let nickname = member_model
            .email
            .split('@')
            .next()
            .unwrap_or(&member_model.email)
            .to_string();

        // 5. member_retro 테이블에 새 레코드 삽입
        let member_retro_model = member_retro::ActiveModel {
            member_id: Set(user_id),
            retrospect_id: Set(retrospect_id),
            personal_insight: Set(None),
            ..Default::default()
        };

        let inserted = member_retro_model.insert(&state.db).await.map_err(|e| {
            // DB 유니크 제약 위반 시 409 Conflict로 매핑
            let error_msg = e.to_string().to_lowercase();
            if error_msg.contains("duplicate")
                || error_msg.contains("unique")
                || error_msg.contains("constraint")
            {
                AppError::ParticipantDuplicate("이미 참석자로 등록되어 있습니다.".to_string())
            } else {
                AppError::InternalError(e.to_string())
            }
        })?;

        // 6. CreateParticipantResponse 반환
        Ok(CreateParticipantResponse {
            participant_id: inserted.member_retro_id,
            member_id: user_id,
            nickname,
        })
    }

    /// 회고 참고자료 목록 조회 (API-018)
    pub async fn list_references(
        state: AppState,
        user_id: i64,
        retrospect_id: i64,
    ) -> Result<Vec<ReferenceItem>, AppError> {
        // 1. 회고 조회 및 팀 멤버십 확인
        let _retrospect_model =
            Self::find_retrospect_for_member(&state, user_id, retrospect_id).await?;

        // 2. 참고자료 목록 조회 (referenceId 오름차순)
        let references = retro_reference::Entity::find()
            .filter(retro_reference::Column::RetrospectId.eq(retrospect_id))
            .order_by_asc(retro_reference::Column::RetroReferenceId)
            .all(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // 3. DTO 변환
        let result: Vec<ReferenceItem> = references
            .into_iter()
            .map(|r| ReferenceItem {
                reference_id: r.retro_reference_id,
                url_name: r.title,
                url: r.url,
            })
            .collect();

        Ok(result)
    }

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

        // 5. 각 회고의 참여자 수 조회 (단일 배치 쿼리)
        let all_member_retros_for_count = member_retro::Entity::find()
            .filter(member_retro::Column::RetrospectId.is_in(retrospect_ids.clone()))
            .all(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        let mut member_counts: HashMap<i64, i64> = HashMap::new();
        for mr in &all_member_retros_for_count {
            *member_counts.entry(mr.retrospect_id).or_insert(0) += 1;
        }

        // 6. 연도별 그룹핑 (BTreeMap으로 정렬)
        let mut year_groups: BTreeMap<i32, Vec<StorageRetrospectItem>> = BTreeMap::new();

        // member_retro에서 submitted_at 기준으로 날짜 매핑
        let submitted_dates: HashMap<i64, chrono::NaiveDateTime> = member_retros
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
                retrospect_method: retro.retrospect_method.clone(),
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

    /// 회고 상세 정보 조회 (API-012)
    pub async fn get_retrospect_detail(
        state: AppState,
        user_id: i64,
        retrospect_id: i64,
    ) -> Result<RetrospectDetailResponse, AppError> {
        info!(
            user_id = user_id,
            retrospect_id = retrospect_id,
            "회고 상세 정보 조회 요청"
        );

        // 1. 회고 존재 여부 확인
        let retrospect_model = retrospect::Entity::find_by_id(retrospect_id)
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .ok_or_else(|| AppError::RetrospectNotFound("존재하지 않는 회고입니다.".to_string()))?;

        // 2. 접근 권한 확인 (해당 회고가 속한 팀의 멤버인지 확인)
        let retrospect_room_id = retrospect_model.retrospect_room_id;
        let is_team_member = member_retro_room::Entity::find()
            .filter(member_retro_room::Column::MemberId.eq(user_id))
            .filter(member_retro_room::Column::RetrospectRoomId.eq(retrospect_room_id))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        if is_team_member.is_none() {
            return Err(AppError::TeamAccessDenied(
                "해당 회고에 접근 권한이 없습니다.".to_string(),
            ));
        }

        // 3. 참여 멤버 조회 (member_retro + member 조인, 등록일 기준 오름차순)
        let member_retros = member_retro::Entity::find()
            .filter(member_retro::Column::RetrospectId.eq(retrospect_id))
            .order_by_asc(member_retro::Column::MemberRetroId)
            .all(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        let member_ids: Vec<i64> = member_retros.iter().map(|mr| mr.member_id).collect();

        let members = if member_ids.is_empty() {
            vec![]
        } else {
            member::Entity::find()
                .filter(member::Column::MemberId.is_in(member_ids))
                .all(&state.db)
                .await
                .map_err(|e| AppError::InternalError(e.to_string()))?
        };

        let member_map: HashMap<i64, String> = members
            .iter()
            .map(|m| {
                let nickname = m
                    .nickname
                    .clone()
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| "Unknown".to_string());
                (m.member_id, nickname)
            })
            .collect();

        // member_retro 순서 유지 (참석 등록일 기준 오름차순)
        let member_items: Vec<RetrospectMemberItem> = member_retros
            .iter()
            .filter_map(|mr| {
                let name = member_map.get(&mr.member_id);
                if name.is_none() {
                    warn!(
                        member_id = mr.member_id,
                        retrospect_id = retrospect_id,
                        "member_retro에 등록되어 있으나 member 테이블에 존재하지 않는 멤버"
                    );
                }
                name.map(|n| RetrospectMemberItem {
                    member_id: mr.member_id,
                    user_name: n.clone(),
                })
            })
            .collect();

        // 4. 해당 회고의 전체 응답(response) 조회
        let responses = response::Entity::find()
            .filter(response::Column::RetrospectId.eq(retrospect_id))
            .order_by_asc(response::Column::ResponseId)
            .all(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        let response_ids: Vec<i64> = responses.iter().map(|r| r.response_id).collect();

        // 5. 질문 리스트 추출 (중복 제거, 순서 유지, 최대 5개)
        let mut seen_questions = HashSet::new();
        let questions: Vec<RetrospectQuestionItem> = responses
            .iter()
            .filter(|r| seen_questions.insert(r.question.clone()))
            .take(5)
            .enumerate()
            .map(|(i, r)| RetrospectQuestionItem {
                index: (i + 1) as i32,
                content: r.question.clone(),
            })
            .collect();

        // 6. 전체 좋아요 수 조회
        let total_like_count = if response_ids.is_empty() {
            0
        } else {
            response_like::Entity::find()
                .filter(response_like::Column::ResponseId.is_in(response_ids.clone()))
                .count(&state.db)
                .await
                .map_err(|e| AppError::InternalError(e.to_string()))? as i64
        };

        // 7. 전체 댓글 수 조회
        let total_comment_count = if response_ids.is_empty() {
            0
        } else {
            response_comment::Entity::find()
                .filter(response_comment::Column::ResponseId.is_in(response_ids))
                .count(&state.db)
                .await
                .map_err(|e| AppError::InternalError(e.to_string()))? as i64
        };

        // 8. 시작일 포맷 (start_time은 생성 시 KST로 저장되므로 변환 불필요)
        let start_time = retrospect_model.start_time.format("%Y-%m-%d").to_string();

        Ok(RetrospectDetailResponse {
            team_id: retrospect_room_id,
            title: retrospect_model.title,
            start_time,
            retro_category: retrospect_model.retrospect_method,
            members: member_items,
            total_like_count,
            total_comment_count,
            questions,
        })
    }

    /// 검색 키워드 검증
    fn validate_search_keyword(keyword: Option<&str>) -> Result<String, AppError> {
        let trimmed = keyword.unwrap_or("").trim().to_string();

        if trimmed.is_empty() {
            return Err(AppError::SearchKeywordInvalid(
                "검색어를 입력해주세요.".to_string(),
            ));
        }

        if trimmed.chars().count() > 100 {
            return Err(AppError::SearchKeywordInvalid(
                "검색어는 최대 100자까지 입력 가능합니다.".to_string(),
            ));
        }

        Ok(trimmed)
    }

    /// 회고 검색 (API-023)
    pub async fn search_retrospects(
        state: AppState,
        user_id: i64,
        params: SearchQueryParams,
    ) -> Result<Vec<SearchRetrospectItem>, AppError> {
        // 1. 키워드 검증
        let keyword = Self::validate_search_keyword(params.keyword.as_deref())?;

        info!(
            user_id = user_id,
            keyword = %keyword,
            "회고 검색 요청"
        );

        // 2. 사용자가 속한 팀 목록 조회
        let user_teams = member_team::Entity::find()
            .filter(member_team::Column::MemberId.eq(user_id))
            .all(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        if user_teams.is_empty() {
            return Ok(vec![]);
        }

        let team_ids: Vec<i64> = user_teams.iter().map(|mt| mt.team_id).collect();

        // 3. 팀 정보 조회 (팀명 매핑)
        let teams = team::Entity::find()
            .filter(team::Column::TeamId.is_in(team_ids.clone()))
            .all(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        let team_map: HashMap<i64, String> =
            teams.iter().map(|t| (t.team_id, t.name.clone())).collect();

        // 4. 해당 팀들의 회고 중 키워드가 포함된 회고 검색 (동일 시간대 안정 정렬을 위해 ID 보조 정렬 추가)
        let retrospects = retrospect::Entity::find()
            .filter(retrospect::Column::TeamId.is_in(team_ids))
            .filter(retrospect::Column::Title.contains(&keyword))
            .order_by_desc(retrospect::Column::StartTime)
            .order_by_desc(retrospect::Column::RetrospectId)
            .all(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // 5. 응답 DTO 변환 (start_time은 생성 시 KST로 저장되므로 변환 불필요)
        let items: Vec<SearchRetrospectItem> = retrospects
            .iter()
            .map(|r| SearchRetrospectItem {
                retrospect_id: r.retrospect_id,
                project_name: r.title.clone(),
                team_name: team_map.get(&r.team_id).cloned().unwrap_or_default(),
                retrospect_method: r.retrospect_method.clone(),
                retrospect_date: r.start_time.format("%Y-%m-%d").to_string(),
                retrospect_time: r.start_time.format("%H:%M").to_string(),
            })
            .collect();

        info!(
            user_id = user_id,
            keyword = %keyword,
            result_count = items.len(),
            "회고 검색 완료"
        );

        Ok(items)
    }

    /// 회고 내보내기 (API-021) - PDF 바이트 생성
    pub async fn export_retrospect(
        state: AppState,
        user_id: i64,
        retrospect_id: i64,
    ) -> Result<Vec<u8>, AppError> {
        info!(
            user_id = user_id,
            retrospect_id = retrospect_id,
            "회고 내보내기 요청"
        );

        // 1. 회고 조회 및 팀 멤버십 확인
        let retrospect_model =
            Self::find_retrospect_for_member(&state, user_id, retrospect_id).await?;

        // 2. 팀 이름 조회
        let team_model = team::Entity::find_by_id(retrospect_model.team_id)
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        let team_name = team_model
            .map(|t| t.name)
            .unwrap_or_else(|| "(알 수 없음)".to_string());

        // 3. 참여 멤버 조회
        let member_retros = member_retro::Entity::find()
            .filter(member_retro::Column::RetrospectId.eq(retrospect_id))
            .order_by_asc(member_retro::Column::MemberRetroId)
            .all(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        let member_ids: Vec<i64> = member_retros.iter().map(|mr| mr.member_id).collect();

        let members = if member_ids.is_empty() {
            vec![]
        } else {
            member::Entity::find()
                .filter(member::Column::MemberId.is_in(member_ids))
                .all(&state.db)
                .await
                .map_err(|e| AppError::InternalError(e.to_string()))?
        };

        let member_map: HashMap<i64, String> = members
            .iter()
            .map(|m| (m.member_id, m.nickname.clone().unwrap_or_default()))
            .collect();

        // 4. 질문/답변 조회
        let responses = response::Entity::find()
            .filter(response::Column::RetrospectId.eq(retrospect_id))
            .order_by_asc(response::Column::ResponseId)
            .all(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // 4-1. 답변-멤버 매핑 조회
        let response_ids: Vec<i64> = responses.iter().map(|r| r.response_id).collect();
        let response_member_map: HashMap<i64, i64> = if response_ids.is_empty() {
            HashMap::new()
        } else {
            member_response::Entity::find()
                .filter(member_response::Column::ResponseId.is_in(response_ids))
                .all(&state.db)
                .await
                .map_err(|e| AppError::InternalError(e.to_string()))?
                .into_iter()
                .map(|mr| (mr.response_id, mr.member_id))
                .collect()
        };

        // 5. PDF 생성
        let pdf_bytes = Self::generate_pdf(
            &retrospect_model,
            &team_name,
            &member_retros,
            &member_map,
            &responses,
            &response_member_map,
        )?;

        info!(
            retrospect_id = retrospect_id,
            pdf_size = pdf_bytes.len(),
            "회고 PDF 생성 완료"
        );

        Ok(pdf_bytes)
    }

    /// 회고 삭제 (API-013)
    ///
    /// TODO: 현재 스키마에 `created_by`(회고 생성자) 필드와 `member_team.role`(팀 역할) 필드가 없어
    /// 팀 멤버십만 확인합니다. 스펙상 팀 Owner 또는 회고 생성자만 삭제 가능해야 하므로,
    /// 스키마 마이그레이션 후 권한 분기를 추가해야 합니다.
    pub async fn delete_retrospect(
        state: AppState,
        user_id: i64,
        retrospect_id: i64,
    ) -> Result<(), AppError> {
        info!(
            user_id = user_id,
            retrospect_id = retrospect_id,
            "회고 삭제 요청"
        );

        // 1. 회고 조회 및 팀 멤버십 확인
        let retrospect_model =
            Self::find_retrospect_for_member(&state, user_id, retrospect_id).await?;

        let retrospect_room_id = retrospect_model.retrospect_room_id;

        // 2. 트랜잭션 시작 (연관 데이터 일괄 삭제)
        let txn = state
            .db
            .begin()
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // 3. 해당 회고의 모든 응답(response) ID만 조회 (전체 모델 불필요)
        let response_ids: Vec<i64> = response::Entity::find()
            .filter(response::Column::RetrospectId.eq(retrospect_id))
            .select_only()
            .column(response::Column::ResponseId)
            .into_tuple()
            .all(&txn)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        if !response_ids.is_empty() {
            // 4. 댓글 삭제 (response_comment)
            let comments_deleted = response_comment::Entity::delete_many()
                .filter(response_comment::Column::ResponseId.is_in(response_ids.clone()))
                .exec(&txn)
                .await
                .map_err(|e| AppError::InternalError(e.to_string()))?;

            // 5. 좋아요 삭제 (response_like)
            let likes_deleted = response_like::Entity::delete_many()
                .filter(response_like::Column::ResponseId.is_in(response_ids.clone()))
                .exec(&txn)
                .await
                .map_err(|e| AppError::InternalError(e.to_string()))?;

            // 6. 멤버 응답 매핑 삭제 (member_response)
            let member_responses_deleted = member_response::Entity::delete_many()
                .filter(member_response::Column::ResponseId.is_in(response_ids.clone()))
                .exec(&txn)
                .await
                .map_err(|e| AppError::InternalError(e.to_string()))?;

            info!(
                retrospect_id = retrospect_id,
                response_count = response_ids.len(),
                comments_deleted = comments_deleted.rows_affected,
                likes_deleted = likes_deleted.rows_affected,
                member_responses_deleted = member_responses_deleted.rows_affected,
                "연관 응답 데이터 삭제 완료"
            );
        }

        // 7. 응답 삭제 (response)
        let responses_deleted = response::Entity::delete_many()
            .filter(response::Column::RetrospectId.eq(retrospect_id))
            .exec(&txn)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // 8. 참고자료 삭제 (retro_reference)
        let references_deleted = retro_reference::Entity::delete_many()
            .filter(retro_reference::Column::RetrospectId.eq(retrospect_id))
            .exec(&txn)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // 9. 멤버-회고 매핑 삭제 (member_retro)
        let member_retros_deleted = member_retro::Entity::delete_many()
            .filter(member_retro::Column::RetrospectId.eq(retrospect_id))
            .exec(&txn)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // 10. 회고 삭제
        retrospect_model
            .delete(&txn)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // 11. 회고방 삭제 (같은 room을 참조하는 다른 회고가 없는 경우에만)
        let other_retro_count = retrospect::Entity::find()
            .filter(retrospect::Column::RetrospectRoomId.eq(retrospect_room_id))
            .count(&txn)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        let (member_retro_rooms_deleted, room_deleted) = if other_retro_count == 0 {
            // 회고방을 참조하는 다른 회고가 없으므로 멤버-회고방 매핑과 회고방 모두 삭제
            let member_retro_rooms_deleted = member_retro_room::Entity::delete_many()
                .filter(member_retro_room::Column::RetrospectRoomId.eq(retrospect_room_id))
                .exec(&txn)
                .await
                .map_err(|e| AppError::InternalError(e.to_string()))?;

            let room_deleted = retro_room::Entity::delete_many()
                .filter(retro_room::Column::RetrospectRoomId.eq(retrospect_room_id))
                .exec(&txn)
                .await
                .map_err(|e| AppError::InternalError(e.to_string()))?;

            (
                member_retro_rooms_deleted.rows_affected,
                room_deleted.rows_affected,
            )
        } else {
            warn!(
                retrospect_room_id = retrospect_room_id,
                other_retro_count = other_retro_count,
                "회고방을 공유하는 다른 회고가 존재하여 회고방 삭제를 건너뜁니다"
            );
            (0, 0)
        };

        // 12. 트랜잭션 커밋
        txn.commit()
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        info!(
            retrospect_id = retrospect_id,
            responses_deleted = responses_deleted.rows_affected,
            references_deleted = references_deleted.rows_affected,
            member_retros_deleted = member_retros_deleted.rows_affected,
            member_retro_rooms_deleted = member_retro_rooms_deleted,
            room_deleted = room_deleted,
            "회고 및 연관 데이터 삭제 완료"
        );

        Ok(())
    }

    /// 회고 방식 표시명 반환
    fn retrospect_method_display(method: &retrospect::RetrospectMethod) -> String {
        match method {
            retrospect::RetrospectMethod::Kpt => "KPT".to_string(),
            retrospect::RetrospectMethod::FourL => "4L".to_string(),
            retrospect::RetrospectMethod::FiveF => "5F".to_string(),
            retrospect::RetrospectMethod::Pmi => "PMI".to_string(),
            retrospect::RetrospectMethod::Free => "Free".to_string(),
        }
    }

    /// PDF 문서 생성
    fn generate_pdf(
        retrospect_model: &retrospect::Model,
        team_name: &str,
        member_retros: &[member_retro::Model],
        member_map: &HashMap<i64, String>,
        responses: &[response::Model],
        response_member_map: &HashMap<i64, i64>,
    ) -> Result<Vec<u8>, AppError> {
        // 폰트 로딩
        let font_dir = std::env::var("PDF_FONT_DIR").unwrap_or_else(|_| "./fonts".to_string());
        let font_family_name =
            std::env::var("PDF_FONT_FAMILY").unwrap_or_else(|_| "NanumGothic".to_string());

        let font_family = match genpdf::fonts::from_files(&font_dir, &font_family_name, None) {
            Ok(family) => family,
            Err(full_err) => {
                warn!(
                    "전체 폰트 패밀리 로딩 실패 ({}), Regular 폰트로 대체합니다.",
                    full_err
                );
                let regular_path = std::path::Path::new(&font_dir)
                    .join(format!("{}-Regular.ttf", font_family_name));
                let font_bytes = std::fs::read(&regular_path).map_err(|e| {
                    AppError::PdfGenerationFailed(format!(
                        "Regular 폰트 파일 읽기 실패 ({}) : {}",
                        regular_path.display(),
                        e
                    ))
                })?;
                genpdf::fonts::FontFamily {
                    regular: genpdf::fonts::FontData::new(font_bytes.clone(), None).map_err(
                        |e| {
                            AppError::PdfGenerationFailed(format!(
                                "Regular 폰트 데이터 로딩 실패: {}",
                                e
                            ))
                        },
                    )?,
                    bold: genpdf::fonts::FontData::new(font_bytes.clone(), None).map_err(|e| {
                        AppError::PdfGenerationFailed(format!("Bold 폰트 데이터 로딩 실패: {}", e))
                    })?,
                    italic: genpdf::fonts::FontData::new(font_bytes.clone(), None).map_err(
                        |e| {
                            AppError::PdfGenerationFailed(format!(
                                "Italic 폰트 데이터 로딩 실패: {}",
                                e
                            ))
                        },
                    )?,
                    bold_italic: genpdf::fonts::FontData::new(font_bytes, None).map_err(|e| {
                        AppError::PdfGenerationFailed(format!(
                            "BoldItalic 폰트 데이터 로딩 실패: {}",
                            e
                        ))
                    })?,
                }
            }
        };

        let mut doc = genpdf::Document::new(font_family);
        doc.set_title(format!("{} - Retrospect Report", retrospect_model.title));
        doc.set_minimal_conformance();

        // 페이지 여백 설정
        let mut decorator = genpdf::SimplePageDecorator::new();
        decorator.set_margins(15);
        doc.set_page_decorator(decorator);

        // ===== 제목 섹션 =====
        doc.push(
            Paragraph::new(format!("{} - Retrospect Report", retrospect_model.title))
                .styled(style::Style::new().bold().with_font_size(18)),
        );
        doc.push(Break::new(0.5));

        // ===== 기본 정보 섹션 =====
        let method_str = Self::retrospect_method_display(&retrospect_model.retrospect_method);
        let date_str = retrospect_model.start_time.format("%Y-%m-%d").to_string();
        let time_str = retrospect_model.start_time.format("%H:%M").to_string();

        doc.push(
            Paragraph::new("Basic Information")
                .styled(style::Style::new().bold().with_font_size(14)),
        );
        doc.push(Break::new(0.3));
        doc.push(Paragraph::new(format!("Team: {}", team_name)));
        doc.push(Paragraph::new(format!("Date: {} {}", date_str, time_str)));
        doc.push(Paragraph::new(format!("Method: {}", method_str)));

        // 참여 멤버 목록
        let participant_names: Vec<String> = member_retros
            .iter()
            .filter_map(|mr| member_map.get(&mr.member_id).cloned())
            .collect();
        doc.push(Paragraph::new(format!(
            "Participants ({}):",
            participant_names.len()
        )));
        for name in &participant_names {
            doc.push(Paragraph::new(format!("  - {}", name)));
        }
        doc.push(Break::new(0.5));

        // ===== 팀 인사이트 섹션 =====
        if let Some(ref insight) = retrospect_model.team_insight {
            doc.push(
                Paragraph::new("Team Insight")
                    .styled(style::Style::new().bold().with_font_size(14)),
            );
            doc.push(Break::new(0.3));
            doc.push(Paragraph::new(insight.clone()));
            doc.push(Break::new(0.5));
        }

        // ===== 질문/답변 섹션 =====
        if !responses.is_empty() {
            doc.push(
                Paragraph::new("Questions & Answers")
                    .styled(style::Style::new().bold().with_font_size(14)),
            );
            doc.push(Break::new(0.3));

            // 중복 제거된 질문 추출
            let mut seen_questions = HashSet::new();
            let unique_questions: Vec<&response::Model> = responses
                .iter()
                .filter(|r| seen_questions.insert(r.question.clone()))
                .collect();

            for (i, question_response) in unique_questions.iter().enumerate() {
                doc.push(
                    Paragraph::new(format!("Q{}. {}", i + 1, question_response.question))
                        .styled(style::Style::new().bold()),
                );

                // 해당 질문에 대한 모든 답변 수집
                let answers_for_question: Vec<&response::Model> = responses
                    .iter()
                    .filter(|r| {
                        r.question == question_response.question && !r.content.trim().is_empty()
                    })
                    .collect();

                if answers_for_question.is_empty() {
                    doc.push(Paragraph::new("  (No answers)"));
                } else {
                    for answer in &answers_for_question {
                        let author = response_member_map
                            .get(&answer.response_id)
                            .and_then(|mid| member_map.get(mid))
                            .cloned()
                            .unwrap_or_else(|| "Anonymous".to_string());
                        doc.push(Paragraph::new(format!(
                            "  - [{}] {}",
                            author, answer.content
                        )));
                    }
                }
                doc.push(Break::new(0.3));
            }
        }

        // ===== 개인 인사이트 섹션 =====
        let members_with_insight: Vec<&member_retro::Model> = member_retros
            .iter()
            .filter(|mr| mr.personal_insight.is_some())
            .collect();

        if !members_with_insight.is_empty() {
            doc.push(Break::new(0.3));
            doc.push(
                Paragraph::new("Personal Insights")
                    .styled(style::Style::new().bold().with_font_size(14)),
            );
            doc.push(Break::new(0.3));

            for mr in &members_with_insight {
                let name = member_map
                    .get(&mr.member_id)
                    .cloned()
                    .unwrap_or_else(|| format!("Member #{}", mr.member_id));
                doc.push(Paragraph::new(format!("[{}]", name)).styled(style::Style::new().bold()));
                if let Some(ref insight) = mr.personal_insight {
                    doc.push(Paragraph::new(format!("  {}", insight)));
                }
                doc.push(Break::new(0.2));
            }
        }

        // PDF 렌더링
        let mut buf = Vec::new();
        doc.render(&mut buf)
            .map_err(|e| AppError::PdfGenerationFailed(format!("PDF 렌더링 실패: {}", e)))?;

        Ok(buf)
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

    /// 회고 분석 (API-022)
    pub async fn analyze_retrospective(
        state: AppState,
        user_id: i64,
        retrospect_id: i64,
    ) -> Result<AnalysisResponse, AppError> {
        info!(
            user_id = user_id,
            retrospect_id = retrospect_id,
            "회고 분석 요청"
        );

        // 1. retrospect_id 검증 (1 이상)
        if retrospect_id < 1 {
            return Err(AppError::BadRequest(
                "유효하지 않은 회고 ID입니다.".to_string(),
            ));
        }

        // 2. 회고 존재 확인 → RETRO4041
        let retrospect_model = retrospect::Entity::find_by_id(retrospect_id)
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .ok_or_else(|| {
                AppError::RetrospectNotFound("존재하지 않는 회고 세션입니다.".to_string())
            })?;

        // 2-1. 이미 분석 완료 여부 확인 (재분석 방지)
        if retrospect_model.team_insight.is_some() {
            return Err(AppError::RetroAlreadyAnalyzed(
                "이미 분석이 완료된 회고입니다.".to_string(),
            ));
        }

        // 3. 팀 멤버십 확인 (팀 기반 접근 제어)
        let is_team_member = member_team::Entity::find()
            .filter(member_team::Column::MemberId.eq(user_id))
            .filter(member_team::Column::TeamId.eq(retrospect_model.team_id))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        if is_team_member.is_none() {
            return Err(AppError::TeamAccessDenied(
                "해당 회고에 접근 권한이 없습니다.".to_string(),
            ));
        }

        let retrospect_room_id = retrospect_model.retrospect_room_id;

        // 4. 월간 사용량 확인 (팀당 월 10회 제한)
        let kst_offset = chrono::Duration::hours(9);
        let now_kst = Utc::now().naive_utc() + kst_offset;
        let current_month_start =
            chrono::NaiveDate::from_ymd_opt(now_kst.year(), now_kst.month(), 1)
                .ok_or_else(|| AppError::InternalError("날짜 계산 오류".to_string()))?
                .and_hms_opt(0, 0, 0)
                .ok_or_else(|| AppError::InternalError("시간 계산 오류".to_string()))?
                - kst_offset; // UTC로 변환

        // 현재 월에 team_insight가 NOT NULL인 회고 수 카운트 (분석 시점 = updated_at 기준)
        let monthly_analysis_count = retrospect::Entity::find()
            .filter(retrospect::Column::RetrospectRoomId.eq(retrospect_room_id))
            .filter(retrospect::Column::TeamInsight.is_not_null())
            .filter(retrospect::Column::UpdatedAt.gte(current_month_start))
            .count(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            as i32;

        if monthly_analysis_count >= 10 {
            return Err(AppError::AiMonthlyLimitExceeded(
                "월간 분석 가능 횟수를 초과하였습니다.".to_string(),
            ));
        }

        // 5. 최소 데이터 기준 확인
        // 5-1. 제출 완료 참여자 수 (member_retro에서 status = SUBMITTED 또는 ANALYZED)
        let submitted_members = member_retro::Entity::find()
            .filter(member_retro::Column::RetrospectId.eq(retrospect_id))
            .filter(
                member_retro::Column::Status
                    .is_in([RetrospectStatus::Submitted, RetrospectStatus::Analyzed]),
            )
            .all(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        if submitted_members.is_empty() {
            return Err(AppError::RetroInsufficientData(
                "분석할 회고 답변 데이터가 부족합니다.".to_string(),
            ));
        }

        // 5-2. 답변 수 확인 (content != "" 카운트)
        let all_responses = response::Entity::find()
            .filter(response::Column::RetrospectId.eq(retrospect_id))
            .all(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        let answer_count = all_responses
            .iter()
            .filter(|r| !r.content.trim().is_empty())
            .count();

        if answer_count < 3 {
            return Err(AppError::RetroInsufficientData(
                "분석할 회고 답변 데이터가 부족합니다.".to_string(),
            ));
        }

        // 6. 참여자 목록 조회 (member_retro + member 조인)
        let member_ids: Vec<i64> = submitted_members.iter().map(|mr| mr.member_id).collect();

        let members = if member_ids.is_empty() {
            vec![]
        } else {
            member::Entity::find()
                .filter(member::Column::MemberId.is_in(member_ids))
                .all(&state.db)
                .await
                .map_err(|e| AppError::InternalError(e.to_string()))?
        };

        // member_id -> nickname 매핑 (빈 닉네임은 "Unknown"으로 fallback)
        let member_map: HashMap<i64, String> = members
            .iter()
            .map(|m| {
                let nickname = m
                    .nickname
                    .clone()
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| "Unknown".to_string());
                (m.member_id, nickname)
            })
            .collect();

        // 7. 각 멤버의 답변 데이터 수집 (AI 프롬프트 입력용)
        use crate::domain::ai::prompt::MemberAnswerData;

        // member_response 테이블에서 멤버별 response_id 매핑 조회
        let all_member_responses = member_response::Entity::find()
            .filter(
                member_response::Column::MemberId.is_in(
                    submitted_members
                        .iter()
                        .map(|mr| mr.member_id)
                        .collect::<Vec<_>>(),
                ),
            )
            .all(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // response_id -> response 매핑
        let response_map: HashMap<i64, &response::Model> =
            all_responses.iter().map(|r| (r.response_id, r)).collect();

        // member_id -> Vec<response_id> 매핑
        let mut member_response_map: HashMap<i64, Vec<i64>> = HashMap::new();
        for mr in &all_member_responses {
            member_response_map
                .entry(mr.member_id)
                .or_default()
                .push(mr.response_id);
        }

        let mut members_data: Vec<MemberAnswerData> = Vec::new();
        for mr in &submitted_members {
            let username = member_map
                .get(&mr.member_id)
                .cloned()
                .unwrap_or_else(|| format!("사용자{}", mr.member_id));

            let response_ids = member_response_map
                .get(&mr.member_id)
                .cloned()
                .unwrap_or_default();

            let mut answers: Vec<(String, String)> = Vec::new();
            for rid in &response_ids {
                if let Some(resp) = response_map.get(rid) {
                    if resp.retrospect_id == retrospect_id {
                        answers.push((resp.question.clone(), resp.content.clone()));
                    }
                }
            }

            members_data.push(MemberAnswerData {
                user_id: mr.member_id,
                user_name: username,
                answers,
            });
        }

        info!(
            "AI 분석 호출 준비 완료 (response_count={}, member_count={})",
            all_responses.len(),
            members_data.len()
        );

        // 8. AI 서비스 호출
        let mut analysis = state
            .ai_service
            .analyze_retrospective(&members_data)
            .await?;

        // personalMissions의 userId 오름차순 정렬
        analysis.personal_missions.sort_by_key(|pm| pm.user_id);

        let team_insight = analysis.team_insight.clone();
        let personal_missions = &analysis.personal_missions;

        // 9. 트랜잭션으로 결과 저장
        let txn = state
            .db
            .begin()
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // 9-1. retrospects.team_insight 업데이트
        let mut retrospect_active: retrospect::ActiveModel = retrospect_model.clone().into();
        retrospect_active.team_insight = Set(Some(team_insight.clone()));
        retrospect_active.updated_at = Set(Utc::now().naive_utc());
        retrospect_active
            .update(&txn)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // 9-2. 각 member_retro.personal_insight 업데이트 + status = ANALYZED
        for mr in &submitted_members {
            // personal_missions에서 해당 member_id의 미션 찾기
            let personal_insight = personal_missions
                .iter()
                .find(|pm| pm.user_id == mr.member_id)
                .map(|pm| {
                    pm.missions
                        .iter()
                        .map(|m| format!("{}: {}", m.mission_title, m.mission_desc))
                        .collect::<Vec<_>>()
                        .join("\n")
                });

            let mut mr_active: member_retro::ActiveModel = mr.clone().into();
            mr_active.personal_insight = Set(personal_insight);
            mr_active.status = Set(RetrospectStatus::Analyzed);
            mr_active
                .update(&txn)
                .await
                .map_err(|e| AppError::InternalError(e.to_string()))?;
        }

        txn.commit()
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        info!(retrospect_id = retrospect_id, "회고 분석 완료");

        Ok(analysis)
    }

    /// 회고 답변 카테고리별 조회 (API-020)
    pub async fn list_responses(
        state: AppState,
        user_id: i64,
        retrospect_id: i64,
        category: ResponseCategory,
        cursor: Option<i64>,
        size: i64,
    ) -> Result<ResponsesListResponse, AppError> {
        info!(
            user_id = user_id,
            retrospect_id = retrospect_id,
            category = %category,
            cursor = ?cursor,
            size = size,
            "회고 답변 카테고리별 조회 요청"
        );

        // 1. 회고 조회 및 팀 멤버십 확인
        let _retrospect_model =
            Self::find_retrospect_for_member(&state, user_id, retrospect_id).await?;

        // 2. 해당 회고의 모든 response 조회 (response_id 오름차순)
        let all_responses = response::Entity::find()
            .filter(response::Column::RetrospectId.eq(retrospect_id))
            .order_by_asc(response::Column::ResponseId)
            .all(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        if all_responses.is_empty() {
            return Ok(ResponsesListResponse {
                responses: vec![],
                has_next: false,
                next_cursor: None,
            });
        }

        // 3. 질문 텍스트 목록 추출 (첫 참여자의 응답 순서 기준으로 질문 순서 결정)
        //    member_response를 통해 첫 번째 참여자의 응답 세트를 찾고, 질문 순서를 확정
        let first_member_responses = member_response::Entity::find()
            .filter(
                member_response::Column::ResponseId.is_in(
                    all_responses
                        .iter()
                        .map(|r| r.response_id)
                        .collect::<Vec<_>>(),
                ),
            )
            .order_by_asc(member_response::Column::ResponseId)
            .all(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // member_id별로 그룹화하여 첫 번째 멤버의 응답 세트 확인
        let mut member_response_map: HashMap<i64, Vec<i64>> = HashMap::new();
        for mr in &first_member_responses {
            member_response_map
                .entry(mr.member_id)
                .or_default()
                .push(mr.response_id);
        }

        // 첫 번째 멤버의 응답 ID 목록 (오름차순 정렬됨)
        let first_member_id = member_response_map.keys().min().copied();
        let question_response_ids: Vec<i64> = first_member_id
            .and_then(|mid| member_response_map.get(&mid))
            .cloned()
            .unwrap_or_default();

        // 질문 텍스트 순서를 response_id 순으로 매핑
        let response_map: HashMap<i64, &response::Model> =
            all_responses.iter().map(|r| (r.response_id, r)).collect();

        let question_texts: Vec<String> = question_response_ids
            .iter()
            .filter_map(|rid| response_map.get(rid).map(|r| r.question.clone()))
            .collect();

        // 4. 카테고리에 따른 대상 응답 ID 필터링
        let target_response_ids: Vec<i64> = match category.question_index() {
            Some(idx) => {
                // 특정 질문에 대한 답변만 필터링
                if idx >= question_texts.len() {
                    // 해당 질문 번호가 없으면 빈 결과 반환
                    return Ok(ResponsesListResponse {
                        responses: vec![],
                        has_next: false,
                        next_cursor: None,
                    });
                }
                let target_question = &question_texts[idx];
                all_responses
                    .iter()
                    .filter(|r| &r.question == target_question)
                    .map(|r| r.response_id)
                    .collect()
            }
            None => {
                // ALL: 모든 응답
                all_responses.iter().map(|r| r.response_id).collect()
            }
        };

        if target_response_ids.is_empty() {
            return Ok(ResponsesListResponse {
                responses: vec![],
                has_next: false,
                next_cursor: None,
            });
        }

        // 5. 공백만 있는 빈 답변 필터링 (content가 비어있거나 공백만인 응답 제외)
        let valid_response_ids: Vec<i64> = target_response_ids
            .iter()
            .filter(|rid| {
                response_map
                    .get(rid)
                    .map(|r| !r.content.trim().is_empty())
                    .unwrap_or(false)
            })
            .copied()
            .collect();

        if valid_response_ids.is_empty() {
            return Ok(ResponsesListResponse {
                responses: vec![],
                has_next: false,
                next_cursor: None,
            });
        }

        // 6. 커서 기반 페이지네이션 (response_id 내림차순)
        let mut query = response::Entity::find()
            .filter(response::Column::ResponseId.is_in(valid_response_ids))
            .order_by_desc(response::Column::ResponseId);

        if let Some(cursor_id) = cursor {
            query = query.filter(response::Column::ResponseId.lt(cursor_id));
        }

        // size + 1개 조회하여 다음 페이지 존재 여부 확인
        let fetched = query
            .limit(Some((size + 1) as u64))
            .all(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        let has_next = fetched.len() as i64 > size;
        let page_responses: Vec<&response::Model> = fetched.iter().take(size as usize).collect();

        // 빈 페이지인 경우 즉시 빈 응답 반환 (이후 is_in([]) 쿼리 방지)
        if page_responses.is_empty() {
            return Ok(ResponsesListResponse {
                responses: vec![],
                has_next: false,
                next_cursor: None,
            });
        }

        // 7. 응답에 대한 member 정보 조회 (member_response -> member)
        let page_response_ids: Vec<i64> = page_responses.iter().map(|r| r.response_id).collect();

        let member_responses_for_page = member_response::Entity::find()
            .filter(member_response::Column::ResponseId.is_in(page_response_ids.clone()))
            .all(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        let response_to_member: HashMap<i64, i64> = member_responses_for_page
            .iter()
            .map(|mr| (mr.response_id, mr.member_id))
            .collect();

        let member_ids: Vec<i64> = response_to_member
            .values()
            .copied()
            .collect::<HashSet<i64>>()
            .into_iter()
            .collect();

        let members = member::Entity::find()
            .filter(member::Column::MemberId.is_in(member_ids))
            .all(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        let member_map: HashMap<i64, &member::Model> =
            members.iter().map(|m| (m.member_id, m)).collect();

        // 8. 좋아요 수 집계
        let like_counts: Vec<(i64, i64)> = response_like::Entity::find()
            .filter(response_like::Column::ResponseId.is_in(page_response_ids.clone()))
            .select_only()
            .column(response_like::Column::ResponseId)
            .column_as(response_like::Column::ResponseLikeId.count(), "count")
            .group_by(response_like::Column::ResponseId)
            .into_tuple()
            .all(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        let like_count_map: HashMap<i64, i64> = like_counts.into_iter().collect();

        // 9. 댓글 수 집계
        let comment_counts: Vec<(i64, i64)> = response_comment::Entity::find()
            .filter(response_comment::Column::ResponseId.is_in(page_response_ids.clone()))
            .select_only()
            .column(response_comment::Column::ResponseId)
            .column_as(response_comment::Column::ResponseCommentId.count(), "count")
            .group_by(response_comment::Column::ResponseId)
            .into_tuple()
            .all(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        let comment_count_map: HashMap<i64, i64> = comment_counts.into_iter().collect();

        // 10. DTO 변환
        let response_items: Vec<ResponseListItem> = page_responses
            .iter()
            .map(|r| {
                let member_id = response_to_member.get(&r.response_id).copied();
                let user_name = member_id
                    .and_then(|mid| member_map.get(&mid))
                    .and_then(|m| m.nickname.clone())
                    .unwrap_or_default();

                ResponseListItem {
                    response_id: r.response_id,
                    user_name,
                    content: r.content.clone(),
                    like_count: like_count_map.get(&r.response_id).copied().unwrap_or(0),
                    comment_count: comment_count_map.get(&r.response_id).copied().unwrap_or(0),
                }
            })
            .collect();

        // 11. 다음 커서 계산
        let next_cursor = if has_next {
            response_items.last().map(|r| r.response_id)
        } else {
            None
        };

        info!(
            retrospect_id = retrospect_id,
            category = %category,
            response_count = response_items.len(),
            has_next = has_next,
            "회고 답변 카테고리별 조회 완료"
        );

        Ok(ResponsesListResponse {
            responses: response_items,
            has_next,
            next_cursor,
        })
    }

    /// 회고 답변 조회 및 팀 멤버십 확인 헬퍼
    /// - 답변이 존재하지 않으면 RES4041 (404) 반환
    /// - 팀 멤버가 아니면 TEAM4031 (403) 반환
    async fn find_response_for_member(
        state: &AppState,
        user_id: i64,
        response_id: i64,
    ) -> Result<response::Model, AppError> {
        // 1. response 조회
        let response_model = response::Entity::find_by_id(response_id)
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .ok_or_else(|| {
                AppError::ResponseNotFound("존재하지 않는 회고 답변입니다.".to_string())
            })?;

        // 2. response -> retrospect -> team 경로로 팀 정보 조회
        let retrospect_model = retrospect::Entity::find_by_id(response_model.retrospect_id)
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .ok_or_else(|| {
                AppError::InternalError(format!(
                    "데이터 정합성 오류: response_id={}에 연결된 retrospect_id={}가 존재하지 않습니다.",
                    response_id, response_model.retrospect_id
                ))
            })?;

        // 3. 팀 멤버십 확인
        let is_member = member_team::Entity::find()
            .filter(member_team::Column::MemberId.eq(user_id))
            .filter(member_team::Column::TeamId.eq(retrospect_model.team_id))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        if is_member.is_none() {
            return Err(AppError::TeamAccessDenied(
                "해당 리소스에 접근 권한이 없습니다.".to_string(),
            ));
        }

        Ok(response_model)
    }

    /// 회고 답변 댓글 목록 조회 (API-026)
    pub async fn list_comments(
        state: AppState,
        user_id: i64,
        response_id: i64,
        cursor: Option<i64>,
        size: i32,
    ) -> Result<ListCommentsResponse, AppError> {
        // 0. size 범위 검증 (방어적 프로그래밍)
        if !(1..=100).contains(&size) {
            return Err(AppError::BadRequest(
                "size는 1~100 범위의 정수여야 합니다.".to_string(),
            ));
        }

        // 1. 답변 조회 및 팀 멤버십 확인
        let _response_model = Self::find_response_for_member(&state, user_id, response_id).await?;

        // 2. 댓글 목록 조회 (커서 기반 페이지네이션, 최신순 정렬)
        let mut query = response_comment::Entity::find()
            .filter(response_comment::Column::ResponseId.eq(response_id));

        if let Some(cursor_id) = cursor {
            query = query.filter(response_comment::Column::ResponseCommentId.lt(cursor_id));
        }

        let comments = query
            .order_by_desc(response_comment::Column::ResponseCommentId)
            .limit((size + 1) as u64)
            .all(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // 3. 다음 페이지 존재 여부 확인
        let has_next = comments.len() > size as usize;
        let comments = if has_next {
            comments.into_iter().take(size as usize).collect()
        } else {
            comments
        };

        // 4. 작성자 정보 조회
        let member_ids: Vec<i64> = comments.iter().map(|c| c.member_id).collect();
        let members = if !member_ids.is_empty() {
            member::Entity::find()
                .filter(member::Column::MemberId.is_in(member_ids))
                .all(&state.db)
                .await
                .map_err(|e| AppError::InternalError(e.to_string()))?
        } else {
            vec![]
        };

        // member_id -> nickname 매핑
        let member_map: HashMap<i64, String> = members
            .into_iter()
            .map(|m| (m.member_id, m.nickname.clone().unwrap_or_default()))
            .collect();

        // 5. DTO 변환 (KST 시간대 적용)
        let comment_items: Vec<CommentItem> = comments
            .iter()
            .map(|c| {
                let created_at_kst = c.created_at + chrono::Duration::hours(9);
                CommentItem {
                    comment_id: c.response_comment_id,
                    member_id: c.member_id,
                    user_name: member_map
                        .get(&c.member_id)
                        .cloned()
                        .unwrap_or_else(|| "Unknown".to_string()),
                    content: c.content.clone(),
                    created_at: created_at_kst.format("%Y-%m-%dT%H:%M:%S").to_string(),
                }
            })
            .collect();

        // 6. 다음 커서 계산
        let next_cursor = if has_next {
            comment_items.last().map(|c| c.comment_id)
        } else {
            None
        };

        Ok(ListCommentsResponse {
            comments: comment_items,
            has_next,
            next_cursor,
        })
    }

    /// 회고 답변 댓글 작성 (API-027)
    pub async fn create_comment(
        state: AppState,
        user_id: i64,
        response_id: i64,
        req: CreateCommentRequest,
    ) -> Result<CreateCommentResponse, AppError> {
        // 1. 댓글 내용 검증
        // 공백만 있는 댓글 차단
        if req.content.trim().is_empty() {
            return Err(AppError::BadRequest(
                "댓글 내용은 공백만으로 구성될 수 없습니다.".to_string(),
            ));
        }
        // 200자 초과 시 RES4001
        if req.content.chars().count() > 200 {
            return Err(AppError::CommentTooLong(
                "댓글은 최대 200자까지만 입력 가능합니다.".to_string(),
            ));
        }

        // 2. 답변 조회 및 팀 멤버십 확인
        let _response_model = Self::find_response_for_member(&state, user_id, response_id).await?;

        // 3. 댓글 생성
        let now = Utc::now().naive_utc();
        let comment_model = response_comment::ActiveModel {
            content: Set(req.content.clone()),
            created_at: Set(now),
            updated_at: Set(now),
            response_id: Set(response_id),
            member_id: Set(user_id),
            ..Default::default()
        };

        let inserted = comment_model
            .insert(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // 4. 응답 생성 (KST 시간대 적용)
        let created_at_kst = inserted.created_at + chrono::Duration::hours(9);
        Ok(CreateCommentResponse {
            comment_id: inserted.response_comment_id,
            response_id,
            content: inserted.content,
            created_at: created_at_kst.format("%Y-%m-%dT%H:%M:%S").to_string(),
        })
    }

    /// [API-025] 회고 답변 좋아요 토글
    pub async fn toggle_like(
        state: AppState,
        user_id: i64,
        response_id: i64,
    ) -> Result<super::dto::LikeToggleResponse, AppError> {
        // 1. 답변 존재 확인
        let response_entity = response::Entity::find_by_id(response_id)
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        let response_model = response_entity.ok_or_else(|| {
            AppError::ResponseNotFound("존재하지 않는 회고 답변입니다.".to_string())
        })?;

        // 2. 회고 정보 조회하여 팀 멤버십 확인
        let retrospect_entity = retrospect::Entity::find_by_id(response_model.retrospect_id)
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        let retrospect_model = retrospect_entity.ok_or_else(|| {
            AppError::InternalError(
                "회고 데이터 불일치: 답변에 연결된 회고가 존재하지 않습니다.".to_string(),
            )
        })?;

        // 3. 팀 멤버십 확인
        let is_team_member = member_team::Entity::find()
            .filter(member_team::Column::MemberId.eq(user_id))
            .filter(member_team::Column::TeamId.eq(retrospect_model.team_id))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        if is_team_member.is_none() {
            return Err(AppError::TeamAccessDenied(
                "해당 리소스에 접근 권한이 없습니다.".to_string(),
            ));
        }

        // 4. 기존 좋아요 확인
        let existing_like = response_like::Entity::find()
            .filter(response_like::Column::MemberId.eq(user_id))
            .filter(response_like::Column::ResponseId.eq(response_id))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        let is_liked = match existing_like {
            Some(like) => {
                // 좋아요가 있으면 삭제 (취소)
                response_like::Entity::delete_by_id(like.response_like_id)
                    .exec(&state.db)
                    .await
                    .map_err(|e| AppError::InternalError(e.to_string()))?;
                false
            }
            None => {
                // 좋아요가 없으면 추가
                let new_like = response_like::ActiveModel {
                    member_id: Set(user_id),
                    response_id: Set(response_id),
                    ..Default::default()
                };
                new_like
                    .insert(&state.db)
                    .await
                    .map_err(|e| AppError::InternalError(e.to_string()))?;
                true
            }
        };

        // 5. 총 좋아요 개수 조회
        let total_likes = response_like::Entity::find()
            .filter(response_like::Column::ResponseId.eq(response_id))
            .count(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        Ok(super::dto::LikeToggleResponse {
            response_id,
            is_liked,
            total_likes: total_likes as i64,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== URL 검증 테스트 =====

    #[test]
    fn should_pass_valid_https_url() {
        // Arrange
        let urls = vec!["https://github.com/example".to_string()];

        // Act
        let result = RetrospectService::validate_reference_urls(&urls);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_pass_valid_http_url() {
        // Arrange
        let urls = vec!["http://example.com".to_string()];

        // Act
        let result = RetrospectService::validate_reference_urls(&urls);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_pass_multiple_valid_urls() {
        // Arrange
        let urls = vec![
            "https://github.com/project".to_string(),
            "https://notion.so/page".to_string(),
        ];

        // Act
        let result = RetrospectService::validate_reference_urls(&urls);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_pass_empty_urls() {
        // Arrange
        let urls: Vec<String> = vec![];

        // Act
        let result = RetrospectService::validate_reference_urls(&urls);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_fail_for_duplicate_urls() {
        // Arrange
        let urls = vec![
            "https://github.com/example".to_string(),
            "https://github.com/example".to_string(),
        ];

        // Act
        let result = RetrospectService::validate_reference_urls(&urls);

        // Assert
        assert!(result.is_err());
        if let Err(AppError::RetroUrlInvalid(msg)) = result {
            assert!(msg.contains("중복"));
        } else {
            panic!("Expected RetroUrlInvalid error");
        }
    }

    #[test]
    fn should_fail_for_ftp_url() {
        // Arrange
        let urls = vec!["ftp://example.com".to_string()];

        // Act
        let result = RetrospectService::validate_reference_urls(&urls);

        // Assert
        assert!(result.is_err());
        assert!(matches!(result, Err(AppError::RetroUrlInvalid(_))));
    }

    #[test]
    fn should_fail_for_url_without_scheme() {
        // Arrange
        let urls = vec!["example.com".to_string()];

        // Act
        let result = RetrospectService::validate_reference_urls(&urls);

        // Assert
        assert!(result.is_err());
        assert!(matches!(result, Err(AppError::RetroUrlInvalid(_))));
    }

    #[test]
    fn should_fail_for_url_exceeding_max_length() {
        // Arrange
        let long_url = format!("https://example.com/{}", "a".repeat(2050));
        let urls = vec![long_url];

        // Act
        let result = RetrospectService::validate_reference_urls(&urls);

        // Assert
        assert!(result.is_err());
        if let Err(AppError::RetroUrlInvalid(msg)) = result {
            assert!(msg.contains("2048"));
        } else {
            panic!("Expected RetroUrlInvalid error");
        }
    }

    #[test]
    fn should_fail_for_url_without_host() {
        // Arrange
        let urls = vec!["https://".to_string()];

        // Act
        let result = RetrospectService::validate_reference_urls(&urls);

        // Assert
        assert!(result.is_err());
        assert!(matches!(result, Err(AppError::RetroUrlInvalid(_))));
    }

    // ===== 날짜 형식 검증 테스트 =====

    #[test]
    fn should_pass_valid_date_format() {
        // Arrange
        let valid_date = &Utc::now()
            .date_naive()
            .succ_opt()
            .expect("valid date")
            .format("%Y-%m-%d")
            .to_string();

        // Act
        let result = RetrospectService::validate_and_parse_date(valid_date);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_fail_for_past_date() {
        // Arrange
        let past_date = "2020-01-01";

        // Act
        let result = RetrospectService::validate_and_parse_date(past_date);

        // Assert
        assert!(result.is_err());
        if let Err(AppError::BadRequest(msg)) = result {
            assert!(msg.contains("오늘 이후"));
        } else {
            panic!("Expected BadRequest error");
        }
    }

    #[test]
    fn should_pass_for_today_date() {
        // Arrange
        let today = Utc::now().date_naive().format("%Y-%m-%d").to_string();

        // Act
        let result = RetrospectService::validate_and_parse_date(&today);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_fail_for_invalid_date_format() {
        // Arrange
        let invalid_date = "01-25-2026"; // MM-DD-YYYY format

        // Act
        let result = RetrospectService::validate_and_parse_date(invalid_date);

        // Assert
        assert!(result.is_err());
        if let Err(AppError::BadRequest(msg)) = result {
            assert!(msg.contains("YYYY-MM-DD"));
        } else {
            panic!("Expected BadRequest error");
        }
    }

    #[test]
    fn should_fail_for_invalid_date_string() {
        // Arrange
        let invalid_date = "not-a-date";

        // Act
        let result = RetrospectService::validate_and_parse_date(invalid_date);

        // Assert
        assert!(result.is_err());
        assert!(matches!(result, Err(AppError::BadRequest(_))));
    }

    // ===== 시간 형식 검증 테스트 =====

    #[test]
    fn should_pass_valid_time_format() {
        // Arrange
        let valid_time = "14:30";

        // Act
        let result = RetrospectService::validate_and_parse_time(valid_time);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_pass_midnight_time() {
        // Arrange
        let midnight = "00:00";

        // Act
        let result = RetrospectService::validate_and_parse_time(midnight);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_pass_end_of_day_time() {
        // Arrange
        let end_of_day = "23:59";

        // Act
        let result = RetrospectService::validate_and_parse_time(end_of_day);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_fail_for_invalid_time_format() {
        // Arrange
        let invalid_time = "1430"; // 콜론 없는 형식

        // Act
        let result = RetrospectService::validate_and_parse_time(invalid_time);

        // Assert
        assert!(result.is_err());
        if let Err(AppError::BadRequest(msg)) = result {
            assert!(msg.contains("HH:mm"));
        } else {
            panic!("Expected BadRequest error");
        }
    }

    #[test]
    fn should_fail_for_invalid_time_value() {
        // Arrange
        let invalid_time = "25:00"; // 유효하지 않은 시간

        // Act
        let result = RetrospectService::validate_and_parse_time(invalid_time);

        // Assert
        assert!(result.is_err());
        assert!(matches!(result, Err(AppError::BadRequest(_))));
    }

    // ===== 미래 날짜/시간 검증 테스트 =====

    #[test]
    fn should_pass_future_datetime() {
        // Arrange
        let future_date = Utc::now().date_naive() + chrono::Duration::days(7);
        let time = NaiveTime::from_hms_opt(14, 0, 0).unwrap();

        // Act
        let result = RetrospectService::validate_future_datetime(future_date, time);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_fail_for_past_datetime() {
        // Arrange
        let past_date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let time = NaiveTime::from_hms_opt(14, 0, 0).unwrap();

        // Act
        let result = RetrospectService::validate_future_datetime(past_date, time);

        // Assert
        assert!(result.is_err());
        if let Err(AppError::BadRequest(msg)) = result {
            assert!(msg.contains("미래"));
        } else {
            panic!("Expected BadRequest error");
        }
    }

    // ===== RetrospectMethod 기본 질문 테스트 =====

    #[test]
    fn should_return_5_questions_for_kpt() {
        // Arrange
        use crate::domain::retrospect::entity::retrospect::RetrospectMethod;
        let method = RetrospectMethod::Kpt;

        // Act
        let questions = method.default_questions();

        // Assert
        assert_eq!(questions.len(), 5);
        assert!(questions[0].contains("유지"));
        assert!(questions[1].contains("문제점"));
        assert!(questions[2].contains("시도"));
    }

    #[test]
    fn should_return_5_questions_for_four_l() {
        // Arrange
        use crate::domain::retrospect::entity::retrospect::RetrospectMethod;
        let method = RetrospectMethod::FourL;

        // Act
        let questions = method.default_questions();

        // Assert
        assert_eq!(questions.len(), 5);
        assert!(questions[0].contains("좋았던"));
        assert!(questions[1].contains("배운"));
        assert!(questions[2].contains("부족"));
        assert!(questions[3].contains("바라는"));
    }

    #[test]
    fn should_return_5_questions_for_five_f() {
        // Arrange
        use crate::domain::retrospect::entity::retrospect::RetrospectMethod;
        let method = RetrospectMethod::FiveF;

        // Act
        let questions = method.default_questions();

        // Assert
        assert_eq!(questions.len(), 5);
        assert!(questions[0].contains("사실"));
        assert!(questions[1].contains("감정"));
        assert!(questions[2].contains("발견"));
        assert!(questions[3].contains("적용"));
        assert!(questions[4].contains("피드백"));
    }

    #[test]
    fn should_return_5_questions_for_pmi() {
        // Arrange
        use crate::domain::retrospect::entity::retrospect::RetrospectMethod;
        let method = RetrospectMethod::Pmi;

        // Act
        let questions = method.default_questions();

        // Assert
        assert_eq!(questions.len(), 5);
        assert!(questions[0].contains("긍정"));
        assert!(questions[1].contains("부정"));
        assert!(questions[2].contains("흥미"));
    }

    #[test]
    fn should_return_5_questions_for_free() {
        // Arrange
        use crate::domain::retrospect::entity::retrospect::RetrospectMethod;
        let method = RetrospectMethod::Free;

        // Act
        let questions = method.default_questions();

        // Assert
        assert_eq!(questions.len(), 5);
        assert!(questions[0].contains("기억"));
    }

    // ===== 임시 저장 답변 검증 테스트 (API-016) =====

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
    fn should_fail_when_draft_duplicate_question_numbers() {
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
    fn should_fail_when_draft_content_exceeds_1000_chars() {
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

    // ===== 답변 검증 테스트 (API-017) =====

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

    // ===== 검색 키워드 검증 테스트 (API-023) =====

    #[test]
    fn should_fail_when_keyword_is_none() {
        // Arrange & Act
        let result = RetrospectService::validate_search_keyword(None);

        // Assert
        assert!(result.is_err());
        assert!(matches!(result, Err(AppError::SearchKeywordInvalid(_))));
    }

    #[test]
    fn should_fail_when_keyword_is_empty() {
        // Arrange & Act
        let result = RetrospectService::validate_search_keyword(Some(""));

        // Assert
        assert!(result.is_err());
        assert!(matches!(result, Err(AppError::SearchKeywordInvalid(_))));
    }

    #[test]
    fn should_fail_when_keyword_exceeds_100_chars() {
        // Arrange
        let keyword = "가".repeat(101);

        // Act
        let result = RetrospectService::validate_search_keyword(Some(&keyword));

        // Assert
        assert!(result.is_err());
        if let Err(AppError::SearchKeywordInvalid(msg)) = result {
            assert!(msg.contains("100자"));
        } else {
            panic!("Expected SearchKeywordInvalid error");
        }
    }

    #[test]
    fn should_pass_when_keyword_is_exactly_100_chars() {
        // Arrange
        let keyword = "가".repeat(100);

        // Act
        let result = RetrospectService::validate_search_keyword(Some(&keyword));

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), keyword);
    }

    #[test]
    fn should_fail_when_keyword_is_whitespace_only() {
        // Arrange & Act
        let result = RetrospectService::validate_search_keyword(Some("   \t\n  "));

        // Assert
        assert!(result.is_err());
        assert!(matches!(result, Err(AppError::SearchKeywordInvalid(_))));
    }

    #[test]
    fn should_trim_keyword_with_leading_trailing_whitespace() {
        // Arrange & Act
        let result = RetrospectService::validate_search_keyword(Some("  스프린트  "));

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "스프린트");
    }

    #[test]
    fn should_pass_valid_keyword() {
        // Arrange & Act
        let result = RetrospectService::validate_search_keyword(Some("스프린트 회고"));

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "스프린트 회고");
    }

    // ===== 회고 방식 표시명 테스트 (API-021) =====

    #[test]
    fn should_display_kpt_as_kpt() {
        // Arrange
        use crate::domain::retrospect::entity::retrospect::RetrospectMethod;

        // Act
        let result = RetrospectService::retrospect_method_display(&RetrospectMethod::Kpt);

        // Assert
        assert_eq!(result, "KPT");
    }

    #[test]
    fn should_display_four_l_as_4l() {
        // Arrange
        use crate::domain::retrospect::entity::retrospect::RetrospectMethod;

        // Act
        let result = RetrospectService::retrospect_method_display(&RetrospectMethod::FourL);

        // Assert
        assert_eq!(result, "4L");
    }

    #[test]
    fn should_display_five_f_as_5f() {
        // Arrange
        use crate::domain::retrospect::entity::retrospect::RetrospectMethod;

        // Act
        let result = RetrospectService::retrospect_method_display(&RetrospectMethod::FiveF);

        // Assert
        assert_eq!(result, "5F");
    }

    #[test]
    fn should_display_pmi_as_pmi() {
        // Arrange
        use crate::domain::retrospect::entity::retrospect::RetrospectMethod;

        // Act
        let result = RetrospectService::retrospect_method_display(&RetrospectMethod::Pmi);

        // Assert
        assert_eq!(result, "PMI");
    }

    #[test]
    fn should_display_free_as_free() {
        // Arrange
        use crate::domain::retrospect::entity::retrospect::RetrospectMethod;

        // Act
        let result = RetrospectService::retrospect_method_display(&RetrospectMethod::Free);

        // Assert
        assert_eq!(result, "Free");
    }
}
