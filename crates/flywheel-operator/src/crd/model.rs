use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ModelCrdSpec {
    pub model_id: String,
    pub model_name: String,
    pub version: String,
    pub model_type: String,
    pub endpoint: String,
    pub input_features: Vec<String>,
    pub output_field: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ModelStatus {
    pub observed_generation: i64,
    pub status: String,
    pub accuracy: Option<f64>,
    pub latency_p99_ms: Option<i64>,
    pub message: Option<String>,
}

// TODO: Add kube CustomResource derive when kube is added as a dependency
