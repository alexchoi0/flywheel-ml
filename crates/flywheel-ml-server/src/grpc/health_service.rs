use chrono::Utc;
use flywheel_ml_db::{Database, DriftEventRepo, PipelineRepo};
use flywheel_ml_proto::health_service_server::HealthService;
use flywheel_ml_proto::{
    DatabaseHealth, DriftEvent, DriftSummary, GetDriftStatusRequest, GetDriftStatusResponse,
    GetHealthRequest, GetHealthResponse, GetPipelineHealthRequest, GetPipelineHealthResponse,
    ListDriftEventsRequest, ListDriftEventsResponse, PipelineMetrics, PerformanceDrift,
    StatisticalDrift, StageHealth,
};
use prost_types::Timestamp;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct HealthServiceImpl {
    db: Database,
    start_time: Instant,
    pipeline_count: Arc<AtomicU32>,
    model_count: Arc<AtomicU32>,
}

impl HealthServiceImpl {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            start_time: Instant::now(),
            pipeline_count: Arc::new(AtomicU32::new(0)),
            model_count: Arc::new(AtomicU32::new(0)),
        }
    }

    fn datetime_to_timestamp(dt: chrono::DateTime<Utc>) -> Option<Timestamp> {
        Some(Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        })
    }
}

#[tonic::async_trait]
impl HealthService for HealthServiceImpl {
    async fn get_health(
        &self,
        _request: Request<GetHealthRequest>,
    ) -> Result<Response<GetHealthResponse>, Status> {
        let db_start = Instant::now();
        let db_connected = self.db.conn().ping().await.is_ok();
        let db_latency = db_start.elapsed().as_millis() as u64;

        let uptime_since = Utc::now() - chrono::Duration::from_std(self.start_time.elapsed())
            .unwrap_or_default();

        let pipelines = PipelineRepo::list(self.db.conn(), None, 1000, 0)
            .await
            .unwrap_or_default();

        let response = GetHealthResponse {
            status: if db_connected { "healthy" } else { "degraded" }.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_since: Self::datetime_to_timestamp(uptime_since),
            active_pipelines: pipelines.len() as i32,
            registered_models: self.model_count.load(Ordering::Relaxed) as i32,
            database: Some(DatabaseHealth {
                connected: db_connected,
                latency_ms: db_latency,
                active_connections: 1,
            }),
        };

        Ok(Response::new(response))
    }

    async fn get_pipeline_health(
        &self,
        request: Request<GetPipelineHealthRequest>,
    ) -> Result<Response<GetPipelineHealthResponse>, Status> {
        let req = request.into_inner();
        let pipeline_id = Uuid::parse_str(&req.pipeline_id)
            .map_err(|_| Status::invalid_argument("Invalid pipeline ID format"))?;

        let pipeline = PipelineRepo::find_by_id(self.db.conn(), pipeline_id)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| Status::not_found("Pipeline not found"))?;

        let drift_events = DriftEventRepo::list_by_pipeline(self.db.conn(), pipeline_id, 1)
            .await
            .unwrap_or_default();

        let drift_summary = drift_events.first().map(|event| DriftSummary {
            is_drifted: event.resolved_at.is_none(),
            severity: format!("{:?}", event.severity),
            psi_score: event.psi_score.unwrap_or(0.0),
            kl_divergence: event.kl_divergence.unwrap_or(0.0),
            accuracy_delta: event.accuracy_delta.unwrap_or(0.0),
            last_checked: Self::datetime_to_timestamp(event.detected_at),
        });

        let response = GetPipelineHealthResponse {
            pipeline_id: pipeline.id.to_string(),
            status: format!("{:?}", pipeline.status),
            metrics: Some(PipelineMetrics {
                records_per_second: 0.0,
                predictions_per_second: 0.0,
                error_rate: 0.0,
                avg_latency_ms: 0,
                p99_latency_ms: 0,
                current_accuracy: 0.0,
                feedback_rate: 0.0,
            }),
            stages: vec![],
            drift: drift_summary,
        };

        Ok(Response::new(response))
    }

    async fn get_drift_status(
        &self,
        request: Request<GetDriftStatusRequest>,
    ) -> Result<Response<GetDriftStatusResponse>, Status> {
        let req = request.into_inner();
        let pipeline_id = Uuid::parse_str(&req.pipeline_id)
            .map_err(|_| Status::invalid_argument("Invalid pipeline ID format"))?;

        let drift_events = DriftEventRepo::list_by_pipeline(self.db.conn(), pipeline_id, 1)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?;

        let event = drift_events.first();

        let response = GetDriftStatusResponse {
            pipeline_id: req.pipeline_id,
            model_id: req.model_id,
            is_drifted: event.map(|e| e.resolved_at.is_none()).unwrap_or(false),
            drift_type: event.map(|e| format!("{:?}", e.drift_type)).unwrap_or_default(),
            severity: event.map(|e| format!("{:?}", e.severity)).unwrap_or_default(),
            statistical: event.map(|e| StatisticalDrift {
                psi_score: e.psi_score.unwrap_or(0.0),
                kl_divergence: e.kl_divergence.unwrap_or(0.0),
                feature_drifts: vec![],
            }),
            performance: event.map(|e| PerformanceDrift {
                accuracy: 0.0,
                accuracy_baseline: 0.0,
                accuracy_delta: e.accuracy_delta.unwrap_or(0.0),
                precision: 0.0,
                recall: 0.0,
                latency_p99_ms: 0,
                error_rate: 0.0,
            }),
            detected_at: event.and_then(|e| Self::datetime_to_timestamp(e.detected_at)),
        };

        Ok(Response::new(response))
    }

    async fn list_drift_events(
        &self,
        request: Request<ListDriftEventsRequest>,
    ) -> Result<Response<ListDriftEventsResponse>, Status> {
        let req = request.into_inner();
        let pipeline_id = Uuid::parse_str(&req.pipeline_id)
            .map_err(|_| Status::invalid_argument("Invalid pipeline ID format"))?;

        let limit = if req.limit > 0 { req.limit as u64 } else { 100 };

        let events = DriftEventRepo::list_by_pipeline(self.db.conn(), pipeline_id, limit)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?;

        let proto_events: Vec<DriftEvent> = events
            .into_iter()
            .map(|e| DriftEvent {
                event_id: e.id.to_string(),
                pipeline_id: e.pipeline_id.to_string(),
                model_id: e.model_id,
                drift_type: format!("{:?}", e.drift_type),
                severity: format!("{:?}", e.severity),
                psi_score: e.psi_score.unwrap_or(0.0),
                kl_divergence: e.kl_divergence.unwrap_or(0.0),
                accuracy_delta: e.accuracy_delta.unwrap_or(0.0),
                detected_at: Self::datetime_to_timestamp(e.detected_at),
                resolved_at: e.resolved_at.and_then(Self::datetime_to_timestamp),
            })
            .collect();

        let response = ListDriftEventsResponse {
            events: proto_events,
            next_cursor: String::new(),
        };

        Ok(Response::new(response))
    }
}
