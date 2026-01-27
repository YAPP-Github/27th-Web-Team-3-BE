use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "response_like")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub response_like_id: i64,
    pub member_id: i64,
    pub response_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::domain::member::entity::member::Entity",
        from = "Column::MemberId",
        to = "crate::domain::member::entity::member::Column::MemberId",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Member,
    #[sea_orm(
        belongs_to = "super::response::Entity",
        from = "Column::ResponseId",
        to = "super::response::Column::ResponseId",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Response,
}

impl Related<crate::domain::member::entity::member::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Member.def()
    }
}

impl Related<super::response::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Response.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
