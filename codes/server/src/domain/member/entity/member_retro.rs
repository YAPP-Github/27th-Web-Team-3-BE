use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 회고 참여 상태 Enum
#[derive(
    Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, ToSchema,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "RetrospectStatus")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RetrospectStatus {
    /// 임시 저장 상태
    #[sea_orm(string_value = "DRAFT")]
    Draft,
    /// 최종 제출 완료
    #[sea_orm(string_value = "SUBMITTED")]
    Submitted,
    /// AI 분석 완료
    #[sea_orm(string_value = "ANALYZED")]
    Analyzed,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "member_retro")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub member_retro_id: i64,
    pub personal_insight: Option<String>,
    pub member_id: i64,
    pub retrospect_id: i64,
    pub status: RetrospectStatus,
    pub submitted_at: Option<DateTime>,
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
        belongs_to = "crate::domain::retrospect::entity::retrospect::Entity",
        from = "Column::RetrospectId",
        to = "crate::domain::retrospect::entity::retrospect::Column::RetrospectId",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Retrospect,
}

impl Related<super::member::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Member.def()
    }
}

impl Related<crate::domain::retrospect::entity::retrospect::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Retrospect.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
