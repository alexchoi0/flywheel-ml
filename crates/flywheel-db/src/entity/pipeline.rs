use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(32))")]
pub enum PipelineStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "running")]
    Running,
    #[sea_orm(string_value = "stopped")]
    Stopped,
    #[sea_orm(string_value = "failed")]
    Failed,
    #[sea_orm(string_value = "disabled")]
    Disabled,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "pipelines")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub name: String,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub namespace: String,
    #[sea_orm(column_type = "String(StringLen::N(64))")]
    pub spec_hash: String,
    #[sea_orm(column_type = "Text")]
    pub spec_yaml: String,
    pub status: PipelineStatus,
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
    pub conveyor_pipeline_id: Option<String>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::pipeline_run::Entity")]
    PipelineRuns,
    #[sea_orm(has_many = "super::prediction::Entity")]
    Predictions,
    #[sea_orm(has_many = "super::drift_event::Entity")]
    DriftEvents,
}

impl Related<super::pipeline_run::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PipelineRuns.def()
    }
}

impl Related<super::prediction::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Predictions.def()
    }
}

impl Related<super::drift_event::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DriftEvents.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
