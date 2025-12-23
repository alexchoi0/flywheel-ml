use crate::types::FlywheelPipelineManifest;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Invalid API version: {0}")]
    InvalidApiVersion(String),
    #[error("Invalid kind: expected FlywheelPipeline, got {0}")]
    InvalidKind(String),
}

pub fn parse_manifest(yaml: &str) -> Result<FlywheelPipelineManifest, ParseError> {
    let manifest: FlywheelPipelineManifest = serde_yaml::from_str(yaml)?;

    if manifest.api_version != "flywheel-ml.io/v1" {
        return Err(ParseError::InvalidApiVersion(manifest.api_version));
    }

    if manifest.kind != "FlywheelPipeline" {
        return Err(ParseError::InvalidKind(manifest.kind));
    }

    Ok(manifest)
}

pub fn parse_manifests(yaml: &str) -> Result<Vec<FlywheelPipelineManifest>, ParseError> {
    let mut manifests = Vec::new();

    for doc in serde_yaml::Deserializer::from_str(yaml) {
        let manifest: FlywheelPipelineManifest = serde::Deserialize::deserialize(doc)?;
        manifests.push(manifest);
    }

    Ok(manifests)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_manifest() {
        let yaml = r#"
apiVersion: flywheel-ml.io/v1
kind: FlywheelPipeline
metadata:
  name: test-pipeline
  namespace: default
spec:
  source: kafka-topic
  stages:
    - id: features
      type: feature-extraction
      config:
        features:
          - name: cpu
            source_field: $.cpu
  sinks:
    - name: output
      all: true
"#;

        let manifest = parse_manifest(yaml).unwrap();
        assert_eq!(manifest.metadata.name, "test-pipeline");
        assert_eq!(manifest.spec.stages.len(), 1);
    }
}
