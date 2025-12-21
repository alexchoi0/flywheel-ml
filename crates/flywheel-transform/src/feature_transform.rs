use flywheel_core::{FeatureError, FeatureExtractor, FeatureVector};

pub struct FeatureExtractionTransform {
    extractor: Box<dyn FeatureExtractor>,
}

impl FeatureExtractionTransform {
    pub fn new(extractor: Box<dyn FeatureExtractor>) -> Self {
        Self { extractor }
    }

    pub async fn process(&self, record: &serde_json::Value) -> Result<FeatureVector, FeatureError> {
        self.extractor.extract(record).await
    }

    pub async fn process_batch(
        &self,
        records: &[serde_json::Value],
    ) -> Result<Vec<FeatureVector>, FeatureError> {
        self.extractor.extract_batch(records).await
    }
}
