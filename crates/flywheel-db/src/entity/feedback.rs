use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(32))")]
pub enum FeedbackSource {
    #[sea_orm(string_value = "explicit")]
    Explicit,
    #[sea_orm(string_value = "implicit")]
    Implicit,
    #[sea_orm(string_value = "automated")]
    Automated,
    #[sea_orm(string_value = "manual")]
    Manual,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "feedback")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub prediction_id: Uuid,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub ground_truth: String,
    pub source: FeedbackSource,
    pub confidence: f64,
    pub received_at: DateTimeUtc,
    pub exported: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::prediction::Entity")]
    Prediction,
}

impl Related<super::prediction::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Prediction.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
