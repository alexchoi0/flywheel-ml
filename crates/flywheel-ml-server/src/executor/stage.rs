use flywheel_ml_db::Database;
use flywheel_ml_dsl::{FlywheelStage, FlywheelStageType};
use uuid::Uuid;

pub struct StageContext {
    pub pipeline_id: Uuid,
    pub pipeline_name: String,
    pub namespace: String,
    pub db: Database,
}

pub struct StageExecutor {
    stage: FlywheelStage,
    ctx: StageContext,
}

pub struct StageResult {
    pub records_processed: u64,
    pub records_failed: u64,
}

impl StageExecutor {
    pub fn for_stage(stage: &FlywheelStage, ctx: &StageContext) -> anyhow::Result<Self> {
        Ok(Self {
            stage: stage.clone(),
            ctx: StageContext {
                pipeline_id: ctx.pipeline_id,
                pipeline_name: ctx.pipeline_name.clone(),
                namespace: ctx.namespace.clone(),
                db: ctx.db.clone(),
            },
        })
    }

    pub async fn execute(&self) -> anyhow::Result<StageResult> {
        match self.stage.stage_type {
            FlywheelStageType::FeatureExtraction => self.execute_feature_extraction().await,
            FlywheelStageType::MlInference => self.execute_inference().await,
            FlywheelStageType::DriftDetection => self.execute_drift_detection().await,
            FlywheelStageType::FeedbackJoin => self.execute_feedback_join().await,
            FlywheelStageType::TrainingExport => self.execute_training_export().await,
        }
    }

    async fn execute_feature_extraction(&self) -> anyhow::Result<StageResult> {
        tracing::trace!(
            stage_id = %self.stage.id,
            "Executing feature extraction"
        );

        Ok(StageResult {
            records_processed: 0,
            records_failed: 0,
        })
    }

    async fn execute_inference(&self) -> anyhow::Result<StageResult> {
        tracing::trace!(
            stage_id = %self.stage.id,
            "Executing ML inference"
        );

        Ok(StageResult {
            records_processed: 0,
            records_failed: 0,
        })
    }

    async fn execute_drift_detection(&self) -> anyhow::Result<StageResult> {
        tracing::trace!(
            stage_id = %self.stage.id,
            "Executing drift detection"
        );

        Ok(StageResult {
            records_processed: 0,
            records_failed: 0,
        })
    }

    async fn execute_feedback_join(&self) -> anyhow::Result<StageResult> {
        tracing::trace!(
            stage_id = %self.stage.id,
            "Executing feedback join"
        );

        Ok(StageResult {
            records_processed: 0,
            records_failed: 0,
        })
    }

    async fn execute_training_export(&self) -> anyhow::Result<StageResult> {
        tracing::trace!(
            stage_id = %self.stage.id,
            "Executing training export"
        );

        Ok(StageResult {
            records_processed: 0,
            records_failed: 0,
        })
    }
}
