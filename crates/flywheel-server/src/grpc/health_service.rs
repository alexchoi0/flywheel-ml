use flywheel_ml_db::Database;
use flywheel_ml_proto::health_service_server::HealthService;
use flywheel_ml_proto::{
    GetHealthRequest, GetHealthResponse,
    GetPipelineHealthRequest, GetPipelineHealthResponse,
    GetDriftStatusRequest, GetDriftStatusResponse,
    ListDriftEventsRequest, ListDriftEventsResponse,
    DatabaseHealth,
};
use tonic::{Request, Response, Status};

pub struct HealthServiceImpl {
    _db: Database,
}

impl HealthServiceImpl {
    pub fn new(db: Database) -> Self {
        Self { _db: db }
    }
}

#[tonic::async_trait]
impl HealthService for HealthServiceImpl {
    async fn get_health(
        &self,
        _request: Request<GetHealthRequest>,
    ) -> Result<Response<GetHealthResponse>, Status> {
        let response = GetHealthResponse {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_since: None,
            active_pipelines: 0,
            registered_models: 0,
            database: Some(DatabaseHealth {
                connected: true,
                latency_ms: 1,
                active_connections: 1,
            }),
        };
        Ok(Response::new(response))
    }

    async fn get_pipeline_health(
        &self,
        _request: Request<GetPipelineHealthRequest>,
    ) -> Result<Response<GetPipelineHealthResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn get_drift_status(
        &self,
        _request: Request<GetDriftStatusRequest>,
    ) -> Result<Response<GetDriftStatusResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn list_drift_events(
        &self,
        _request: Request<ListDriftEventsRequest>,
    ) -> Result<Response<ListDriftEventsResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }
}
