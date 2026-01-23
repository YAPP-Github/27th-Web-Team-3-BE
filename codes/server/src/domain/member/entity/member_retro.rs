use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "member_retro")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub member_retro_id: i64,
    pub personal_insight: Option<String>,
    pub member_id: i64,
    pub retrospect_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::member::Entity",
        from = "Column::MemberId",
        to = "super::member::Column::MemberId",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Member,
    #[sea_orm(
        belongs_to = "crate::domain::retrospect::entity::retrospect::Entity",
        from = "Column::RetrospectId",
        to = "crate::domain::retrospect::entity::retrospect::Column::RetrospectId",
        on_update = "NoAction",
        on_delete = "NoAction"
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
