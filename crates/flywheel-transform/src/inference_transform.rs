use flywheel_core::{FeatureVector, Model, ModelError, Prediction};
use flywheel_inference::circuit_breaker::CircuitBreaker;
use std::sync::Arc;

pub struct InferenceTransform {
    model: Arc<dyn Model>,
    circuit_breaker: CircuitBreaker,
}

impl InferenceTransform {
    pub fn new(model: Arc<dyn Model>) -> Self {
        Self {
            model,
            circuit_breaker: CircuitBreaker::new(Default::default()),
        }
    }

    pub async fn process(&self, features: FeatureVector) -> Result<Prediction, ModelError> {
        if !self.circuit_breaker.can_execute() {
            return Err(ModelError::CircuitBreakerOpen(
                self.model.metadata().model_id.clone(),
            ));
        }

        match self.model.predict(features).await {
            Ok(prediction) => {
                self.circuit_breaker.record_success();
                Ok(prediction)
            }
            Err(e) => {
                self.circuit_breaker.record_failure();
                Err(e)
            }
        }
    }

    pub async fn process_batch(
        &self,
        features: Vec<FeatureVector>,
    ) -> Result<Vec<Prediction>, ModelError> {
        if !self.circuit_breaker.can_execute() {
            return Err(ModelError::CircuitBreakerOpen(
                self.model.metadata().model_id.clone(),
            ));
        }

        match self.model.predict_batch(features).await {
            Ok(predictions) => {
                self.circuit_breaker.record_success();
                Ok(predictions)
            }
            Err(e) => {
                self.circuit_breaker.record_failure();
                Err(e)
            }
        }
    }
}
