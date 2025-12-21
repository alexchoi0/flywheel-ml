use async_trait::async_trait;
use flywheel_core::LabeledExample;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Storage error: {0}")]
    Storage(String),
}

#[async_trait]
pub trait TrainingExporter: Send + Sync {
    async fn export(&self, example: LabeledExample) -> Result<(), ExportError>;
    async fn export_batch(&self, examples: Vec<LabeledExample>) -> Result<(), ExportError>;
    async fn flush(&self) -> Result<(), ExportError>;
}

pub struct LocalExporter {
    output_dir: std::path::PathBuf,
    format: ExportFormat,
}

#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Parquet,
    JsonLines,
    Csv,
}

impl LocalExporter {
    pub fn new(output_dir: impl Into<std::path::PathBuf>, format: ExportFormat) -> Self {
        Self {
            output_dir: output_dir.into(),
            format,
        }
    }
}

#[async_trait]
impl TrainingExporter for LocalExporter {
    async fn export(&self, example: LabeledExample) -> Result<(), ExportError> {
        self.export_batch(vec![example]).await
    }

    async fn export_batch(&self, examples: Vec<LabeledExample>) -> Result<(), ExportError> {
        // TODO: Implement actual export logic
        tracing::info!("Exporting {} examples to {:?}", examples.len(), self.output_dir);
        Ok(())
    }

    async fn flush(&self) -> Result<(), ExportError> {
        Ok(())
    }
}
