use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 회고 방식 Enum
/// API 스펙에 따라 5가지 회고 방식을 지원합니다.
#[derive(
    Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, ToSchema,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "RetrospectMethod")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RetrospectMethod {
    /// Keep-Problem-Try: 유지할 점, 문제점, 시도할 점을 정리하는 방식
    #[sea_orm(string_value = "KPT")]
    Kpt,
    /// 4L: Liked-Learned-Lacked-Longed for 방식
    #[sea_orm(string_value = "FOUR_L")]
    FourL,
    /// 5F: Facts-Feelings-Findings-Future-Feedback 방식
    #[sea_orm(string_value = "FIVE_F")]
    FiveF,
    /// Plus-Minus-Interesting: 긍정-부정-흥미로운 점을 분류하는 방식
    #[sea_orm(string_value = "PMI")]
    Pmi,
    /// 자유 형식: 형식 제약 없이 자유롭게 작성
    #[sea_orm(string_value = "FREE")]
    Free,
}

impl std::fmt::Display for RetrospectMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            RetrospectMethod::Kpt => "KPT",
            RetrospectMethod::FourL => "FOUR_L",
            RetrospectMethod::FiveF => "FIVE_F",
            RetrospectMethod::Pmi => "PMI",
            RetrospectMethod::Free => "FREE",
        };
        write!(f, "{}", s)
    }
}

impl RetrospectMethod {
    /// 회고 방식에 따른 기본 질문 목록을 반환합니다.
    pub fn default_questions(&self) -> Vec<&'static str> {
        match self {
            RetrospectMethod::Kpt => vec![
                "계속 유지하고 싶은 좋은 점은 무엇인가요?",
                "개선이 필요한 문제점은 무엇인가요?",
                "다음에 시도해보고 싶은 것은 무엇인가요?",
                "전체 프로젝트를 돌아보며 느낀 점을 자유롭게 작성해주세요.",
                "추가로 공유하고 싶은 의견이 있나요?",
            ],
            RetrospectMethod::FourL => vec![
                "프로젝트에서 좋았던 점은 무엇인가요?",
                "새롭게 배운 것은 무엇인가요?",
                "부족했던 점은 무엇인가요?",
                "앞으로 바라는 것은 무엇인가요?",
                "추가로 공유하고 싶은 의견이 있나요?",
            ],
            RetrospectMethod::FiveF => vec![
                "프로젝트의 객관적 사실은 무엇인가요?",
                "프로젝트 진행 중 어떤 감정을 느꼈나요?",
                "프로젝트에서 발견한 것은 무엇인가요?",
                "앞으로 어떻게 적용할 수 있을까요?",
                "팀원에게 전하고 싶은 피드백이 있나요?",
            ],
            RetrospectMethod::Pmi => vec![
                "긍정적이었던 점은 무엇인가요?",
                "부정적이었던 점은 무엇인가요?",
                "흥미로웠던 점은 무엇인가요?",
                "전체 프로젝트를 돌아보며 느낀 점을 자유롭게 작성해주세요.",
                "추가로 공유하고 싶은 의견이 있나요?",
            ],
            RetrospectMethod::Free => vec![
                "이번 프로젝트에서 가장 기억에 남는 것은 무엇인가요?",
                "프로젝트를 진행하며 어떤 생각이 들었나요?",
                "다음 프로젝트에서 개선하고 싶은 점은 무엇인가요?",
                "팀원들에게 전하고 싶은 말이 있나요?",
                "추가로 공유하고 싶은 의견이 있나요?",
            ],
        }
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "retrospects")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub retrospect_id: i64,
    pub title: String,
    pub team_insight: Option<String>,
    pub retrospect_method: RetrospectMethod,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub start_time: DateTime,
    pub retrospect_room_id: i64,
    pub team_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::retro_room::Entity",
        from = "Column::RetrospectRoomId",
        to = "super::retro_room::Column::RetrospectRoomId",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    RetroRoom,
    #[sea_orm(has_many = "super::response::Entity")]
    Response,
    #[sea_orm(has_many = "crate::domain::member::entity::member_retro::Entity")]
    MemberRetro,
    #[sea_orm(has_many = "super::retro_reference::Entity")]
    RetroReference,
}

impl Related<super::retro_room::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RetroRoom.def()
    }
}

impl Related<super::response::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Response.def()
    }
}

impl Related<crate::domain::member::entity::member_retro::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MemberRetro.def()
    }
}

impl Related<super::retro_reference::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RetroReference.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
