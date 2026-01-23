use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "member_retro_room")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub member_retrospect_room_id: i64,
    pub member_id: i64,
    pub retrospect_room_id: i64,
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
        belongs_to = "crate::domain::retrospect::entity::retro_room::Entity",
        from = "Column::RetrospectRoomId",
        to = "crate::domain::retrospect::entity::retro_room::Column::RetrospectRoomId",
        on_update = "NoAction",
        on_delete = "NoAction"
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
