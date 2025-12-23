use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use flywheel_ml_db::{entity::pipeline::PipelineStatus, Database, PipelineRepo};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use uuid::Uuid;

use super::PipelineRunner;

pub struct ExecutionEngine {
    db: Database,
    runners: Arc<RwLock<HashMap<Uuid, RunnerHandle>>>,
    poll_interval: Duration,
}

struct RunnerHandle {
    runner: Arc<PipelineRunner>,
    task: JoinHandle<()>,
}

impl ExecutionEngine {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            runners: Arc::new(RwLock::new(HashMap::new())),
            poll_interval: Duration::from_secs(5),
        }
    }

    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    pub async fn start(self: Arc<Self>) {
        tracing::info!("Starting execution engine");

        loop {
            if let Err(e) = self.reconcile().await {
                tracing::error!(error = %e, "Reconciliation failed");
            }

            tokio::time::sleep(self.poll_interval).await;
        }
    }

    async fn reconcile(&self) -> anyhow::Result<()> {
        let running_pipelines = PipelineRepo::list_by_status(
            self.db.conn(),
            PipelineStatus::Running,
            100,
        )
        .await?;

        let running_ids: std::collections::HashSet<Uuid> =
            running_pipelines.iter().map(|p| p.id).collect();

        let mut runners = self.runners.write().await;
        let current_ids: Vec<Uuid> = runners.keys().cloned().collect();
        for id in current_ids {
            if !running_ids.contains(&id) {
                tracing::info!(pipeline_id = %id, "Stopping pipeline runner");
                if let Some(handle) = runners.remove(&id) {
                    handle.runner.stop();
                    handle.task.abort();
                }
            }
        }

        for pipeline in running_pipelines {
            if !runners.contains_key(&pipeline.id) {
                tracing::info!(
                    pipeline_id = %pipeline.id,
                    name = %pipeline.name,
                    "Starting pipeline runner"
                );

                match PipelineRunner::new(pipeline.clone(), self.db.clone()) {
                    Ok(runner) => {
                        let runner = Arc::new(runner);
                        let runner_clone = runner.clone();
                        let task = tokio::spawn(async move {
                            runner_clone.run().await;
                        });

                        runners.insert(pipeline.id, RunnerHandle { runner, task });
                    }
                    Err(e) => {
                        tracing::error!(
                            pipeline_id = %pipeline.id,
                            error = %e,
                            "Failed to create pipeline runner"
                        );

                        if let Err(update_err) = PipelineRepo::update_status(
                            self.db.conn(),
                            pipeline.id,
                            PipelineStatus::Failed,
                        ).await {
                            tracing::error!(error = %update_err, "Failed to update pipeline status");
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn active_count(&self) -> usize {
        self.runners.read().await.len()
    }

    pub async fn stop_all(&self) {
        let mut runners = self.runners.write().await;
        for (id, handle) in runners.drain() {
            tracing::info!(pipeline_id = %id, "Stopping pipeline runner");
            handle.runner.stop();
            handle.task.abort();
        }
    }
}
