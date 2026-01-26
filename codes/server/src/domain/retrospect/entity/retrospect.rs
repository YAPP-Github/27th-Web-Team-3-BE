use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
    Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, ToSchema,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "RetroCategory")]
pub enum RetroCategory {
    /// Keep-Problem-Try
    #[sea_orm(string_value = "KPT")]
    Kpt,
    /// 4L (Liked-Learned-Lacked-Longed for)
    #[sea_orm(string_value = "FOUR_L")]
    FourL,
    /// 5F (Facts-Feelings-Findings-Future-Feedback)
    #[sea_orm(string_value = "FIVE_F")]
    FiveF,
    /// Plus-Minus-Interesting
    #[sea_orm(string_value = "PMI")]
    Pmi,
    /// Free format
    #[sea_orm(string_value = "FREE")]
    Free,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "retrospects")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub retrospect_id: i64,
    pub title: String,
    pub team_insight: Option<String>,
    pub retro_category: RetroCategory,
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
        on_delete = "NoAction"
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
