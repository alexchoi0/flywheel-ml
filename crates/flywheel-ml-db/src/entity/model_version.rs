use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(32))")]
pub enum ModelStatus {
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "deprecated")]
    Deprecated,
    #[sea_orm(string_value = "failed")]
    Failed,
    #[sea_orm(string_value = "pending")]
    Pending,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "model_versions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub model_id: String,
    #[sea_orm(column_type = "String(StringLen::N(64))")]
    pub version: String,
    #[sea_orm(column_type = "String(StringLen::N(64))")]
    pub model_type: String,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub endpoint: String,
    pub status: ModelStatus,
    pub accuracy: Option<f64>,
    pub latency_p99_ms: Option<i64>,
    pub deployed_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
