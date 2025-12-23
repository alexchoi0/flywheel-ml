use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use flywheel_ml_db::{entity::pipeline, Database};
use flywheel_ml_dsl::{FlywheelPipelineManifest, FlywheelStageType};

use super::stage::{StageExecutor, StageContext};

pub struct PipelineRunner {
    pipeline: pipeline::Model,
    manifest: FlywheelPipelineManifest,
    db: Database,
    running: AtomicBool,
    records_processed: AtomicU64,
    predictions_made: AtomicU64,
    errors: AtomicU64,
}

impl PipelineRunner {
    pub fn new(pipeline: pipeline::Model, db: Database) -> anyhow::Result<Self> {
        let manifest = flywheel_ml_dsl::parser::parse_manifest(&pipeline.spec_yaml)
            .map_err(|e| anyhow::anyhow!("Failed to parse pipeline spec: {}", e))?;

        Ok(Self {
            pipeline,
            manifest,
            db,
            running: AtomicBool::new(true),
            records_processed: AtomicU64::new(0),
            predictions_made: AtomicU64::new(0),
            errors: AtomicU64::new(0),
        })
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub async fn run(&self) {
        tracing::info!(
            pipeline_id = %self.pipeline.id,
            name = %self.pipeline.name,
            stages = self.manifest.spec.stages.len(),
            "Pipeline runner started"
        );

        let ctx = StageContext {
            pipeline_id: self.pipeline.id,
            pipeline_name: self.pipeline.name.clone(),
            namespace: self.pipeline.namespace.clone(),
            db: self.db.clone(),
        };

        while self.is_running() {
            match self.execute_cycle(&ctx).await {
                Ok(processed) => {
                    self.records_processed.fetch_add(processed, Ordering::Relaxed);
                }
                Err(e) => {
                    self.errors.fetch_add(1, Ordering::Relaxed);
                    tracing::error!(
                        pipeline_id = %self.pipeline.id,
                        error = %e,
                        "Pipeline execution cycle failed"
                    );
                }
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        tracing::info!(
            pipeline_id = %self.pipeline.id,
            records_processed = self.records_processed.load(Ordering::Relaxed),
            predictions_made = self.predictions_made.load(Ordering::Relaxed),
            errors = self.errors.load(Ordering::Relaxed),
            "Pipeline runner stopped"
        );
    }

    async fn execute_cycle(&self, ctx: &StageContext) -> anyhow::Result<u64> {
        let mut records_in_cycle = 0u64;

        for stage in &self.manifest.spec.stages {
            let executor = StageExecutor::for_stage(stage, ctx)?;

            match executor.execute().await {
                Ok(result) => {
                    records_in_cycle += result.records_processed;

                    if stage.stage_type == FlywheelStageType::MlInference {
                        self.predictions_made.fetch_add(result.records_processed, Ordering::Relaxed);
                    }

                    tracing::debug!(
                        pipeline_id = %self.pipeline.id,
                        stage_id = %stage.id,
                        stage_type = ?stage.stage_type,
                        records = result.records_processed,
                        "Stage executed"
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        pipeline_id = %self.pipeline.id,
                        stage_id = %stage.id,
                        error = %e,
                        "Stage execution failed"
                    );
                    return Err(e);
                }
            }
        }

        Ok(records_in_cycle)
    }

    pub fn stats(&self) -> PipelineStats {
        PipelineStats {
            records_processed: self.records_processed.load(Ordering::Relaxed),
            predictions_made: self.predictions_made.load(Ordering::Relaxed),
            errors: self.errors.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PipelineStats {
    pub records_processed: u64,
    pub predictions_made: u64,
    pub errors: u64,
}
