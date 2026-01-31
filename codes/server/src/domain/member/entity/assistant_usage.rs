use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// 회고 어시스턴트 사용 기록 엔티티
/// 사용자별 월간 사용 횟수를 추적합니다.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "assistant_usage")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub assistant_usage_id: i64,
    /// 사용자 ID
    pub member_id: i64,
    /// 회고 ID
    pub retrospect_id: i64,
    /// 질문 ID (1~5)
    pub question_id: i32,
    /// 사용 일시
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::member::Entity",
        from = "Column::MemberId",
        to = "super::member::Column::MemberId"
    )]
    Member,
    #[sea_orm(
        belongs_to = "crate::domain::retrospect::entity::retrospect::Entity",
        from = "Column::RetrospectId",
        to = "crate::domain::retrospect::entity::retrospect::Column::RetrospectId"
    )]
    Retrospect,
}

impl Related<super::member::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Member.def()
    }
}

impl Related<crate::domain::retrospect::entity::retrospect::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Retrospect.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
