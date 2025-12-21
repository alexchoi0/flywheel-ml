use sea_orm::*;
use uuid::Uuid;

use crate::entity::{pipeline, pipeline_run, model_version, drift_event, prediction, feedback};

pub struct PipelineRepo;

impl PipelineRepo {
    pub async fn create(
        db: &DatabaseConnection,
        name: String,
        namespace: String,
        spec_hash: String,
        spec_yaml: String,
    ) -> Result<pipeline::Model, DbErr> {
        let model = pipeline::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set(name),
            namespace: Set(namespace),
            spec_hash: Set(spec_hash),
            spec_yaml: Set(spec_yaml),
            status: Set(pipeline::PipelineStatus::Pending),
            conveyor_pipeline_id: Set(None),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
        };
        model.insert(db).await
    }

    pub async fn find_by_id(db: &DatabaseConnection, id: Uuid) -> Result<Option<pipeline::Model>, DbErr> {
        pipeline::Entity::find_by_id(id).one(db).await
    }

    pub async fn find_by_name(
        db: &DatabaseConnection,
        name: &str,
        namespace: &str,
    ) -> Result<Option<pipeline::Model>, DbErr> {
        pipeline::Entity::find()
            .filter(pipeline::Column::Name.eq(name))
            .filter(pipeline::Column::Namespace.eq(namespace))
            .one(db)
            .await
    }

    pub async fn list(
        db: &DatabaseConnection,
        namespace: Option<&str>,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<pipeline::Model>, DbErr> {
        let mut query = pipeline::Entity::find();
        if let Some(ns) = namespace {
            query = query.filter(pipeline::Column::Namespace.eq(ns));
        }
        query
            .order_by_desc(pipeline::Column::CreatedAt)
            .limit(limit)
            .offset(offset)
            .all(db)
            .await
    }

    pub async fn update_status(
        db: &DatabaseConnection,
        id: Uuid,
        status: pipeline::PipelineStatus,
    ) -> Result<pipeline::Model, DbErr> {
        let model = pipeline::ActiveModel {
            id: Set(id),
            status: Set(status),
            updated_at: Set(chrono::Utc::now()),
            ..Default::default()
        };
        model.update(db).await
    }

    pub async fn delete(db: &DatabaseConnection, id: Uuid) -> Result<DeleteResult, DbErr> {
        pipeline::Entity::delete_by_id(id).exec(db).await
    }
}

pub struct ModelVersionRepo;

impl ModelVersionRepo {
    pub async fn create(
        db: &DatabaseConnection,
        model_id: String,
        version: String,
        model_type: String,
        endpoint: String,
    ) -> Result<model_version::Model, DbErr> {
        let model = model_version::ActiveModel {
            id: Set(Uuid::new_v4()),
            model_id: Set(model_id),
            version: Set(version),
            model_type: Set(model_type),
            endpoint: Set(endpoint),
            status: Set(model_version::ModelStatus::Pending),
            accuracy: Set(None),
            latency_p99_ms: Set(None),
            deployed_at: Set(chrono::Utc::now()),
        };
        model.insert(db).await
    }

    pub async fn find_by_model_id(
        db: &DatabaseConnection,
        model_id: &str,
    ) -> Result<Option<model_version::Model>, DbErr> {
        model_version::Entity::find()
            .filter(model_version::Column::ModelId.eq(model_id))
            .filter(model_version::Column::Status.eq(model_version::ModelStatus::Active))
            .order_by_desc(model_version::Column::DeployedAt)
            .one(db)
            .await
    }

    pub async fn list(db: &DatabaseConnection, limit: u64) -> Result<Vec<model_version::Model>, DbErr> {
        model_version::Entity::find()
            .order_by_desc(model_version::Column::DeployedAt)
            .limit(limit)
            .all(db)
            .await
    }

    pub async fn update_metrics(
        db: &DatabaseConnection,
        id: Uuid,
        accuracy: Option<f64>,
        latency_p99_ms: Option<i64>,
    ) -> Result<model_version::Model, DbErr> {
        let model = model_version::ActiveModel {
            id: Set(id),
            accuracy: Set(accuracy),
            latency_p99_ms: Set(latency_p99_ms),
            ..Default::default()
        };
        model.update(db).await
    }
}

pub struct DriftEventRepo;

impl DriftEventRepo {
    pub async fn create(
        db: &DatabaseConnection,
        pipeline_id: Uuid,
        model_id: String,
        drift_type: drift_event::DriftType,
        severity: drift_event::DriftSeverity,
        psi_score: Option<f64>,
        kl_divergence: Option<f64>,
        accuracy_delta: Option<f64>,
    ) -> Result<drift_event::Model, DbErr> {
        let model = drift_event::ActiveModel {
            id: Set(Uuid::new_v4()),
            pipeline_id: Set(pipeline_id),
            model_id: Set(model_id),
            drift_type: Set(drift_type),
            severity: Set(severity),
            psi_score: Set(psi_score),
            kl_divergence: Set(kl_divergence),
            accuracy_delta: Set(accuracy_delta),
            detected_at: Set(chrono::Utc::now()),
            resolved_at: Set(None),
        };
        model.insert(db).await
    }

    pub async fn list_by_pipeline(
        db: &DatabaseConnection,
        pipeline_id: Uuid,
        limit: u64,
    ) -> Result<Vec<drift_event::Model>, DbErr> {
        drift_event::Entity::find()
            .filter(drift_event::Column::PipelineId.eq(pipeline_id))
            .order_by_desc(drift_event::Column::DetectedAt)
            .limit(limit)
            .all(db)
            .await
    }

    pub async fn resolve(db: &DatabaseConnection, id: Uuid) -> Result<drift_event::Model, DbErr> {
        let model = drift_event::ActiveModel {
            id: Set(id),
            resolved_at: Set(Some(chrono::Utc::now())),
            ..Default::default()
        };
        model.update(db).await
    }
}

pub struct PredictionRepo;

impl PredictionRepo {
    pub async fn create(
        db: &DatabaseConnection,
        pipeline_id: Uuid,
        model_id: String,
        model_version: String,
        features_json: serde_json::Value,
        prediction_json: serde_json::Value,
    ) -> Result<prediction::Model, DbErr> {
        let model = prediction::ActiveModel {
            id: Set(Uuid::new_v4()),
            pipeline_id: Set(pipeline_id),
            model_id: Set(model_id),
            model_version: Set(model_version),
            features_json: Set(features_json),
            prediction_json: Set(prediction_json),
            created_at: Set(chrono::Utc::now()),
            feedback_id: Set(None),
        };
        model.insert(db).await
    }

    pub async fn find_by_id(db: &DatabaseConnection, id: Uuid) -> Result<Option<prediction::Model>, DbErr> {
        prediction::Entity::find_by_id(id).one(db).await
    }

    pub async fn link_feedback(
        db: &DatabaseConnection,
        prediction_id: Uuid,
        feedback_id: Uuid,
    ) -> Result<prediction::Model, DbErr> {
        let model = prediction::ActiveModel {
            id: Set(prediction_id),
            feedback_id: Set(Some(feedback_id)),
            ..Default::default()
        };
        model.update(db).await
    }
}

pub struct FeedbackRepo;

impl FeedbackRepo {
    pub async fn create(
        db: &DatabaseConnection,
        prediction_id: Uuid,
        ground_truth: String,
        source: feedback::FeedbackSource,
        confidence: f64,
    ) -> Result<feedback::Model, DbErr> {
        let model = feedback::ActiveModel {
            id: Set(Uuid::new_v4()),
            prediction_id: Set(prediction_id),
            ground_truth: Set(ground_truth),
            source: Set(source),
            confidence: Set(confidence),
            received_at: Set(chrono::Utc::now()),
            exported: Set(false),
        };
        model.insert(db).await
    }

    pub async fn mark_exported(db: &DatabaseConnection, id: Uuid) -> Result<feedback::Model, DbErr> {
        let model = feedback::ActiveModel {
            id: Set(id),
            exported: Set(true),
            ..Default::default()
        };
        model.update(db).await
    }

    pub async fn list_unexported(
        db: &DatabaseConnection,
        limit: u64,
    ) -> Result<Vec<feedback::Model>, DbErr> {
        feedback::Entity::find()
            .filter(feedback::Column::Exported.eq(false))
            .order_by_asc(feedback::Column::ReceivedAt)
            .limit(limit)
            .all(db)
            .await
    }
}
