use flywheel_dsl::FlywheelPipelineSpec;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FlywheelPipelineCrdSpec {
    #[serde(flatten)]
    pub inner: FlywheelPipelineSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FlywheelPipelineStatus {
    pub observed_generation: i64,
    pub pipeline_id: Option<String>,
    pub conveyor_pipeline_id: Option<String>,
    pub status: String,
    pub message: Option<String>,
}

// TODO: Add kube CustomResource derive when kube is added as a dependency
