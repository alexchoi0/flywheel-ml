use async_trait::async_trait;
use flywheel_ml_core::LabeledExample;
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

use crate::format::{CsvWriter, FormatWriter, JsonLinesWriter, ParquetBatchWriter};

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
    output_dir: PathBuf,
    format: ExportFormat,
    partition_by: Vec<PartitionKey>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Parquet,
    JsonLines,
    Csv,
}

impl ExportFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Parquet => "parquet",
            ExportFormat::JsonLines => "jsonl",
            ExportFormat::Csv => "csv",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionKey {
    ModelId,
    ModelVersion,
    Date,
}

impl LocalExporter {
    pub fn new(output_dir: impl Into<PathBuf>, format: ExportFormat) -> Self {
        Self {
            output_dir: output_dir.into(),
            format,
            partition_by: vec![PartitionKey::ModelId, PartitionKey::Date],
        }
    }

    pub fn with_partitions(mut self, partition_by: Vec<PartitionKey>) -> Self {
        self.partition_by = partition_by;
        self
    }

    fn partition_examples(
        &self,
        examples: &[LabeledExample],
    ) -> HashMap<PartitionPath, Vec<LabeledExample>> {
        let mut partitions: HashMap<PartitionPath, Vec<LabeledExample>> = HashMap::new();

        for example in examples {
            let path = self.build_partition_path(example);
            partitions.entry(path).or_default().push(example.clone());
        }

        partitions
    }

    fn build_partition_path(&self, example: &LabeledExample) -> PartitionPath {
        let mut parts = Vec::new();

        for key in &self.partition_by {
            match key {
                PartitionKey::ModelId => {
                    parts.push(format!("model_id={}", example.model_id));
                }
                PartitionKey::ModelVersion => {
                    parts.push(format!("model_version={}", example.model_version));
                }
                PartitionKey::Date => {
                    let date = example.prediction_timestamp.date_naive();
                    parts.push(format!("date={}", date.format("%Y-%m-%d")));
                }
            }
        }

        PartitionPath { parts }
    }

    fn write_examples_sync(
        &self,
        path: &std::path::Path,
        examples: &[LabeledExample],
    ) -> Result<PathBuf, ExportError> {
        std::fs::create_dir_all(path)?;

        let filename = format!(
            "examples_{}.{}",
            chrono::Utc::now().timestamp_millis(),
            self.format.extension()
        );
        let file_path = path.join(&filename);
        let file = std::fs::File::create(&file_path)?;

        match self.format {
            ExportFormat::JsonLines => {
                let mut writer = JsonLinesWriter::new(file);
                for example in examples {
                    writer.write(example)?;
                }
                writer.flush()?;
            }
            ExportFormat::Csv => {
                let mut writer = CsvWriter::new(file);
                for example in examples {
                    writer.write(example)?;
                }
                writer.flush()?;
            }
            ExportFormat::Parquet => {
                let mut writer = ParquetBatchWriter::new(file, 1000)?;
                for example in examples {
                    writer.write(example)?;
                }
                writer.flush()?;
            }
        }

        Ok(file_path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PartitionPath {
    parts: Vec<String>,
}

impl PartitionPath {
    fn to_path(&self, base: &std::path::Path) -> PathBuf {
        let mut path = base.to_path_buf();
        for part in &self.parts {
            path = path.join(part);
        }
        path
    }
}

#[async_trait]
impl TrainingExporter for LocalExporter {
    async fn export(&self, example: LabeledExample) -> Result<(), ExportError> {
        self.export_batch(vec![example]).await
    }

    async fn export_batch(&self, examples: Vec<LabeledExample>) -> Result<(), ExportError> {
        if examples.is_empty() {
            return Ok(());
        }

        let partitions = self.partition_examples(&examples);
        let output_dir = self.output_dir.clone();

        for (partition_path, partition_examples) in partitions {
            let path = partition_path.to_path(&output_dir);

            let file_path = self.write_examples_sync(&path, &partition_examples)?;

            tracing::info!(
                path = %file_path.display(),
                count = partition_examples.len(),
                format = ?self.format,
                "Exported training examples"
            );
        }

        Ok(())
    }

    async fn flush(&self) -> Result<(), ExportError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use flywheel_ml_core::GroundTruth;
    use std::collections::HashMap as StdHashMap;
    use tempfile::tempdir;

    fn make_test_example(model_id: &str) -> LabeledExample {
        LabeledExample {
            example_id: uuid::Uuid::new_v4().to_string(),
            prediction_id: "pred-1".to_string(),
            model_id: model_id.to_string(),
            model_version: "v1".to_string(),
            features: serde_json::json!({"cpu": 0.85}),
            prediction: serde_json::json!({"score": 0.9}),
            ground_truth: GroundTruth::Binary(true),
            prediction_timestamp: Utc::now(),
            feedback_timestamp: Utc::now(),
            delay_ms: 1000,
            feedback_confidence: 0.95,
            is_correct: Some(true),
            metadata: StdHashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_local_exporter_jsonlines() {
        let dir = tempdir().unwrap();
        let exporter = LocalExporter::new(dir.path(), ExportFormat::JsonLines);

        let examples = vec![
            make_test_example("model-a"),
            make_test_example("model-b"),
            make_test_example("model-a"),
        ];

        exporter.export_batch(examples).await.unwrap();

        let model_a_dir = dir.path().join("model_id=model-a");
        assert!(model_a_dir.exists());

        let model_b_dir = dir.path().join("model_id=model-b");
        assert!(model_b_dir.exists());
    }

    #[tokio::test]
    async fn test_local_exporter_csv() {
        let dir = tempdir().unwrap();
        let exporter = LocalExporter::new(dir.path(), ExportFormat::Csv);

        let examples = vec![make_test_example("model-csv")];
        exporter.export_batch(examples).await.unwrap();

        let entries: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert!(!entries.is_empty());
    }

    #[tokio::test]
    async fn test_local_exporter_parquet() {
        let dir = tempdir().unwrap();
        let exporter = LocalExporter::new(dir.path(), ExportFormat::Parquet);

        let examples = vec![make_test_example("model-parquet")];
        exporter.export_batch(examples).await.unwrap();

        let entries: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert!(!entries.is_empty());
    }

    #[tokio::test]
    async fn test_partition_by_model_and_date() {
        let dir = tempdir().unwrap();
        let exporter = LocalExporter::new(dir.path(), ExportFormat::JsonLines)
            .with_partitions(vec![PartitionKey::ModelId, PartitionKey::Date]);

        let examples = vec![make_test_example("model-x")];
        exporter.export_batch(examples).await.unwrap();

        let model_dir = dir.path().join("model_id=model-x");
        assert!(model_dir.exists());

        let date_dirs: Vec<_> = std::fs::read_dir(&model_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert!(!date_dirs.is_empty());
    }
}
