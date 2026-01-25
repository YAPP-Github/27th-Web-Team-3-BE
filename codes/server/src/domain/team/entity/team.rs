use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "team")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub team_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::member_team::Entity")]
    MemberTeam,
    #[sea_orm(has_many = "crate::domain::retrospect::entity::retrospect::Entity")]
    Retrospect,
}

impl Related<super::member_team::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MemberTeam.def()
    }
}

impl Related<crate::domain::retrospect::entity::retrospect::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Retrospect.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
