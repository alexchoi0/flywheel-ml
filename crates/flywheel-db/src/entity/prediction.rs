use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "predictions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub pipeline_id: Uuid,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub model_id: String,
    #[sea_orm(column_type = "String(StringLen::N(64))")]
    pub model_version: String,
    pub features_json: Json,
    pub prediction_json: Json,
    pub created_at: DateTimeUtc,
    pub feedback_id: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::pipeline::Entity",
        from = "Column::PipelineId",
        to = "super::pipeline::Column::Id"
    )]
    Pipeline,
    #[sea_orm(
        belongs_to = "super::feedback::Entity",
        from = "Column::FeedbackId",
        to = "super::feedback::Column::Id"
    )]
    Feedback,
}

impl Related<super::pipeline::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Pipeline.def()
    }
}

impl Related<super::feedback::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Feedback.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
