use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
    Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, ToSchema,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "SocialType")]
pub enum SocialType {
    #[sea_orm(string_value = "KAKAO")]
    Kakao,
    #[sea_orm(string_value = "GOOGLE")]
    Google,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "member")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub member_id: i64,
    pub email: String,
    pub nickname: Option<String>,
    pub insight_count: i32,
    #[sea_orm(column_name = "SocialType")]
    pub social_type: SocialType,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "crate::domain::retrospect::entity::response_comment::Entity")]
    ResponseComment,
    #[sea_orm(has_many = "crate::domain::retrospect::entity::response_like::Entity")]
    ResponseLike,
    #[sea_orm(has_many = "super::member_response::Entity")]
    MemberResponse,
    #[sea_orm(has_many = "super::member_retro::Entity")]
    MemberRetro,
    #[sea_orm(has_many = "super::member_retro_room::Entity")]
    MemberRetroRoom,
}

impl Related<crate::domain::retrospect::entity::response_comment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ResponseComment.def()
    }
}

impl Related<crate::domain::retrospect::entity::response_like::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ResponseLike.def()
    }
}

impl Related<super::member_response::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MemberResponse.def()
    }
}

impl Related<super::member_retro::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MemberRetro.def()
    }
}

impl Related<super::member_retro_room::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MemberRetroRoom.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
