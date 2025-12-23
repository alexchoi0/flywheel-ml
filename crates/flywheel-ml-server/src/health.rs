use std::sync::Arc;
use tokio::sync::RwLock;

pub struct HealthTracker {
    state: Arc<RwLock<HealthState>>,
}

#[derive(Debug, Clone, Default)]
pub struct HealthState {
    pub database_connected: bool,
    pub conveyor_connected: bool,
    pub active_pipelines: u32,
    pub registered_models: u32,
}

impl HealthTracker {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(HealthState::default())),
        }
    }

    pub async fn update_database_status(&self, connected: bool) {
        let mut state = self.state.write().await;
        state.database_connected = connected;
    }

    pub async fn update_pipeline_count(&self, count: u32) {
        let mut state = self.state.write().await;
        state.active_pipelines = count;
    }

    pub async fn update_model_count(&self, count: u32) {
        let mut state = self.state.write().await;
        state.registered_models = count;
    }

    pub async fn get_state(&self) -> HealthState {
        self.state.read().await.clone()
    }

    pub async fn is_healthy(&self) -> bool {
        let state = self.state.read().await;
        state.database_connected
    }
}

impl Default for HealthTracker {
    fn default() -> Self {
        Self::new()
    }
}
