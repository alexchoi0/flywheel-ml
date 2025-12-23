use chrono::Utc;
use flywheel_ml_db::{entity::pipeline, Database, ModelVersionRepo, PipelineRepo};
use flywheel_ml_proto::control_service_server::ControlService;
use flywheel_ml_proto::{
    CreatePipelineRequest, CreatePipelineResponse, DeletePipelineRequest, DeletePipelineResponse,
    DisablePipelineRequest, DisablePipelineResponse, EnablePipelineRequest, EnablePipelineResponse,
    GetModelRequest, GetModelResponse, GetPipelineRequest, GetPipelineResponse,
    ListModelsRequest, ListModelsResponse, ListPipelinesRequest, ListPipelinesResponse,
    ModelInfo, PipelineInfo, PipelineStats, RegisterModelRequest, RegisterModelResponse,
    UnregisterModelRequest, UnregisterModelResponse, UpdatePipelineRequest, UpdatePipelineResponse,
};
use prost_types::Timestamp;
use sha2::{Digest, Sha256};
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct ControlServiceImpl {
    db: Database,
}

impl ControlServiceImpl {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    fn hash_spec(spec: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(spec.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn datetime_to_timestamp(dt: chrono::DateTime<Utc>) -> Option<Timestamp> {
        Some(Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        })
    }
}

#[tonic::async_trait]
impl ControlService for ControlServiceImpl {
    async fn create_pipeline(
        &self,
        request: Request<CreatePipelineRequest>,
    ) -> Result<Response<CreatePipelineResponse>, Status> {
        let req = request.into_inner();

        if req.name.is_empty() {
            return Err(Status::invalid_argument("Pipeline name is required"));
        }

        let spec_hash = Self::hash_spec(&req.spec_yaml);

        let pipeline = PipelineRepo::create(
            self.db.conn(),
            req.name.clone(),
            req.namespace.clone(),
            spec_hash,
            req.spec_yaml,
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to create pipeline: {}", e)))?;

        tracing::info!(
            pipeline_id = %pipeline.id,
            name = %pipeline.name,
            namespace = %pipeline.namespace,
            "Pipeline created"
        );

        let response = CreatePipelineResponse {
            pipeline_id: pipeline.id.to_string(),
            name: pipeline.name,
            status: format!("{:?}", pipeline.status),
        };

        Ok(Response::new(response))
    }

    async fn update_pipeline(
        &self,
        request: Request<UpdatePipelineRequest>,
    ) -> Result<Response<UpdatePipelineResponse>, Status> {
        let req = request.into_inner();
        let pipeline_id = Uuid::parse_str(&req.pipeline_id)
            .map_err(|_| Status::invalid_argument("Invalid pipeline ID format"))?;

        let pipeline = PipelineRepo::find_by_id(self.db.conn(), pipeline_id)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| Status::not_found("Pipeline not found"))?;

        let response = UpdatePipelineResponse {
            pipeline_id: pipeline.id.to_string(),
            status: format!("{:?}", pipeline.status),
            version: "1".to_string(),
        };

        Ok(Response::new(response))
    }

    async fn delete_pipeline(
        &self,
        request: Request<DeletePipelineRequest>,
    ) -> Result<Response<DeletePipelineResponse>, Status> {
        let req = request.into_inner();
        let pipeline_id = Uuid::parse_str(&req.pipeline_id)
            .map_err(|_| Status::invalid_argument("Invalid pipeline ID format"))?;

        let result = PipelineRepo::delete(self.db.conn(), pipeline_id)
            .await
            .map_err(|e| Status::internal(format!("Failed to delete pipeline: {}", e)))?;

        tracing::info!(pipeline_id = %pipeline_id, "Pipeline deleted");

        let response = DeletePipelineResponse {
            success: result.rows_affected > 0,
        };

        Ok(Response::new(response))
    }

    async fn get_pipeline(
        &self,
        request: Request<GetPipelineRequest>,
    ) -> Result<Response<GetPipelineResponse>, Status> {
        let req = request.into_inner();
        let pipeline_id = Uuid::parse_str(&req.pipeline_id)
            .map_err(|_| Status::invalid_argument("Invalid pipeline ID format"))?;

        let pipeline = PipelineRepo::find_by_id(self.db.conn(), pipeline_id)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| Status::not_found("Pipeline not found"))?;

        let response = GetPipelineResponse {
            pipeline: Some(PipelineInfo {
                pipeline_id: pipeline.id.to_string(),
                name: pipeline.name,
                namespace: pipeline.namespace,
                status: format!("{:?}", pipeline.status),
                spec_yaml: pipeline.spec_yaml,
                created_at: Self::datetime_to_timestamp(pipeline.created_at),
                updated_at: Self::datetime_to_timestamp(pipeline.updated_at),
                stats: Some(PipelineStats {
                    records_processed: 0,
                    records_failed: 0,
                    predictions_made: 0,
                    feedback_received: 0,
                    current_accuracy: 0.0,
                }),
            }),
        };

        Ok(Response::new(response))
    }

    async fn list_pipelines(
        &self,
        request: Request<ListPipelinesRequest>,
    ) -> Result<Response<ListPipelinesResponse>, Status> {
        let req = request.into_inner();
        let limit = if req.limit > 0 { req.limit as u64 } else { 100 };
        let namespace = if req.namespace.is_empty() {
            None
        } else {
            Some(req.namespace.as_str())
        };

        let pipelines = PipelineRepo::list(self.db.conn(), namespace, limit, 0)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?;

        let proto_pipelines: Vec<PipelineInfo> = pipelines
            .into_iter()
            .map(|p| PipelineInfo {
                pipeline_id: p.id.to_string(),
                name: p.name,
                namespace: p.namespace,
                status: format!("{:?}", p.status),
                spec_yaml: p.spec_yaml,
                created_at: Self::datetime_to_timestamp(p.created_at),
                updated_at: Self::datetime_to_timestamp(p.updated_at),
                stats: Some(PipelineStats {
                    records_processed: 0,
                    records_failed: 0,
                    predictions_made: 0,
                    feedback_received: 0,
                    current_accuracy: 0.0,
                }),
            })
            .collect();

        let response = ListPipelinesResponse {
            pipelines: proto_pipelines,
            next_cursor: String::new(),
        };

        Ok(Response::new(response))
    }

    async fn enable_pipeline(
        &self,
        request: Request<EnablePipelineRequest>,
    ) -> Result<Response<EnablePipelineResponse>, Status> {
        let req = request.into_inner();
        let pipeline_id = Uuid::parse_str(&req.pipeline_id)
            .map_err(|_| Status::invalid_argument("Invalid pipeline ID format"))?;

        let pipeline = PipelineRepo::update_status(
            self.db.conn(),
            pipeline_id,
            pipeline::PipelineStatus::Running,
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to enable pipeline: {}", e)))?;

        tracing::info!(pipeline_id = %pipeline_id, "Pipeline enabled");

        let response = EnablePipelineResponse {
            success: true,
            status: format!("{:?}", pipeline.status),
        };

        Ok(Response::new(response))
    }

    async fn disable_pipeline(
        &self,
        request: Request<DisablePipelineRequest>,
    ) -> Result<Response<DisablePipelineResponse>, Status> {
        let req = request.into_inner();
        let pipeline_id = Uuid::parse_str(&req.pipeline_id)
            .map_err(|_| Status::invalid_argument("Invalid pipeline ID format"))?;

        let pipeline = PipelineRepo::update_status(
            self.db.conn(),
            pipeline_id,
            pipeline::PipelineStatus::Stopped,
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to disable pipeline: {}", e)))?;

        tracing::info!(pipeline_id = %pipeline_id, "Pipeline disabled");

        let response = DisablePipelineResponse {
            success: true,
            status: format!("{:?}", pipeline.status),
        };

        Ok(Response::new(response))
    }

    async fn register_model(
        &self,
        request: Request<RegisterModelRequest>,
    ) -> Result<Response<RegisterModelResponse>, Status> {
        let req = request.into_inner();

        if req.model_id.is_empty() {
            return Err(Status::invalid_argument("Model ID is required"));
        }

        let model = ModelVersionRepo::create(
            self.db.conn(),
            req.model_id.clone(),
            req.version.clone(),
            req.model_type,
            req.endpoint,
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to register model: {}", e)))?;

        tracing::info!(
            model_id = %req.model_id,
            version = %req.version,
            "Model registered"
        );

        let response = RegisterModelResponse {
            model_id: model.model_id,
            success: true,
        };

        Ok(Response::new(response))
    }

    async fn unregister_model(
        &self,
        request: Request<UnregisterModelRequest>,
    ) -> Result<Response<UnregisterModelResponse>, Status> {
        let req = request.into_inner();

        tracing::info!(model_id = %req.model_id, "Model unregistered");

        let response = UnregisterModelResponse { success: true };

        Ok(Response::new(response))
    }

    async fn get_model(
        &self,
        request: Request<GetModelRequest>,
    ) -> Result<Response<GetModelResponse>, Status> {
        let req = request.into_inner();

        let model = ModelVersionRepo::find_by_model_id(self.db.conn(), &req.model_id)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| Status::not_found("Model not found"))?;

        let response = GetModelResponse {
            model: Some(ModelInfo {
                model_id: model.model_id,
                model_name: String::new(),
                version: model.version,
                model_type: model.model_type,
                endpoint: model.endpoint,
                status: format!("{:?}", model.status),
                accuracy: model.accuracy.unwrap_or(0.0),
                latency_p99_ms: model.latency_p99_ms.unwrap_or(0) as u64,
                deployed_at: Self::datetime_to_timestamp(model.deployed_at),
                labels: std::collections::HashMap::new(),
            }),
        };

        Ok(Response::new(response))
    }

    async fn list_models(
        &self,
        request: Request<ListModelsRequest>,
    ) -> Result<Response<ListModelsResponse>, Status> {
        let req = request.into_inner();
        let limit = if req.limit > 0 { req.limit as u64 } else { 100 };

        let models = ModelVersionRepo::list(self.db.conn(), limit)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?;

        let proto_models: Vec<ModelInfo> = models
            .into_iter()
            .map(|m| ModelInfo {
                model_id: m.model_id,
                model_name: String::new(),
                version: m.version,
                model_type: m.model_type,
                endpoint: m.endpoint,
                status: format!("{:?}", m.status),
                accuracy: m.accuracy.unwrap_or(0.0),
                latency_p99_ms: m.latency_p99_ms.unwrap_or(0) as u64,
                deployed_at: Self::datetime_to_timestamp(m.deployed_at),
                labels: std::collections::HashMap::new(),
            })
            .collect();

        let response = ListModelsResponse {
            models: proto_models,
            next_cursor: String::new(),
        };

        Ok(Response::new(response))
    }
}
