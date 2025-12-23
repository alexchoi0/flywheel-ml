use std::collections::HashMap;
use std::pin::Pin;

use chrono::Utc;
use flywheel_ml_db::{Database, ModelVersionRepo, PredictionRepo};
use flywheel_ml_proto::inference_service_server::InferenceService;
use flywheel_ml_proto::{
    prediction_result, AnomalyResult, BatchStats, HealthCheckRequest, HealthCheckResponse,
    ModelInfoRequest, ModelInfoResponse, PredictBatchRequest, PredictBatchResponse,
    PredictRequest, PredictResponse, PredictionResult,
};
use prost_types::Timestamp;
use tokio_stream::Stream;
use tonic::{Request, Response, Status, Streaming};
use uuid::Uuid;

pub struct InferenceServiceImpl {
    db: Database,
}

impl InferenceServiceImpl {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    fn now_timestamp() -> Option<Timestamp> {
        let now = Utc::now();
        Some(Timestamp {
            seconds: now.timestamp(),
            nanos: now.timestamp_subsec_nanos() as i32,
        })
    }
}

#[tonic::async_trait]
impl InferenceService for InferenceServiceImpl {
    async fn predict(
        &self,
        request: Request<PredictRequest>,
    ) -> Result<Response<PredictResponse>, Status> {
        let req = request.into_inner();
        let prediction_id = Uuid::new_v4().to_string();

        let model = ModelVersionRepo::find_by_model_id(self.db.conn(), &req.model_id)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| Status::not_found(format!("Model not found: {}", req.model_id)))?;

        let features_json = serde_json::json!({
            "feature_count": req.features.len(),
        });

        let anomaly_score = 0.3;
        let is_anomaly = anomaly_score > 0.5;

        let prediction_json = serde_json::json!({
            "score": anomaly_score,
            "is_anomaly": is_anomaly,
            "model_version": model.version,
        });

        let prediction = PredictionRepo::create(
            self.db.conn(),
            Uuid::nil(),
            req.model_id.clone(),
            model.version.clone(),
            features_json,
            prediction_json,
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to store prediction: {}", e)))?;

        tracing::debug!(
            model_id = %req.model_id,
            prediction_id = %prediction.id,
            "Prediction made"
        );

        let response = PredictResponse {
            prediction_id: prediction.id.to_string(),
            model_id: req.model_id,
            model_version: model.version,
            result: Some(PredictionResult {
                result: Some(prediction_result::Result::Anomaly(AnomalyResult {
                    score: anomaly_score,
                    is_anomaly,
                    threshold: 0.5,
                    contributing_features: vec![],
                })),
            }),
            confidence: 0.95,
            latency_us: 100,
            timestamp: Self::now_timestamp(),
        };

        Ok(Response::new(response))
    }

    async fn predict_batch(
        &self,
        request: Request<PredictBatchRequest>,
    ) -> Result<Response<PredictBatchResponse>, Status> {
        let req = request.into_inner();
        let mut responses = Vec::with_capacity(req.requests.len());
        let mut succeeded = 0u32;
        let mut failed = 0u32;

        for predict_req in req.requests {
            let inner_request = Request::new(predict_req);
            match self.predict(inner_request).await {
                Ok(resp) => {
                    responses.push(resp.into_inner());
                    succeeded += 1;
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Batch prediction failed for one request");
                    failed += 1;
                }
            }
        }

        let response = PredictBatchResponse {
            batch_id: req.batch_id,
            responses,
            stats: Some(BatchStats {
                total: succeeded + failed,
                succeeded,
                failed,
                avg_latency_us: 100,
                p99_latency_us: 200,
            }),
        };

        Ok(Response::new(response))
    }

    type PredictStreamStream =
        Pin<Box<dyn Stream<Item = Result<PredictResponse, Status>> + Send + 'static>>;

    async fn predict_stream(
        &self,
        request: Request<Streaming<PredictRequest>>,
    ) -> Result<Response<Self::PredictStreamStream>, Status> {
        let mut stream = request.into_inner();
        let db = self.db.clone();

        let output = async_stream::try_stream! {
            while let Some(req) = stream.message().await? {
                let model = ModelVersionRepo::find_by_model_id(db.conn(), &req.model_id)
                    .await
                    .map_err(|e| Status::internal(format!("Database error: {}", e)))?
                    .ok_or_else(|| Status::not_found(format!("Model not found: {}", req.model_id)))?;

                let anomaly_score = 0.3;
                let is_anomaly = anomaly_score > 0.5;

                yield PredictResponse {
                    prediction_id: Uuid::new_v4().to_string(),
                    model_id: req.model_id,
                    model_version: model.version,
                    result: Some(PredictionResult {
                        result: Some(prediction_result::Result::Anomaly(AnomalyResult {
                            score: anomaly_score,
                            is_anomaly,
                            threshold: 0.5,
                            contributing_features: vec![],
                        })),
                    }),
                    confidence: 0.95,
                    latency_us: 100,
                    timestamp: InferenceServiceImpl::now_timestamp(),
                };
            }
        };

        Ok(Response::new(Box::pin(output)))
    }

    async fn get_model_info(
        &self,
        request: Request<ModelInfoRequest>,
    ) -> Result<Response<ModelInfoResponse>, Status> {
        let req = request.into_inner();

        let model = ModelVersionRepo::find_by_model_id(self.db.conn(), &req.model_id)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| Status::not_found(format!("Model not found: {}", req.model_id)))?;

        let response = ModelInfoResponse {
            model_id: model.model_id,
            model_name: String::new(),
            version: model.version,
            model_type: model.model_type,
            input_features: vec![],
            output_field: "prediction".to_string(),
            labels: HashMap::new(),
        };

        Ok(Response::new(response))
    }

    async fn health_check(
        &self,
        request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        let req = request.into_inner();

        let (status, message) = if req.model_id.is_empty() {
            ("healthy".to_string(), "Inference service is ready".to_string())
        } else {
            match ModelVersionRepo::find_by_model_id(self.db.conn(), &req.model_id).await {
                Ok(Some(_)) => ("healthy".to_string(), "Model is ready for inference".to_string()),
                Ok(None) => ("not_found".to_string(), "Model not found".to_string()),
                Err(e) => ("error".to_string(), format!("Database error: {}", e)),
            }
        };

        let response = HealthCheckResponse {
            status,
            latency_ms: 1,
            error_rate: 0.0,
            message,
        };

        Ok(Response::new(response))
    }
}
