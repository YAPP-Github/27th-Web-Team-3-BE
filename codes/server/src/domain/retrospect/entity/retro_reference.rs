use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "retro_reference")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub retro_reference_id: i64,
    pub title: String,
    pub url: String,
    pub retrospect_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::retrospect::Entity",
        from = "Column::RetrospectId",
        to = "super::retrospect::Column::RetrospectId",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Retrospect,
}

impl Related<super::retrospect::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Retrospect.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
