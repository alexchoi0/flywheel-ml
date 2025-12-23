use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FlywheelPipelineManifest {
    pub api_version: String,
    pub kind: String,
    pub metadata: ObjectMeta,
    pub spec: FlywheelPipelineSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ObjectMeta {
    pub name: String,
    pub namespace: Option<String>,
    #[serde(default)]
    pub labels: HashMap<String, String>,
    #[serde(default)]
    pub annotations: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FlywheelPipelineSpec {
    pub source: String,
    pub stages: Vec<FlywheelStage>,
    #[serde(default)]
    pub feedback: Option<FeedbackSpec>,
    #[serde(default)]
    pub training_export: Option<TrainingExportSpec>,
    pub sinks: Vec<SinkSpec>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FlywheelStage {
    pub id: String,
    #[serde(rename = "type")]
    pub stage_type: FlywheelStageType,
    #[serde(default)]
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum FlywheelStageType {
    FeatureExtraction,
    MlInference,
    DriftDetection,
    FeedbackJoin,
    TrainingExport,
}

impl FlywheelStageType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FlywheelStageType::FeatureExtraction => "feature-extraction",
            FlywheelStageType::MlInference => "ml-inference",
            FlywheelStageType::DriftDetection => "drift-detection",
            FlywheelStageType::FeedbackJoin => "feedback-join",
            FlywheelStageType::TrainingExport => "training-export",
        }
    }

    pub fn to_conveyor_labels(&self) -> HashMap<String, String> {
        let mut labels = HashMap::new();
        labels.insert("flywheel-ml.io/stage-type".to_string(), self.as_str().to_string());
        labels.insert("flywheel-ml.io/version".to_string(), "v1".to_string());
        labels
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FeatureExtractionConfig {
    pub features: Vec<FeatureDef>,
    #[serde(default)]
    pub include_raw: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FeatureDef {
    pub name: String,
    pub source_field: String,
    #[serde(default)]
    pub transform: Option<FeatureTransformSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FeatureTransformSpec {
    Normalize { min: f64, max: f64 },
    Log1p {},
    Clip { min: f64, max: f64 },
    Bucketize { boundaries: Vec<f64> },
    OneHot { categories: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MlInferenceConfig {
    pub model_endpoint: String,
    pub model_id: String,
    pub input_features: Vec<String>,
    pub output_field: String,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    #[serde(default)]
    pub fallback: FallbackStrategy,
}

fn default_timeout_ms() -> u64 {
    1000
}

fn default_batch_size() -> usize {
    32
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum FallbackStrategy {
    #[default]
    Passthrough,
    ReturnNull,
    UseDefault(serde_json::Value),
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DriftDetectionConfig {
    #[serde(default)]
    pub mode: DriftMode,
    pub baseline_uri: String,
    #[serde(default = "default_window_size")]
    pub window_size: usize,
    #[serde(default = "default_check_interval")]
    pub check_interval_secs: u64,
    pub thresholds: DriftThresholds,
    #[serde(default)]
    pub on_drift: DriftAction,
}

fn default_window_size() -> usize {
    10000
}

fn default_check_interval() -> u64 {
    300
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum DriftMode {
    #[default]
    Shadow,
    Blocking,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DriftThresholds {
    #[serde(default = "default_psi")]
    pub psi: f64,
    #[serde(default = "default_kl")]
    pub kl_divergence: f64,
}

fn default_psi() -> f64 {
    0.25
}

fn default_kl() -> f64 {
    0.1
}

impl Default for DriftThresholds {
    fn default() -> Self {
        Self {
            psi: default_psi(),
            kl_divergence: default_kl(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum DriftAction {
    #[default]
    Alert,
    Retrain,
    Fallback { to_model: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FeedbackSpec {
    pub source: String,
    pub join_key: String,
    #[serde(default = "default_max_delay_hours")]
    pub max_delay_hours: u64,
    #[serde(default)]
    pub labels: Vec<ImplicitLabelSpec>,
}

fn default_max_delay_hours() -> u64 {
    24
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ImplicitLabelSpec {
    pub event: String,
    pub label: String,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
}

fn default_confidence() -> f64 {
    0.9
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TrainingExportSpec {
    pub destination_uri: String,
    #[serde(default)]
    pub format: ExportFormat,
    #[serde(default)]
    pub partition_by: Vec<String>,
    #[serde(default)]
    pub sampling: SamplingSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    #[default]
    Parquet,
    TfRecord,
    Csv,
    JsonLines,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "strategy", rename_all = "snake_case")]
pub enum SamplingSpec {
    All,
    Random { rate: f64 },
    Stratified { positive_rate: f64, negative_rate: f64 },
}

impl Default for SamplingSpec {
    fn default() -> Self {
        SamplingSpec::All
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SinkSpec {
    pub name: String,
    #[serde(default)]
    pub condition: Option<String>,
    #[serde(default)]
    pub all: bool,
}
