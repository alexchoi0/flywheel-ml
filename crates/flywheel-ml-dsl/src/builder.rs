use crate::types::*;
use crate::validation::{validate_manifest, ValidationError};
use std::marker::PhantomData;

pub struct NoSource;
pub struct HasSource;
pub struct NoStages;
pub struct HasStages;
pub struct NoSinks;
pub struct HasSinks;

pub struct FlywheelPipelineBuilder<Source, Stages, Sinks> {
    name: String,
    namespace: String,
    source: Option<String>,
    stages: Vec<FlywheelStage>,
    feedback: Option<FeedbackSpec>,
    training_export: Option<TrainingExportSpec>,
    sinks: Vec<SinkSpec>,
    enabled: bool,
    _marker: PhantomData<(Source, Stages, Sinks)>,
}

impl FlywheelPipelineBuilder<NoSource, NoStages, NoSinks> {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            namespace: "default".to_string(),
            source: None,
            stages: Vec::new(),
            feedback: None,
            training_export: None,
            sinks: Vec::new(),
            enabled: true,
            _marker: PhantomData,
        }
    }
}

impl<Stages, Sinks> FlywheelPipelineBuilder<NoSource, Stages, Sinks> {
    pub fn source(self, source: impl Into<String>) -> FlywheelPipelineBuilder<HasSource, Stages, Sinks> {
        FlywheelPipelineBuilder {
            name: self.name,
            namespace: self.namespace,
            source: Some(source.into()),
            stages: self.stages,
            feedback: self.feedback,
            training_export: self.training_export,
            sinks: self.sinks,
            enabled: self.enabled,
            _marker: PhantomData,
        }
    }
}

impl<Source, Sinks> FlywheelPipelineBuilder<Source, NoStages, Sinks> {
    pub fn stage(self, stage: FlywheelStage) -> FlywheelPipelineBuilder<Source, HasStages, Sinks> {
        let mut stages = self.stages;
        stages.push(stage);
        FlywheelPipelineBuilder {
            name: self.name,
            namespace: self.namespace,
            source: self.source,
            stages,
            feedback: self.feedback,
            training_export: self.training_export,
            sinks: self.sinks,
            enabled: self.enabled,
            _marker: PhantomData,
        }
    }
}

impl<Source, Stages> FlywheelPipelineBuilder<Source, Stages, NoSinks> {
    pub fn sink(self, sink: SinkSpec) -> FlywheelPipelineBuilder<Source, Stages, HasSinks> {
        let mut sinks = self.sinks;
        sinks.push(sink);
        FlywheelPipelineBuilder {
            name: self.name,
            namespace: self.namespace,
            source: self.source,
            stages: self.stages,
            feedback: self.feedback,
            training_export: self.training_export,
            sinks,
            enabled: self.enabled,
            _marker: PhantomData,
        }
    }
}

impl<Source, Stages, Sinks> FlywheelPipelineBuilder<Source, Stages, Sinks> {
    pub fn namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = namespace.into();
        self
    }

    pub fn with_feedback(mut self, feedback: FeedbackSpec) -> Self {
        self.feedback = Some(feedback);
        self
    }

    pub fn with_training_export(mut self, export: TrainingExportSpec) -> Self {
        self.training_export = Some(export);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl FlywheelPipelineBuilder<HasSource, HasStages, HasSinks> {
    pub fn add_stage(mut self, stage: FlywheelStage) -> Self {
        self.stages.push(stage);
        self
    }

    pub fn add_sink(mut self, sink: SinkSpec) -> Self {
        self.sinks.push(sink);
        self
    }

    pub fn build(self) -> Result<FlywheelPipelineManifest, ValidationError> {
        let manifest = FlywheelPipelineManifest {
            api_version: "flywheel-ml.io/v1".to_string(),
            kind: "FlywheelPipeline".to_string(),
            metadata: ObjectMeta {
                name: self.name,
                namespace: Some(self.namespace),
                labels: std::collections::HashMap::new(),
                annotations: std::collections::HashMap::new(),
            },
            spec: FlywheelPipelineSpec {
                source: self.source.unwrap(),
                stages: self.stages,
                feedback: self.feedback,
                training_export: self.training_export,
                sinks: self.sinks,
                enabled: self.enabled,
            },
        };

        validate_manifest(&manifest)?;
        Ok(manifest)
    }
}
