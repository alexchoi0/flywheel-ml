use flywheel_ml_core::{FeedbackError, FeedbackRecord, LabeledExample};

pub struct FeedbackJoinTransform {
    // TODO: Add prediction store
}

impl FeedbackJoinTransform {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn process(
        &self,
        _feedback: FeedbackRecord,
    ) -> Result<Option<LabeledExample>, FeedbackError> {
        // TODO: Lookup prediction from store and create labeled example
        Err(FeedbackError::PredictionNotFound("Not implemented".to_string()))
    }
}

impl Default for FeedbackJoinTransform {
    fn default() -> Self {
        Self::new()
    }
}
