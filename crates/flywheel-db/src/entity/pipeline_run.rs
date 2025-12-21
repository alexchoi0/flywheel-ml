use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(32))")]
pub enum RunStatus {
    #[sea_orm(string_value = "running")]
    Running,
    #[sea_orm(string_value = "completed")]
    Completed,
    #[sea_orm(string_value = "failed")]
    Failed,
    #[sea_orm(string_value = "cancelled")]
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "pipeline_runs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub pipeline_id: Uuid,
    pub status: RunStatus,
    pub started_at: DateTimeUtc,
    pub ended_at: Option<DateTimeUtc>,
    pub records_processed: i64,
    pub records_failed: i64,
    #[sea_orm(column_type = "Text", nullable)]
    pub error_message: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::pipeline::Entity",
        from = "Column::PipelineId",
        to = "super::pipeline::Column::Id"
    )]
    Pipeline,
}

impl Related<super::pipeline::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Pipeline.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
