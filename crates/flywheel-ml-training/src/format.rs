use arrow::array::{ArrayRef, BooleanArray, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use flywheel_ml_core::LabeledExample;
use parquet::arrow::ArrowWriter;
use std::io::Write;
use std::sync::Arc;

pub trait FormatWriter {
    fn write(&mut self, example: &LabeledExample) -> Result<(), std::io::Error>;
    fn flush(&mut self) -> Result<(), std::io::Error>;
}

pub struct JsonLinesWriter<W: Write> {
    writer: W,
}

impl<W: Write> JsonLinesWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}

impl<W: Write> FormatWriter for JsonLinesWriter<W> {
    fn write(&mut self, example: &LabeledExample) -> Result<(), std::io::Error> {
        let json = serde_json::to_string(example)?;
        writeln!(self.writer, "{}", json)
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        self.writer.flush()
    }
}

pub struct CsvWriter<W: Write> {
    writer: csv::Writer<W>,
    headers_written: bool,
}

impl<W: Write> CsvWriter<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer: csv::Writer::from_writer(writer),
            headers_written: false,
        }
    }
}

impl<W: Write> FormatWriter for CsvWriter<W> {
    fn write(&mut self, example: &LabeledExample) -> Result<(), std::io::Error> {
        if !self.headers_written {
            self.writer
                .write_record([
                    "example_id",
                    "prediction_id",
                    "model_id",
                    "model_version",
                    "features",
                    "prediction",
                    "ground_truth",
                    "prediction_timestamp",
                    "feedback_timestamp",
                    "delay_ms",
                    "feedback_confidence",
                    "is_correct",
                ])
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            self.headers_written = true;
        }

        self.writer
            .write_record([
                &example.example_id,
                &example.prediction_id,
                &example.model_id,
                &example.model_version,
                &serde_json::to_string(&example.features).unwrap_or_default(),
                &serde_json::to_string(&example.prediction).unwrap_or_default(),
                &serde_json::to_string(&example.ground_truth).unwrap_or_default(),
                &example.prediction_timestamp.to_rfc3339(),
                &example.feedback_timestamp.to_rfc3339(),
                &example.delay_ms.to_string(),
                &example.feedback_confidence.to_string(),
                &example
                    .is_correct
                    .map(|b| b.to_string())
                    .unwrap_or_default(),
            ])
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        self.writer
            .flush()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

pub struct ParquetBatchWriter<W: Write + Send> {
    writer: Option<ArrowWriter<W>>,
    buffer: Vec<LabeledExample>,
    batch_size: usize,
    schema: Arc<Schema>,
}

impl<W: Write + Send> ParquetBatchWriter<W> {
    pub fn new(writer: W, batch_size: usize) -> Result<Self, std::io::Error> {
        let schema = Arc::new(Self::schema());
        let arrow_writer = ArrowWriter::try_new(writer, schema.clone(), None)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(Self {
            writer: Some(arrow_writer),
            buffer: Vec::with_capacity(batch_size),
            batch_size,
            schema,
        })
    }

    fn schema() -> Schema {
        Schema::new(vec![
            Field::new("example_id", DataType::Utf8, false),
            Field::new("prediction_id", DataType::Utf8, false),
            Field::new("model_id", DataType::Utf8, false),
            Field::new("model_version", DataType::Utf8, false),
            Field::new("features", DataType::Utf8, false),
            Field::new("prediction", DataType::Utf8, false),
            Field::new("ground_truth", DataType::Utf8, false),
            Field::new("prediction_timestamp", DataType::Utf8, false),
            Field::new("feedback_timestamp", DataType::Utf8, false),
            Field::new("delay_ms", DataType::Int64, false),
            Field::new("feedback_confidence", DataType::Utf8, false),
            Field::new("is_correct", DataType::Boolean, true),
        ])
    }

    fn flush_batch(&mut self) -> Result<(), std::io::Error> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        let batch = self.to_record_batch()?;

        if let Some(writer) = self.writer.as_mut() {
            writer
                .write(&batch)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        }

        self.buffer.clear();
        Ok(())
    }

    fn to_record_batch(&self) -> Result<RecordBatch, std::io::Error> {
        let example_ids: Vec<&str> = self.buffer.iter().map(|e| e.example_id.as_str()).collect();
        let prediction_ids: Vec<&str> = self
            .buffer
            .iter()
            .map(|e| e.prediction_id.as_str())
            .collect();
        let model_ids: Vec<&str> = self.buffer.iter().map(|e| e.model_id.as_str()).collect();
        let model_versions: Vec<&str> = self
            .buffer
            .iter()
            .map(|e| e.model_version.as_str())
            .collect();
        let features: Vec<String> = self
            .buffer
            .iter()
            .map(|e| serde_json::to_string(&e.features).unwrap_or_default())
            .collect();
        let predictions: Vec<String> = self
            .buffer
            .iter()
            .map(|e| serde_json::to_string(&e.prediction).unwrap_or_default())
            .collect();
        let ground_truths: Vec<String> = self
            .buffer
            .iter()
            .map(|e| serde_json::to_string(&e.ground_truth).unwrap_or_default())
            .collect();
        let prediction_timestamps: Vec<String> = self
            .buffer
            .iter()
            .map(|e| e.prediction_timestamp.to_rfc3339())
            .collect();
        let feedback_timestamps: Vec<String> = self
            .buffer
            .iter()
            .map(|e| e.feedback_timestamp.to_rfc3339())
            .collect();
        let delay_ms: Vec<i64> = self.buffer.iter().map(|e| e.delay_ms as i64).collect();
        let feedback_confidences: Vec<String> = self
            .buffer
            .iter()
            .map(|e| e.feedback_confidence.to_string())
            .collect();
        let is_correct: Vec<Option<bool>> = self.buffer.iter().map(|e| e.is_correct).collect();

        let columns: Vec<ArrayRef> = vec![
            Arc::new(StringArray::from(example_ids)),
            Arc::new(StringArray::from(prediction_ids)),
            Arc::new(StringArray::from(model_ids)),
            Arc::new(StringArray::from(model_versions)),
            Arc::new(StringArray::from(features.iter().map(|s| s.as_str()).collect::<Vec<_>>())),
            Arc::new(StringArray::from(predictions.iter().map(|s| s.as_str()).collect::<Vec<_>>())),
            Arc::new(StringArray::from(ground_truths.iter().map(|s| s.as_str()).collect::<Vec<_>>())),
            Arc::new(StringArray::from(prediction_timestamps.iter().map(|s| s.as_str()).collect::<Vec<_>>())),
            Arc::new(StringArray::from(feedback_timestamps.iter().map(|s| s.as_str()).collect::<Vec<_>>())),
            Arc::new(Int64Array::from(delay_ms)),
            Arc::new(StringArray::from(feedback_confidences.iter().map(|s| s.as_str()).collect::<Vec<_>>())),
            Arc::new(BooleanArray::from(is_correct)),
        ];

        RecordBatch::try_new(self.schema.clone(), columns)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

impl<W: Write + Send> FormatWriter for ParquetBatchWriter<W> {
    fn write(&mut self, example: &LabeledExample) -> Result<(), std::io::Error> {
        self.buffer.push(example.clone());
        if self.buffer.len() >= self.batch_size {
            self.flush_batch()?;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        self.flush_batch()?;
        if let Some(writer) = self.writer.take() {
            writer
                .close()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use flywheel_ml_core::GroundTruth;
    use std::collections::HashMap;

    fn make_test_example() -> LabeledExample {
        LabeledExample {
            example_id: "ex-1".to_string(),
            prediction_id: "pred-1".to_string(),
            model_id: "model-1".to_string(),
            model_version: "v1".to_string(),
            features: serde_json::json!({"cpu": 0.85, "memory": 0.72}),
            prediction: serde_json::json!({"type": "anomaly", "score": 0.9}),
            ground_truth: GroundTruth::Binary(true),
            prediction_timestamp: Utc::now(),
            feedback_timestamp: Utc::now(),
            delay_ms: 1000,
            feedback_confidence: 0.95,
            is_correct: Some(true),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_jsonlines_writer() {
        let mut buffer = Vec::new();
        let mut writer = JsonLinesWriter::new(&mut buffer);
        let example = make_test_example();

        writer.write(&example).unwrap();
        writer.flush().unwrap();

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("ex-1"));
        assert!(output.contains("model-1"));
    }

    #[test]
    fn test_csv_writer() {
        let mut buffer = Vec::new();
        let mut writer = CsvWriter::new(&mut buffer);
        let example = make_test_example();

        writer.write(&example).unwrap();
        writer.flush().unwrap();

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("example_id"));
        assert!(output.contains("ex-1"));
    }

    #[test]
    fn test_parquet_writer() {
        let mut buffer = Vec::new();
        let mut writer = ParquetBatchWriter::new(&mut buffer, 100).unwrap();
        let example = make_test_example();

        writer.write(&example).unwrap();
        writer.flush().unwrap();

        assert!(!buffer.is_empty());
    }
}
