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
                "이번 일을 통해 유지했으면 하는 문화나 방식이 있나요?",
                "이번 일을 하는 중 문제라고 판단되었던 점이 있나요?",
                "이번 일을 겪으면서 새롭게 시도해보고 싶은 게 있나요?",
            ],
            RetrospectMethod::FourL => vec![
                "이번 일을 하면서 기억에 남는 좋은 순간이 있었나요?",
                "이번 일을 통해 새롭게 알게 되거나 성장한 부분이 있나요?",
                "이번 일을 하면서 아쉬웠거나 더 필요했던 게 있나요?",
                "앞으로 일할 때 이런 부분이 개선되면 좋겠다고 생각한 게 있나요?",
            ],
            RetrospectMethod::FiveF => vec![
                "이번 업무를 통해 새롭게 알게 된 사실이 있나요?",
                "업무 중 가장 힘들었던 순간과 가장 뿌듯했던 순간은 언제였나요?",
                "업무를 진행하면서 예상하지 못했던 발견이 있었나요?",
                "비슷한 업무를 다시 한다면 어떤 점을 다르게 하고 싶나요?",
                "함께 업무를 진행한 분들에게 하고 싶은 이야기가 있나요?",
            ],
            RetrospectMethod::Pmi => vec![
                "이번 일을 통해 도움이 되었던 문화나 방법은 무엇인가요?",
                "이번 일을 통해 안 좋은 영향을 끼쳤던 것은 무엇인가요?",
                "이번 일을 하면서 새롭게 발견한 점은 무엇인가요?",
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

    /// 회고 방식별 질문 개수를 반환합니다.
    pub fn question_count(&self) -> usize {
        self.default_questions().len()
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "retrospects")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub retrospect_id: i64,
    pub title: String,
    pub insight: Option<String>,
    pub retrospect_method: RetrospectMethod,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub start_time: DateTime,
    pub retrospect_room_id: i64,
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
