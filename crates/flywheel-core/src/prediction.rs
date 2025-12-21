use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    pub prediction_id: String,
    pub model_id: String,
    pub model_version: String,
    pub timestamp: DateTime<Utc>,
    pub result: PredictionResult,
    pub confidence: Option<f64>,
    pub latency_us: u64,
    pub features_hash: String,
    pub metadata: HashMap<String, String>,
}

impl Prediction {
    pub fn new(model_id: impl Into<String>, result: PredictionResult) -> Self {
        Self {
            prediction_id: Uuid::new_v4().to_string(),
            model_id: model_id.into(),
            model_version: String::new(),
            timestamp: Utc::now(),
            result,
            confidence: None,
            latency_us: 0,
            features_hash: String::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.model_version = version.into();
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = Some(confidence);
        self
    }

    pub fn with_latency(mut self, latency_us: u64) -> Self {
        self.latency_us = latency_us;
        self
    }

    pub fn with_features_hash(mut self, hash: impl Into<String>) -> Self {
        self.features_hash = hash.into();
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn is_anomaly(&self) -> bool {
        match &self.result {
            PredictionResult::Anomaly { is_anomaly, .. } => *is_anomaly,
            _ => false,
        }
    }

    pub fn anomaly_score(&self) -> Option<f64> {
        match &self.result {
            PredictionResult::Anomaly { score, .. } => Some(*score),
            _ => None,
        }
    }

    pub fn predicted_class(&self) -> Option<&str> {
        match &self.result {
            PredictionResult::Classification { class, .. } => Some(class),
            _ => None,
        }
    }

    pub fn regression_value(&self) -> Option<f64> {
        match &self.result {
            PredictionResult::Regression { value, .. } => Some(*value),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PredictionResult {
    Anomaly {
        score: f64,
        is_anomaly: bool,
        threshold: f64,
        #[serde(default)]
        contributing_features: Vec<String>,
    },
    Classification {
        class: String,
        probabilities: HashMap<String, f64>,
    },
    Regression {
        value: f64,
        #[serde(default)]
        confidence_interval: Option<(f64, f64)>,
    },
    Clustering {
        cluster_id: i32,
        distance: f64,
    },
    Embedding {
        vector: Vec<f32>,
    },
    Custom(serde_json::Value),
}

impl PredictionResult {
    pub fn anomaly(score: f64, threshold: f64) -> Self {
        PredictionResult::Anomaly {
            score,
            is_anomaly: score > threshold,
            threshold,
            contributing_features: Vec::new(),
        }
    }

    pub fn anomaly_with_features(
        score: f64,
        threshold: f64,
        contributing_features: Vec<String>,
    ) -> Self {
        PredictionResult::Anomaly {
            score,
            is_anomaly: score > threshold,
            threshold,
            contributing_features,
        }
    }

    pub fn classification(class: impl Into<String>, probabilities: HashMap<String, f64>) -> Self {
        PredictionResult::Classification {
            class: class.into(),
            probabilities,
        }
    }

    pub fn binary_classification(class: impl Into<String>, probability: f64) -> Self {
        let class = class.into();
        let mut probabilities = HashMap::new();
        probabilities.insert(class.clone(), probability);
        probabilities.insert("other".to_string(), 1.0 - probability);
        PredictionResult::Classification { class, probabilities }
    }

    pub fn regression(value: f64) -> Self {
        PredictionResult::Regression {
            value,
            confidence_interval: None,
        }
    }

    pub fn regression_with_interval(value: f64, lower: f64, upper: f64) -> Self {
        PredictionResult::Regression {
            value,
            confidence_interval: Some((lower, upper)),
        }
    }

    pub fn clustering(cluster_id: i32, distance: f64) -> Self {
        PredictionResult::Clustering {
            cluster_id,
            distance,
        }
    }

    pub fn embedding(vector: Vec<f32>) -> Self {
        PredictionResult::Embedding { vector }
    }

    pub fn result_type(&self) -> &'static str {
        match self {
            PredictionResult::Anomaly { .. } => "anomaly",
            PredictionResult::Classification { .. } => "classification",
            PredictionResult::Regression { .. } => "regression",
            PredictionResult::Clustering { .. } => "clustering",
            PredictionResult::Embedding { .. } => "embedding",
            PredictionResult::Custom(_) => "custom",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchPredictionResult {
    pub batch_id: String,
    pub predictions: Vec<Prediction>,
    pub stats: BatchStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchStats {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub avg_latency_us: u64,
    pub p50_latency_us: u64,
    pub p99_latency_us: u64,
}

impl BatchStats {
    pub fn from_predictions(predictions: &[Prediction]) -> Self {
        if predictions.is_empty() {
            return Self {
                total: 0,
                succeeded: 0,
                failed: 0,
                avg_latency_us: 0,
                p50_latency_us: 0,
                p99_latency_us: 0,
            };
        }

        let mut latencies: Vec<u64> = predictions.iter().map(|p| p.latency_us).collect();
        latencies.sort();

        let total = predictions.len();
        let avg_latency_us = latencies.iter().sum::<u64>() / total as u64;
        let p50_latency_us = latencies[total / 2];
        let p99_latency_us = latencies[(total * 99 / 100).min(total - 1)];

        Self {
            total,
            succeeded: total,
            failed: 0,
            avg_latency_us,
            p50_latency_us,
            p99_latency_us,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredPrediction {
    pub prediction: Prediction,
    pub features: serde_json::Value,
    pub source_record_id: String,
    pub stored_at: DateTime<Utc>,
    pub ttl_seconds: Option<u64>,
}

impl StoredPrediction {
    pub fn new(prediction: Prediction, features: serde_json::Value, source_record_id: String) -> Self {
        Self {
            prediction,
            features,
            source_record_id,
            stored_at: Utc::now(),
            ttl_seconds: None,
        }
    }

    pub fn with_ttl(mut self, ttl_seconds: u64) -> Self {
        self.ttl_seconds = Some(ttl_seconds);
        self
    }

    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl_seconds {
            let elapsed = Utc::now().signed_duration_since(self.stored_at);
            elapsed.num_seconds() > ttl as i64
        } else {
            false
        }
    }
}
