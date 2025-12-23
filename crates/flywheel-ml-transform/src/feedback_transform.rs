use flywheel_ml_core::{
    FeedbackError, FeedbackRecord, LabeledExample, Prediction, PredictionResult, StoredPrediction,
};
use flywheel_ml_db::{entity, PredictionRepo};
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::sync::Arc;

pub struct FeedbackJoinTransform {
    db: Arc<DatabaseConnection>,
    max_join_delay_secs: i64,
}

impl FeedbackJoinTransform {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            db,
            max_join_delay_secs: 86400,
        }
    }

    pub fn with_max_delay(mut self, secs: i64) -> Self {
        self.max_join_delay_secs = secs;
        self
    }

    pub async fn process(
        &self,
        feedback: FeedbackRecord,
    ) -> Result<Option<LabeledExample>, FeedbackError> {
        let prediction_uuid = uuid::Uuid::parse_str(&feedback.prediction_id).map_err(|_| {
            FeedbackError::PredictionNotFound(format!(
                "Invalid prediction ID format: {}",
                feedback.prediction_id
            ))
        })?;

        let prediction_model = PredictionRepo::find_by_id(&self.db, prediction_uuid)
            .await
            .map_err(|e| FeedbackError::JoinFailed(format!("Database error: {}", e)))?
            .ok_or_else(|| FeedbackError::PredictionNotFound(feedback.prediction_id.clone()))?;

        let stored = self.convert_to_stored_prediction(prediction_model)?;

        if stored.is_expired() {
            tracing::debug!(
                prediction_id = %feedback.prediction_id,
                "Prediction expired, skipping feedback join"
            );
            return Ok(None);
        }

        let delay_secs = feedback
            .feedback_time
            .signed_duration_since(stored.prediction.timestamp)
            .num_seconds();

        if delay_secs > self.max_join_delay_secs {
            tracing::debug!(
                prediction_id = %feedback.prediction_id,
                delay_secs = delay_secs,
                max_delay_secs = self.max_join_delay_secs,
                "Feedback received after max join delay, skipping"
            );
            return Ok(None);
        }

        let labeled = LabeledExample::from_prediction_and_feedback(&stored, &feedback);

        tracing::info!(
            example_id = %labeled.example_id,
            prediction_id = %labeled.prediction_id,
            model_id = %labeled.model_id,
            is_correct = ?labeled.is_correct,
            "Created labeled example from feedback join"
        );

        Ok(Some(labeled))
    }

    pub async fn process_batch(
        &self,
        feedbacks: Vec<FeedbackRecord>,
    ) -> Vec<Result<Option<LabeledExample>, FeedbackError>> {
        let mut results = Vec::with_capacity(feedbacks.len());
        for feedback in feedbacks {
            results.push(self.process(feedback).await);
        }
        results
    }

    fn convert_to_stored_prediction(
        &self,
        model: entity::prediction::Model,
    ) -> Result<StoredPrediction, FeedbackError> {
        let prediction_result: PredictionResult =
            serde_json::from_value(model.prediction_json.clone()).map_err(|e| {
                FeedbackError::JoinFailed(format!("Invalid prediction JSON: {}", e))
            })?;

        let prediction = Prediction {
            prediction_id: model.id.to_string(),
            model_id: model.model_id,
            model_version: model.model_version,
            timestamp: model.created_at,
            result: prediction_result,
            confidence: extract_confidence(&model.prediction_json),
            latency_us: 0,
            features_hash: String::new(),
            metadata: HashMap::new(),
        };

        Ok(StoredPrediction {
            prediction,
            features: model.features_json,
            source_record_id: model.id.to_string(),
            stored_at: model.created_at,
            ttl_seconds: None,
        })
    }
}

fn extract_confidence(prediction_json: &serde_json::Value) -> Option<f64> {
    prediction_json.get("confidence").and_then(|v| v.as_f64())
}

impl Default for FeedbackJoinTransform {
    fn default() -> Self {
        panic!("FeedbackJoinTransform requires a database connection. Use FeedbackJoinTransform::new(db) instead.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flywheel_ml_core::{FeedbackSource, GroundTruth};

    fn make_test_feedback(prediction_id: &str) -> FeedbackRecord {
        FeedbackRecord::new(
            prediction_id,
            "test-model",
            GroundTruth::Binary(true),
            FeedbackSource::explicit("user-1", "confirm"),
        )
    }

    #[test]
    fn test_extract_confidence() {
        let json = serde_json::json!({
            "type": "anomaly",
            "score": 0.85,
            "is_anomaly": true,
            "threshold": 0.7,
            "confidence": 0.92
        });
        assert_eq!(extract_confidence(&json), Some(0.92));

        let json_no_conf = serde_json::json!({
            "type": "anomaly",
            "score": 0.85
        });
        assert_eq!(extract_confidence(&json_no_conf), None);
    }
}
