use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "response")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub response_id: i64,
    pub question: String,
    #[sea_orm(column_name = "response")]
    pub content: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub retrospect_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::retrospect::Entity",
        from = "Column::RetrospectId",
        to = "super::retrospect::Column::RetrospectId",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Retrospect,
    #[sea_orm(has_many = "super::response_comment::Entity")]
    ResponseComment,
    #[sea_orm(has_many = "super::response_like::Entity")]
    ResponseLike,
    #[sea_orm(has_many = "crate::domain::member::entity::member_response::Entity")]
    MemberResponse,
}

impl Related<super::retrospect::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Retrospect.def()
    }
}

impl Related<super::response_comment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ResponseComment.def()
    }
}

impl Related<super::response_like::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ResponseLike.def()
    }
}

impl Related<crate::domain::member::entity::member_response::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MemberResponse.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
