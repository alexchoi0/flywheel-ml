use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PipelineEntry {
    pub id: Uuid,
    pub name: String,
    pub namespace: String,
    pub spec_hash: String,
    pub conveyor_pipeline_id: Option<String>,
}

pub struct PipelineRegistry {
    pipelines: Arc<RwLock<HashMap<Uuid, PipelineEntry>>>,
    by_name: Arc<RwLock<HashMap<(String, String), Uuid>>>,
}

impl PipelineRegistry {
    pub fn new() -> Self {
        Self {
            pipelines: Arc::new(RwLock::new(HashMap::new())),
            by_name: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, entry: PipelineEntry) {
        let id = entry.id;
        let key = (entry.name.clone(), entry.namespace.clone());

        self.pipelines.write().await.insert(id, entry);
        self.by_name.write().await.insert(key, id);
    }

    pub async fn unregister(&self, id: Uuid) -> Option<PipelineEntry> {
        let entry = self.pipelines.write().await.remove(&id)?;
        let key = (entry.name.clone(), entry.namespace.clone());
        self.by_name.write().await.remove(&key);
        Some(entry)
    }

    pub async fn get(&self, id: Uuid) -> Option<PipelineEntry> {
        self.pipelines.read().await.get(&id).cloned()
    }

    pub async fn get_by_name(&self, name: &str, namespace: &str) -> Option<PipelineEntry> {
        let key = (name.to_string(), namespace.to_string());
        let id = self.by_name.read().await.get(&key).copied()?;
        self.get(id).await
    }

    pub async fn list(&self, namespace: Option<&str>) -> Vec<PipelineEntry> {
        self.pipelines
            .read()
            .await
            .values()
            .filter(|e| namespace.map(|ns| e.namespace == ns).unwrap_or(true))
            .cloned()
            .collect()
    }

    pub async fn count(&self) -> usize {
        self.pipelines.read().await.len()
    }
}

impl Default for PipelineRegistry {
    fn default() -> Self {
        Self::new()
    }
}
