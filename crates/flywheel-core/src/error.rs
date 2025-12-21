use thiserror::Error;

#[derive(Error, Debug)]
pub enum FlywheelError {
    #[error("Model error: {0}")]
    Model(#[from] ModelError),

    #[error("Feature extraction error: {0}")]
    Feature(#[from] FeatureError),

    #[error("Prediction error: {0}")]
    Prediction(#[from] PredictionError),

    #[error("Feedback error: {0}")]
    Feedback(#[from] FeedbackError),

    #[error("Drift detection error: {0}")]
    Drift(#[from] DriftError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Model not found: {0}")]
    NotFound(String),

    #[error("Model inference failed: {0}")]
    InferenceFailed(String),

    #[error("Model timeout after {0}ms")]
    Timeout(u64),

    #[error("Model unavailable: {0}")]
    Unavailable(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Circuit breaker open for model: {0}")]
    CircuitBreakerOpen(String),

    #[error("Connection error: {0}")]
    Connection(String),
}

#[derive(Error, Debug)]
pub enum FeatureError {
    #[error("Feature not found: {0}")]
    NotFound(String),

    #[error("Invalid feature value for '{feature}': {reason}")]
    InvalidValue { feature: String, reason: String },

    #[error("Feature extraction failed: {0}")]
    ExtractionFailed(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Type mismatch for feature '{feature}': expected {expected}, got {actual}")]
    TypeMismatch {
        feature: String,
        expected: String,
        actual: String,
    },

    #[error("JSON path error: {0}")]
    JsonPath(String),
}

#[derive(Error, Debug)]
pub enum PredictionError {
    #[error("Prediction not found: {0}")]
    NotFound(String),

    #[error("Prediction storage failed: {0}")]
    StorageFailed(String),

    #[error("Invalid prediction result: {0}")]
    InvalidResult(String),
}

#[derive(Error, Debug)]
pub enum FeedbackError {
    #[error("Feedback not found: {0}")]
    NotFound(String),

    #[error("Prediction not found for feedback: {0}")]
    PredictionNotFound(String),

    #[error("Missing prediction ID in feedback")]
    MissingPredictionId,

    #[error("Invalid ground truth: {0}")]
    InvalidGroundTruth(String),

    #[error("Feedback join failed: {0}")]
    JoinFailed(String),

    #[error("Feedback storage failed: {0}")]
    StorageFailed(String),
}

#[derive(Error, Debug)]
pub enum DriftError {
    #[error("Insufficient samples: need {required}, have {actual}")]
    InsufficientSamples { required: usize, actual: usize },

    #[error("Baseline not found: {0}")]
    BaselineNotFound(String),

    #[error("Invalid threshold: {0}")]
    InvalidThreshold(String),

    #[error("Drift detection failed: {0}")]
    DetectionFailed(String),
}

pub type Result<T> = std::result::Result<T, FlywheelError>;
