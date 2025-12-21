use crate::types::*;

// TODO: Convert FlywheelPipelineManifest to Conveyor Pipeline format
// This will require importing conveyor-dsl types

pub fn to_conveyor_pipeline(_manifest: &FlywheelPipelineManifest) -> Result<(), ConvertError> {
    // Placeholder for Conveyor integration
    Ok(())
}

#[derive(Debug)]
pub struct ConvertError {
    pub message: String,
}
