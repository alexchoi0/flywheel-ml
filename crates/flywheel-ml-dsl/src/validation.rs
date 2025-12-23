use crate::types::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Pipeline name cannot be empty")]
    EmptyName,
    #[error("Pipeline must have at least one stage")]
    NoStages,
    #[error("Pipeline must have at least one sink")]
    NoSinks,
    #[error("Stage '{0}' has no id")]
    MissingStageId(String),
    #[error("Duplicate stage id: {0}")]
    DuplicateStageId(String),
    #[error("Invalid feature extraction config: {0}")]
    InvalidFeatureExtraction(String),
    #[error("Invalid ML inference config: {0}")]
    InvalidMlInference(String),
    #[error("Invalid drift detection config: {0}")]
    InvalidDriftDetection(String),
}

pub fn validate_manifest(manifest: &FlywheelPipelineManifest) -> Result<(), ValidationError> {
    if manifest.metadata.name.is_empty() {
        return Err(ValidationError::EmptyName);
    }

    validate_spec(&manifest.spec)?;

    Ok(())
}

pub fn validate_spec(spec: &FlywheelPipelineSpec) -> Result<(), ValidationError> {
    if spec.stages.is_empty() {
        return Err(ValidationError::NoStages);
    }

    if spec.sinks.is_empty() {
        return Err(ValidationError::NoSinks);
    }

    let mut stage_ids = std::collections::HashSet::new();
    for stage in &spec.stages {
        if stage.id.is_empty() {
            return Err(ValidationError::MissingStageId(format!("{:?}", stage.stage_type)));
        }

        if !stage_ids.insert(&stage.id) {
            return Err(ValidationError::DuplicateStageId(stage.id.clone()));
        }

        validate_stage(stage)?;
    }

    Ok(())
}

fn validate_stage(stage: &FlywheelStage) -> Result<(), ValidationError> {
    match stage.stage_type {
        FlywheelStageType::FeatureExtraction => {
            let config: Result<FeatureExtractionConfig, _> =
                serde_json::from_value(stage.config.clone());
            if let Err(e) = config {
                return Err(ValidationError::InvalidFeatureExtraction(e.to_string()));
            }
        }
        FlywheelStageType::MlInference => {
            let config: Result<MlInferenceConfig, _> =
                serde_json::from_value(stage.config.clone());
            if let Err(e) = config {
                return Err(ValidationError::InvalidMlInference(e.to_string()));
            }
        }
        FlywheelStageType::DriftDetection => {
            let config: Result<DriftDetectionConfig, _> =
                serde_json::from_value(stage.config.clone());
            if let Err(e) = config {
                return Err(ValidationError::InvalidDriftDetection(e.to_string()));
            }
        }
        _ => {}
    }

    Ok(())
}
