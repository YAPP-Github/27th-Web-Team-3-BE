use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
    TransactionTrait,
};
use std::collections::HashSet;

use crate::domain::member::entity::member;
use crate::domain::member::entity::member_retro;
use crate::domain::retrospect::entity::response;
use crate::domain::retrospect::entity::response_comment;
use crate::domain::retrospect::entity::retro_reference;
use crate::domain::retrospect::entity::retro_room;
use crate::domain::retrospect::entity::retrospect;
use crate::domain::team::entity::member_team;
use crate::domain::team::entity::team;
use crate::state::AppState;
use crate::utils::error::AppError;

use super::dto::{
    CommentItem, CreateCommentRequest, CreateCommentResponse, CreateParticipantResponse,
    CreateRetrospectRequest, CreateRetrospectResponse, ListCommentsResponse, ReferenceItem,
    TeamRetrospectListItem, REFERENCE_URL_MAX_LENGTH,
};

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
            invition_url: Set(invitation_url),
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
            .order_by_asc(retro_reference::Column::RetroRefrenceId)
            .all(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // 3. DTO 변환
        let result: Vec<ReferenceItem> = references
            .into_iter()
            .map(|r| ReferenceItem {
                reference_id: r.retro_refrence_id,
                url_name: r.title,
                url: r.url,
            })
            .collect();

        Ok(result)
    }

    /// 회고 답변 조회 및 팀 멤버십 확인 헬퍼
    /// 비멤버에게 답변 존재 여부를 노출하지 않도록
    /// "존재하지 않음"과 "접근 권한 없음"을 동일한 404로 처리
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
                AppError::ResponseNotFound("존재하지 않는 회고 답변입니다.".to_string())
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
        // 1. 답변 조회 및 팀 멤버십 확인
        let _response_model = Self::find_response_for_member(&state, user_id, response_id).await?;

        // 2. 댓글 목록 조회 (커서 기반 페이지네이션, 최신순 정렬)
        // size + 1개를 조회하여 다음 페이지 존재 여부 확인
        let mut query = response_comment::Entity::find()
            .filter(response_comment::Column::ResponseId.eq(response_id));

        // 커서가 있으면 해당 커서 이전의 댓글만 조회 (commentId 내림차순이므로 < 사용)
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
        let member_map: std::collections::HashMap<i64, String> = members
            .into_iter()
            .map(|m| {
                let nickname = m.email.split('@').next().unwrap_or(&m.email).to_string();
                (m.member_id, nickname)
            })
            .collect();

        // 5. DTO 변환
        let comment_items: Vec<CommentItem> = comments
            .iter()
            .map(|c| CommentItem {
                comment_id: c.response_comment_id,
                member_id: c.member_id,
                user_name: member_map
                    .get(&c.member_id)
                    .cloned()
                    .unwrap_or_else(|| "Unknown".to_string()),
                content: c.content.clone(),
                created_at: c.created_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
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
        // 1. 댓글 길이 검증 (200자 초과 시 RES4001)
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

        // 4. 응답 생성
        Ok(CreateCommentResponse {
            comment_id: inserted.response_comment_id,
            response_id,
            content: inserted.content,
            created_at: inserted.created_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
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
}
