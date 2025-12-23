use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::ModelError;
use crate::feature::FeatureVector;
use crate::prediction::Prediction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub model_id: String,
    pub model_name: String,
    pub version: String,
    pub model_type: ModelType,
    pub endpoint: String,
    pub input_features: Vec<String>,
    pub output_field: String,
    pub labels: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
}

impl ModelMetadata {
    pub fn new(model_id: impl Into<String>, model_type: ModelType) -> Self {
        Self {
            model_id: model_id.into(),
            model_name: String::new(),
            version: "1.0.0".to_string(),
            model_type,
            endpoint: String::new(),
            input_features: Vec::new(),
            output_field: "prediction".to_string(),
            labels: HashMap::new(),
            created_at: Utc::now(),
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.model_name = name.into();
        self
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }

    pub fn with_input_features(mut self, features: Vec<String>) -> Self {
        self.input_features = features;
        self
    }

    pub fn with_output_field(mut self, field: impl Into<String>) -> Self {
        self.output_field = field.into();
        self
    }

    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModelType {
    AnomalyDetection,
    Classification,
    Regression,
    Clustering,
    Embedding,
    Custom,
}

impl ModelType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelType::AnomalyDetection => "anomaly_detection",
            ModelType::Classification => "classification",
            ModelType::Regression => "regression",
            ModelType::Clustering => "clustering",
            ModelType::Embedding => "embedding",
            ModelType::Custom => "custom",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHealth {
    pub status: HealthStatus,
    pub last_check: DateTime<Utc>,
    pub latency_ms: Option<u64>,
    pub error_rate: Option<f64>,
    pub message: Option<String>,
}

impl Default for ModelHealth {
    fn default() -> Self {
        Self {
            status: HealthStatus::Unknown,
            last_check: Utc::now(),
            latency_ms: None,
            error_rate: None,
            message: None,
        }
    }
}

#[async_trait]
pub trait Model: Send + Sync {
    fn metadata(&self) -> &ModelMetadata;

    async fn predict(&self, features: FeatureVector) -> Result<Prediction, ModelError>;

    async fn predict_batch(
        &self,
        features: Vec<FeatureVector>,
    ) -> Result<Vec<Prediction>, ModelError> {
        let mut results = Vec::with_capacity(features.len());
        for feature in features {
            results.push(self.predict(feature).await?);
        }
        Ok(results)
    }

    async fn health_check(&self) -> ModelHealth {
        ModelHealth::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model_id: String,
    pub endpoint: String,
    pub model_type: ModelType,
    pub timeout_ms: u64,
    pub batch_size: usize,
    pub fallback: FallbackStrategy,
    pub circuit_breaker: Option<CircuitBreakerConfig>,
    pub retry: Option<RetryConfig>,
}

impl ModelConfig {
    pub fn new(model_id: impl Into<String>) -> Self {
        Self {
            model_id: model_id.into(),
            endpoint: String::new(),
            model_type: ModelType::Custom,
            timeout_ms: 1000,
            batch_size: 32,
            fallback: FallbackStrategy::PassThrough,
            circuit_breaker: None,
            retry: None,
        }
    }

    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }

    pub fn with_model_type(mut self, model_type: ModelType) -> Self {
        self.model_type = model_type;
        self
    }

    pub fn with_timeout_ms(mut self, timeout: u64) -> Self {
        self.timeout_ms = timeout;
        self
    }

    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    pub fn with_fallback(mut self, fallback: FallbackStrategy) -> Self {
        self.fallback = fallback;
        self
    }

    pub fn with_circuit_breaker(mut self, config: CircuitBreakerConfig) -> Self {
        self.circuit_breaker = Some(config);
        self
    }

    pub fn with_retry(mut self, config: RetryConfig) -> Self {
        self.retry = Some(config);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FallbackStrategy {
    ReturnNull,
    UseDefault(serde_json::Value),
    UseLastKnown,
    SendToDlq,
    PassThrough,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub half_open_max_calls: u32,
    pub reset_timeout_secs: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            half_open_max_calls: 3,
            reset_timeout_secs: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            multiplier: 2.0,
        }
    }
}
