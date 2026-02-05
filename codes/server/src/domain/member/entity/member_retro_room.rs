use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "RoomRole")]
pub enum RoomRole {
    #[sea_orm(string_value = "OWNER")]
    Owner,
    #[sea_orm(string_value = "MEMBER")]
    Member,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "member_retro_room")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub member_retrospect_room_id: i64,
    pub member_id: Option<i64>,
    pub retrospect_room_id: i64,
    pub role: RoomRole,
    #[sea_orm(default_value = "1")]
    pub order_index: i32,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::member::Entity",
        from = "Column::MemberId",
        to = "super::member::Column::MemberId",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    Member,
    #[sea_orm(
        belongs_to = "crate::domain::retrospect::entity::retro_room::Entity",
        from = "Column::RetrospectRoomId",
        to = "crate::domain::retrospect::entity::retro_room::Column::RetrospectRoomId",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    RetroRoom,
}

impl Related<super::member::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Member.def()
    }
}

impl Related<crate::domain::retrospect::entity::retro_room::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RetroRoom.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
