use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "member_team")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub member_team_id: i64,
    pub member_id: i64,
    pub team_id: i64,
    pub created_at: DateTime,
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
        belongs_to = "super::team::Entity",
        from = "Column::TeamId",
        to = "super::team::Column::TeamId",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Team,
}

impl Related<crate::domain::member::entity::member::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Member.def()
    }
}

impl Related<super::team::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Team.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
