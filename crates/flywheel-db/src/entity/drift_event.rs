use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(32))")]
pub enum DriftType {
    #[sea_orm(string_value = "statistical")]
    Statistical,
    #[sea_orm(string_value = "performance")]
    Performance,
    #[sea_orm(string_value = "both")]
    Both,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(32))")]
pub enum DriftSeverity {
    #[sea_orm(string_value = "low")]
    Low,
    #[sea_orm(string_value = "medium")]
    Medium,
    #[sea_orm(string_value = "high")]
    High,
    #[sea_orm(string_value = "critical")]
    Critical,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "drift_events")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub pipeline_id: Uuid,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub model_id: String,
    pub drift_type: DriftType,
    pub severity: DriftSeverity,
    pub psi_score: Option<f64>,
    pub kl_divergence: Option<f64>,
    pub accuracy_delta: Option<f64>,
    pub detected_at: DateTimeUtc,
    pub resolved_at: Option<DateTimeUtc>,
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
