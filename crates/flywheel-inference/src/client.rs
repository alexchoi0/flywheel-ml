use async_trait::async_trait;
use flywheel_core::{FeatureVector, Model, ModelError, ModelHealth, ModelMetadata, Prediction};

pub struct InferenceClient {
    endpoint: String,
    metadata: ModelMetadata,
}

impl InferenceClient {
    pub fn new(endpoint: impl Into<String>, metadata: ModelMetadata) -> Self {
        Self {
            endpoint: endpoint.into(),
            metadata,
        }
    }
}

#[async_trait]
impl Model for InferenceClient {
    fn metadata(&self) -> &ModelMetadata {
        &self.metadata
    }

    async fn predict(&self, _features: FeatureVector) -> Result<Prediction, ModelError> {
        // TODO: Implement gRPC call to Python model service
        Err(ModelError::Unavailable("Not implemented".to_string()))
    }

    async fn health_check(&self) -> ModelHealth {
        ModelHealth::default()
    }
}
