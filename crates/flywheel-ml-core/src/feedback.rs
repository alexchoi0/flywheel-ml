use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::FeedbackError;
use crate::prediction::StoredPrediction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackRecord {
    pub feedback_id: String,
    pub prediction_id: String,
    pub model_id: String,
    pub ground_truth: GroundTruth,
    pub feedback_time: DateTime<Utc>,
    pub delay_ms: u64,
    pub source: FeedbackSource,
    pub metadata: HashMap<String, String>,
}

impl FeedbackRecord {
    pub fn new(
        prediction_id: impl Into<String>,
        model_id: impl Into<String>,
        ground_truth: GroundTruth,
        source: FeedbackSource,
    ) -> Self {
        Self {
            feedback_id: Uuid::new_v4().to_string(),
            prediction_id: prediction_id.into(),
            model_id: model_id.into(),
            ground_truth,
            feedback_time: Utc::now(),
            delay_ms: 0,
            source,
            metadata: HashMap::new(),
        }
    }

    pub fn with_delay(mut self, prediction_time: DateTime<Utc>) -> Self {
        let delay = self.feedback_time.signed_duration_since(prediction_time);
        self.delay_ms = delay.num_milliseconds().max(0) as u64;
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GroundTruth {
    Label(String),
    Value(f64),
    Binary(bool),
    Ranking(Vec<String>),
    MultiLabel(Vec<String>),
    Custom(serde_json::Value),
}

impl GroundTruth {
    pub fn label(label: impl Into<String>) -> Self {
        GroundTruth::Label(label.into())
    }

    pub fn value(value: f64) -> Self {
        GroundTruth::Value(value)
    }

    pub fn binary(is_positive: bool) -> Self {
        GroundTruth::Binary(is_positive)
    }

    pub fn ranking(items: Vec<String>) -> Self {
        GroundTruth::Ranking(items)
    }

    pub fn multi_label(labels: Vec<String>) -> Self {
        GroundTruth::MultiLabel(labels)
    }

    pub fn as_label(&self) -> Option<&str> {
        match self {
            GroundTruth::Label(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_value(&self) -> Option<f64> {
        match self {
            GroundTruth::Value(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_binary(&self) -> Option<bool> {
        match self {
            GroundTruth::Binary(b) => Some(*b),
            GroundTruth::Label(s) => match s.to_lowercase().as_str() {
                "true" | "yes" | "1" | "positive" | "anomaly" => Some(true),
                "false" | "no" | "0" | "negative" | "normal" => Some(false),
                _ => None,
            },
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FeedbackSource {
    Explicit {
        user_id: String,
        action: String,
    },
    Implicit {
        event_type: String,
        context: HashMap<String, String>,
    },
    Automated {
        rule: String,
        confidence: f64,
    },
    Manual {
        annotator_id: String,
    },
}

impl FeedbackSource {
    pub fn explicit(user_id: impl Into<String>, action: impl Into<String>) -> Self {
        FeedbackSource::Explicit {
            user_id: user_id.into(),
            action: action.into(),
        }
    }

    pub fn implicit(event_type: impl Into<String>) -> Self {
        FeedbackSource::Implicit {
            event_type: event_type.into(),
            context: HashMap::new(),
        }
    }

    pub fn implicit_with_context(
        event_type: impl Into<String>,
        context: HashMap<String, String>,
    ) -> Self {
        FeedbackSource::Implicit {
            event_type: event_type.into(),
            context,
        }
    }

    pub fn automated(rule: impl Into<String>, confidence: f64) -> Self {
        FeedbackSource::Automated {
            rule: rule.into(),
            confidence,
        }
    }

    pub fn manual(annotator_id: impl Into<String>) -> Self {
        FeedbackSource::Manual {
            annotator_id: annotator_id.into(),
        }
    }

    pub fn confidence(&self) -> f64 {
        match self {
            FeedbackSource::Explicit { .. } => 1.0,
            FeedbackSource::Implicit { .. } => 0.7,
            FeedbackSource::Automated { confidence, .. } => *confidence,
            FeedbackSource::Manual { .. } => 0.95,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabeledExample {
    pub example_id: String,
    pub prediction_id: String,
    pub model_id: String,
    pub model_version: String,
    pub features: serde_json::Value,
    pub prediction: serde_json::Value,
    pub ground_truth: GroundTruth,
    pub prediction_timestamp: DateTime<Utc>,
    pub feedback_timestamp: DateTime<Utc>,
    pub delay_ms: u64,
    pub feedback_confidence: f64,
    pub is_correct: Option<bool>,
    pub metadata: HashMap<String, String>,
}

impl LabeledExample {
    pub fn from_prediction_and_feedback(
        stored: &StoredPrediction,
        feedback: &FeedbackRecord,
    ) -> Self {
        let is_correct = Self::compute_correctness(&stored.prediction.result, &feedback.ground_truth);

        Self {
            example_id: Uuid::new_v4().to_string(),
            prediction_id: stored.prediction.prediction_id.clone(),
            model_id: stored.prediction.model_id.clone(),
            model_version: stored.prediction.model_version.clone(),
            features: stored.features.clone(),
            prediction: serde_json::to_value(&stored.prediction.result).unwrap_or_default(),
            ground_truth: feedback.ground_truth.clone(),
            prediction_timestamp: stored.prediction.timestamp,
            feedback_timestamp: feedback.feedback_time,
            delay_ms: feedback.delay_ms,
            feedback_confidence: feedback.source.confidence(),
            is_correct,
            metadata: feedback.metadata.clone(),
        }
    }

    fn compute_correctness(
        prediction: &crate::prediction::PredictionResult,
        ground_truth: &GroundTruth,
    ) -> Option<bool> {
        use crate::prediction::PredictionResult;

        match (prediction, ground_truth) {
            (PredictionResult::Anomaly { is_anomaly, .. }, GroundTruth::Binary(truth)) => {
                Some(*is_anomaly == *truth)
            }
            (PredictionResult::Anomaly { is_anomaly, .. }, GroundTruth::Label(label)) => {
                let is_positive = matches!(
                    label.to_lowercase().as_str(),
                    "anomaly" | "true" | "yes" | "1" | "positive"
                );
                Some(*is_anomaly == is_positive)
            }
            (PredictionResult::Classification { class, .. }, GroundTruth::Label(truth)) => {
                Some(class.eq_ignore_ascii_case(truth))
            }
            (PredictionResult::Regression { value, .. }, GroundTruth::Value(truth)) => {
                let tolerance = truth.abs() * 0.1;
                Some((value - truth).abs() <= tolerance)
            }
            _ => None,
        }
    }

    pub fn is_positive(&self) -> bool {
        match &self.ground_truth {
            GroundTruth::Binary(b) => *b,
            GroundTruth::Label(s) => matches!(
                s.to_lowercase().as_str(),
                "anomaly" | "true" | "yes" | "1" | "positive"
            ),
            _ => false,
        }
    }

    pub fn is_false_positive(&self) -> bool {
        !self.is_positive() && self.is_correct == Some(false)
    }

    pub fn is_false_negative(&self) -> bool {
        self.is_positive() && self.is_correct == Some(false)
    }
}

#[async_trait]
pub trait FeedbackCollector: Send + Sync {
    async fn collect(&self, feedback: FeedbackRecord) -> Result<(), FeedbackError>;

    async fn query_feedback(
        &self,
        prediction_ids: &[String],
    ) -> Result<Vec<FeedbackRecord>, FeedbackError>;

    async fn get_feedback_rate(
        &self,
        model_id: &str,
        window: Duration,
    ) -> Result<f64, FeedbackError>;

    async fn get_accuracy(
        &self,
        model_id: &str,
        window: Duration,
    ) -> Result<Option<f64>, FeedbackError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackConfig {
    pub join_key_field: String,
    pub prediction_id_field: String,
    pub max_join_delay: Duration,
    pub label_extraction: LabelExtractionConfig,
    pub sampling: SamplingConfig,
}

impl Default for FeedbackConfig {
    fn default() -> Self {
        Self {
            join_key_field: "trace_id".to_string(),
            prediction_id_field: "_fw_prediction_id".to_string(),
            max_join_delay: Duration::hours(24),
            label_extraction: LabelExtractionConfig::default(),
            sampling: SamplingConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelExtractionConfig {
    pub ground_truth_field: String,
    pub label_mapping: HashMap<String, String>,
    pub implicit_labels: Vec<ImplicitLabelRule>,
}

impl Default for LabelExtractionConfig {
    fn default() -> Self {
        Self {
            ground_truth_field: "label".to_string(),
            label_mapping: HashMap::new(),
            implicit_labels: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplicitLabelRule {
    pub event_type: String,
    pub label: String,
    pub confidence: f64,
}

impl ImplicitLabelRule {
    pub fn new(event_type: impl Into<String>, label: impl Into<String>, confidence: f64) -> Self {
        Self {
            event_type: event_type.into(),
            label: label.into(),
            confidence: confidence.clamp(0.0, 1.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "strategy", rename_all = "snake_case")]
pub enum SamplingConfig {
    All,
    Random {
        rate: f64,
    },
    Stratified {
        positive_rate: f64,
        negative_rate: f64,
    },
    HardNegative {
        threshold: f64,
    },
    ReservoirSampling {
        size: usize,
    },
}

impl Default for SamplingConfig {
    fn default() -> Self {
        SamplingConfig::All
    }
}

impl SamplingConfig {
    pub fn should_sample(&self, example: &LabeledExample) -> bool {
        match self {
            SamplingConfig::All => true,
            SamplingConfig::Random { rate } => rand_rate(*rate),
            SamplingConfig::Stratified {
                positive_rate,
                negative_rate,
            } => {
                if example.is_positive() {
                    rand_rate(*positive_rate)
                } else {
                    rand_rate(*negative_rate)
                }
            }
            SamplingConfig::HardNegative { threshold } => {
                if example.is_false_positive() {
                    if let Some(confidence) = example
                        .prediction
                        .get("confidence")
                        .and_then(|v| v.as_f64())
                    {
                        return confidence > *threshold;
                    }
                }
                example.is_positive()
            }
            SamplingConfig::ReservoirSampling { .. } => true,
        }
    }
}

fn rand_rate(rate: f64) -> bool {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::time::Instant::now().hash(&mut hasher);
    let hash = hasher.finish();
    (hash as f64 / u64::MAX as f64) < rate
}
