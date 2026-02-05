use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "retro_room")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub retrospect_room_id: i64,
    #[sea_orm(unique, column_type = "String(StringLen::N(20))")]
    pub title: String,
    #[sea_orm(column_type = "String(StringLen::N(50))", nullable)]
    pub description: Option<String>,
    pub invition_url: String, // Keeping schema spelling
    /// 초대 코드 생성 시각 (만료 기준)
    pub invite_code_created_at: DateTime,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::retrospect::Entity")]
    Retrospect,
    #[sea_orm(has_many = "crate::domain::member::entity::member_retro_room::Entity")]
    MemberRetroRoom,
}

impl Related<super::retrospect::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Retrospect.def()
    }
}

impl Related<crate::domain::member::entity::member_retro_room::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MemberRetroRoom.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
